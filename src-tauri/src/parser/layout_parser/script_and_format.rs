use std::io::BufRead;

use quick_xml::{events::Event, Reader};

use crate::parser::{
    helpers::{get_attr, read_text_content, skip_element},
    models::{ConditionalFormat, LayoutFieldRef, ScriptTrigger},
    ParseError,
};

use super::field_extractor::{parse_field_elem_attrs, parse_field_reference_attrs};

/// 任意の要素内を深掘りして、スクリプト参照とフィールド参照を両方収集する。
pub(super) fn deep_scan_for_scripts_and_fields<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<(Vec<String>, Vec<LayoutFieldRef>), ParseError> {
    let mut scripts = Vec::new();
    let mut field_refs = Vec::new();
    let mut depth: u32 = 1;

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) => {
                let name_bytes = e.name().as_ref().to_vec();
                if name_bytes == b"Script" || name_bytes == b"ScriptReference" {
                    let file = get_attr(e, b"file").unwrap_or_default();
                    if file.is_empty() {
                        if let Ok(name) = get_attr(e, b"name") {
                            if !name.is_empty() {
                                scripts.push(name);
                            }
                        }
                    }
                } else if name_bytes == b"FieldReference" {
                    if let Some(fr) = parse_field_reference_attrs(e) {
                        field_refs.push(fr);
                    }
                } else if name_bytes == b"Field" {
                    if let Some(fr) = parse_field_elem_attrs(e) {
                        field_refs.push(fr);
                    }
                }
                depth += 1;
            }
            Event::Empty(ref e) => {
                let name_bytes = e.name().as_ref().to_vec();
                if name_bytes == b"Script" || name_bytes == b"ScriptReference" {
                    let file = get_attr(e, b"file").unwrap_or_default();
                    if file.is_empty() {
                        if let Ok(name) = get_attr(e, b"name") {
                            if !name.is_empty() {
                                scripts.push(name);
                            }
                        }
                    }
                } else if name_bytes == b"FieldReference" {
                    if let Some(fr) = parse_field_reference_attrs(e) {
                        field_refs.push(fr);
                    }
                } else if name_bytes == b"Field" {
                    if let Some(fr) = parse_field_elem_attrs(e) {
                        field_refs.push(fr);
                    }
                }
            }
            Event::End(_) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok((scripts, field_refs))
}

/// トリガーコンテナ（`<ScriptTriggerList>` または `<ScriptTriggers>`）の内容をパース。
pub(super) fn parse_script_trigger_elements<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    trigger_elem: &[u8],
) -> Result<Vec<ScriptTrigger>, ParseError> {
    let mut triggers = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == trigger_elem => {
                let event = get_attr(e, b"event").unwrap_or_default();
                let (script_name, file_name) = parse_trigger_script_ref(reader, buf)?;
                if !script_name.is_empty() {
                    triggers.push(ScriptTrigger {
                        event,
                        script_name,
                        file_name,
                    });
                }
            }
            Event::Empty(ref e) if e.name().as_ref() == trigger_elem => {}
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break,
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok(triggers)
}

/// `<ScriptTrigger>` または `<Trigger>` の子要素から `<Script name="..." file="..."/>` を取得する。
fn parse_trigger_script_ref<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<(String, String), ParseError> {
    let mut script_name = String::new();
    let mut file_name = String::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Empty(ref e) if e.name().as_ref() == b"Script" => {
                script_name = get_attr(e, b"name").unwrap_or_default();
                file_name = get_attr(e, b"file").unwrap_or_default();
            }
            Event::Start(ref e) if e.name().as_ref() == b"Script" => {
                script_name = get_attr(e, b"name").unwrap_or_default();
                file_name = get_attr(e, b"file").unwrap_or_default();
                skip_element(reader, buf)?;
            }
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break,
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok((script_name, file_name))
}

/// `<ConditionalFormatting>` ブロックをパースして `ConditionalFormat` のベクタを返す。
/// 呼び出し時点で `<ConditionalFormatting>` 開始タグは消費済み。
pub(super) fn parse_conditional_formatting<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Vec<ConditionalFormat>, ParseError> {
    let mut formats: Vec<ConditionalFormat> = Vec::new();
    let mut current_rule_order: u32 = 0;
    let mut current_calculation: Option<String> = None;
    let mut current_format_css: Option<String> = None;

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) => match e.name().as_ref() {
                b"Item" => {
                    current_rule_order = get_attr(e, b"id")
                        .ok()
                        .and_then(|v| v.parse::<u32>().ok())
                        .unwrap_or(0);
                    current_calculation = None;
                    current_format_css = None;
                }
                b"Calculation" => {
                    let text = read_text_content(reader, buf)?;
                    if !text.is_empty() {
                        current_calculation = Some(text);
                    }
                }
                b"LocalCSS" => {
                    let text = read_text_content(reader, buf)?;
                    if !text.is_empty() {
                        current_format_css = Some(text);
                    }
                }
                _ => {}
            },
            Event::End(ref e) => match e.name().as_ref() {
                b"ConditionalFormatting" => break,
                b"Item" => {
                    if let Some(calc) = current_calculation.take() {
                        formats.push(ConditionalFormat {
                            rule_order: current_rule_order,
                            calculation: calc,
                            format_css: current_format_css.take().unwrap_or_default(),
                        });
                    }
                }
                _ => {}
            },
            Event::Empty(_) => {}
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok(formats)
}
