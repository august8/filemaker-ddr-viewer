use std::io::BufRead;

use quick_xml::{events::Event, Reader};

use crate::parser::{
    helpers::{get_attr, skip_element},
    models::{Bounds, ConditionalFormat, LayoutFieldRef, LayoutObject},
    ParseError,
};

use super::{
    field_extractor::{
        parse_bounds, parse_field_elem_attrs, parse_field_reference_attrs,
        read_calculation_from_container, read_text_obj_label, scan_field_obj,
    },
    script_and_format::{
        deep_scan_for_scripts_and_fields, parse_conditional_formatting,
        parse_script_trigger_elements,
    },
    ObjectScanResult,
};

/// `<ObjectList>` の内容を走査してトリガー・ボタンスクリプト参照・フィールド参照・オブジェクト一覧を収集する。
pub(super) fn scan_object_list<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<ObjectScanResult, ParseError> {
    let mut triggers = Vec::new();
    let mut button_scripts = Vec::new();
    let mut field_refs = Vec::new();
    let mut layout_objects = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"Object" => {
                let obj_type = get_attr(e, b"type").unwrap_or_default();
                let obj_key = get_attr(e, b"key")
                    .and_then(|v| {
                        v.parse::<u64>()
                            .map_err(|_| ParseError::InvalidValue(format!("Object key: {v}")))
                    })
                    .unwrap_or(0);
                let obj_name = get_attr(e, b"name").ok().filter(|s| !s.is_empty());
                let (ot, os, ofr, olo) = scan_object(reader, buf, obj_type, obj_key, obj_name)?;
                triggers.extend(ot);
                button_scripts.extend(os);
                field_refs.extend(ofr);
                layout_objects.extend(olo);
            }
            Event::Empty(ref e) if e.name().as_ref() == b"Object" => {
                let obj_type = get_attr(e, b"type").unwrap_or_default();
                let obj_key = get_attr(e, b"key")
                    .and_then(|v| {
                        v.parse::<u64>()
                            .map_err(|_| ParseError::InvalidValue(format!("Object key: {v}")))
                    })
                    .unwrap_or(0);
                let obj_name = get_attr(e, b"name").ok().filter(|s| !s.is_empty());
                layout_objects.push(LayoutObject {
                    object_type: obj_type,
                    object_key: obj_key,
                    object_name: obj_name,
                    button_label: None,
                    field_table_occurrence: None,
                    field_name: None,
                    tooltip: None,
                    hide_condition: None,
                    bounds: None,
                    conditional_formats: Vec::new(),
                });
            }
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </ObjectList>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok((triggers, button_scripts, field_refs, layout_objects))
}

/// `<Object>` 要素内を走査して ScriptTrigger・ボタンスクリプト参照・フィールド参照・LayoutObject を収集する。
///
/// `object_type` と `object_key` は呼び出し元が `<Object>` 開始タグから読み取って渡す。
pub(super) fn scan_object<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    object_type: String,
    object_key: u64,
    object_name: Option<String>,
) -> Result<ObjectScanResult, ParseError> {
    let mut triggers = Vec::new();
    let mut button_scripts = Vec::new();
    let mut field_refs: Vec<LayoutFieldRef> = Vec::new();
    let mut layout_objects: Vec<LayoutObject> = Vec::new();

    // このオブジェクト自身の属性
    let mut field_table_occurrence: Option<String> = None;
    let mut field_name_val: Option<String> = None;
    let mut tooltip: Option<String> = None;
    let mut hide_condition: Option<String> = None;
    let mut bounds: Option<Bounds> = None;
    let mut conditional_formats: Vec<ConditionalFormat> = Vec::new();

    let is_field_obj = object_type == "Field";
    let mut button_label: Option<String> = None;

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            // ---- Bounds（位置・サイズ情報）----
            Event::Empty(ref e) if e.name().as_ref() == b"Bounds" => {
                bounds = Some(parse_bounds(e));
            }

            // ---- ToolTip ----
            Event::Start(ref e) if e.name().as_ref() == b"ToolTip" => {
                tooltip = read_calculation_from_container(reader, buf)?;
            }
            Event::Empty(ref e) if e.name().as_ref() == b"ToolTip" => {}

            // ---- HideCondition ----
            Event::Start(ref e) if e.name().as_ref() == b"HideCondition" => {
                hide_condition = read_calculation_from_container(reader, buf)?;
            }
            Event::Empty(ref e) if e.name().as_ref() == b"HideCondition" => {}

            // ---- FieldObj（type="Field" のみ解析）----
            Event::Start(ref e) if e.name().as_ref() == b"FieldObj" => {
                if is_field_obj {
                    let (ft, fn_) = scan_field_obj(reader, buf)?;
                    field_table_occurrence = ft.clone();
                    field_name_val = fn_.clone();
                    // layout_field_refs 用にも追加
                    if let (Some(t), Some(f)) = (ft, fn_) {
                        field_refs.push(LayoutFieldRef {
                            table_occurrence: t,
                            field_name: f,
                        });
                    }
                } else {
                    let (nested_scripts, nested_frs) =
                        deep_scan_for_scripts_and_fields(reader, buf)?;
                    button_scripts.extend(nested_scripts);
                    field_refs.extend(nested_frs);
                }
            }
            Event::Empty(ref e) if e.name().as_ref() == b"FieldObj" => {}

            // ---- FieldReference (self-closing) ----
            Event::Empty(ref e) if e.name().as_ref() == b"FieldReference" => {
                if let Some(fr) = parse_field_reference_attrs(e) {
                    field_refs.push(fr);
                }
            }
            // ---- FieldReference (with children) ----
            Event::Start(ref e) if e.name().as_ref() == b"FieldReference" => {
                if let Some(fr) = parse_field_reference_attrs(e) {
                    field_refs.push(fr);
                }
                skip_element(reader, buf)?;
            }
            // ---- 実DDR形式: <Field name="..." table="..."/> (self-closing) ----
            Event::Empty(ref e) if e.name().as_ref() == b"Field" => {
                if let Some(fr) = parse_field_elem_attrs(e) {
                    field_refs.push(fr);
                }
            }
            // ---- 実DDR形式: <Field name="..." table="..."> (with children) ----
            Event::Start(ref e) if e.name().as_ref() == b"Field" => {
                if let Some(fr) = parse_field_elem_attrs(e) {
                    field_refs.push(fr);
                }
                let (_, nested_frs) = deep_scan_for_scripts_and_fields(reader, buf)?;
                field_refs.extend(nested_frs);
            }

            // ---- ScriptTriggerList ----
            Event::Start(ref e) if e.name().as_ref() == b"ScriptTriggerList" => {
                triggers.extend(parse_script_trigger_elements(
                    reader,
                    buf,
                    b"ScriptTrigger",
                )?);
            }
            Event::Empty(ref e) if e.name().as_ref() == b"ScriptTriggerList" => {}
            // ---- 旧形式: ScriptTriggers ----
            Event::Start(ref e) if e.name().as_ref() == b"ScriptTriggers" => {
                triggers.extend(parse_script_trigger_elements(reader, buf, b"Trigger")?);
            }
            Event::Empty(ref e) if e.name().as_ref() == b"ScriptTriggers" => {}

            // ---- ネストされた Object ----
            Event::Start(ref e) if e.name().as_ref() == b"Object" => {
                let nested_type = get_attr(e, b"type").unwrap_or_default();
                let nested_key = get_attr(e, b"key")
                    .and_then(|v| {
                        v.parse::<u64>()
                            .map_err(|_| ParseError::InvalidValue(format!("Object key: {v}")))
                    })
                    .unwrap_or(0);
                let nested_name = get_attr(e, b"name").ok().filter(|s| !s.is_empty());
                let (nt, ns, nfr, nlo) =
                    scan_object(reader, buf, nested_type, nested_key, nested_name)?;
                triggers.extend(nt);
                button_scripts.extend(ns);
                field_refs.extend(nfr);
                layout_objects.extend(nlo);
            }
            // ---- ネストされた ObjectList ----
            Event::Start(ref e) if e.name().as_ref() == b"ObjectList" => {
                let (nt, ns, nfr, nlo) = scan_object_list(reader, buf)?;
                triggers.extend(nt);
                button_scripts.extend(ns);
                field_refs.extend(nfr);
                layout_objects.extend(nlo);
            }
            Event::Empty(ref e) if e.name().as_ref() == b"ObjectList" => {}

            // ---- Script (self-closing) ----
            Event::Empty(ref e) if e.name().as_ref() == b"Script" => {
                let file = get_attr(e, b"file").unwrap_or_default();
                if file.is_empty() {
                    if let Ok(name) = get_attr(e, b"name") {
                        if !name.is_empty() {
                            button_scripts.push(name);
                        }
                    }
                }
            }
            // ---- Script (with children) ----
            Event::Start(ref e) if e.name().as_ref() == b"Script" => {
                let file = get_attr(e, b"file").unwrap_or_default();
                if file.is_empty() {
                    if let Ok(name) = get_attr(e, b"name") {
                        if !name.is_empty() {
                            button_scripts.push(name);
                        }
                    }
                }
                skip_element(reader, buf)?;
            }
            // ---- ScriptReference (self-closing) ----
            Event::Empty(ref e) if e.name().as_ref() == b"ScriptReference" => {
                let file = get_attr(e, b"file").unwrap_or_default();
                if file.is_empty() {
                    if let Ok(name) = get_attr(e, b"name") {
                        if !name.is_empty() {
                            button_scripts.push(name);
                        }
                    }
                }
            }
            // ---- TextObj（ボタンラベル取得）----
            Event::Start(ref e) if e.name().as_ref() == b"TextObj" => {
                button_label = read_text_obj_label(reader, buf)?;
            }
            Event::Empty(ref e) if e.name().as_ref() == b"TextObj" => {}

            // ---- ConditionalFormatting ----
            Event::Start(ref e) if e.name().as_ref() == b"ConditionalFormatting" => {
                conditional_formats = parse_conditional_formatting(reader, buf)?;
            }
            Event::Empty(ref e) if e.name().as_ref() == b"ConditionalFormatting" => {}

            // その他の子要素は深掘りスキャン
            Event::Start(_) => {
                let (nested_scripts, nested_frs) = deep_scan_for_scripts_and_fields(reader, buf)?;
                button_scripts.extend(nested_scripts);
                field_refs.extend(nested_frs);
            }
            Event::Empty(_) => {}
            Event::End(_) => {
                // このオブジェクト自身の LayoutObject を追加
                layout_objects.push(LayoutObject {
                    object_type,
                    object_key,
                    object_name,
                    button_label,
                    field_table_occurrence,
                    field_name: field_name_val,
                    tooltip,
                    hide_condition,
                    bounds,
                    conditional_formats,
                });
                break;
            }
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok((triggers, button_scripts, field_refs, layout_objects))
}
