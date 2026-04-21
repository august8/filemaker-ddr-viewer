use std::io::BufRead;

use quick_xml::{events::Event, Reader};

use crate::parser::{
    helpers::{get_attr, read_text_content, skip_element},
    models::{Bounds, LayoutFieldRef},
    ParseError,
};

/// `<FieldReference>` 要素から `(tableOccurrence, field)` を抽出する。
pub(super) fn parse_field_reference_attrs(
    e: &quick_xml::events::BytesStart<'_>,
) -> Option<LayoutFieldRef> {
    let table_occurrence = get_attr(e, b"tableOccurrence")
        .or_else(|_| get_attr(e, b"table"))
        .unwrap_or_default();
    let field_name = get_attr(e, b"field").unwrap_or_default();
    if !table_occurrence.is_empty() && !field_name.is_empty() {
        Some(LayoutFieldRef {
            table_occurrence,
            field_name,
        })
    } else {
        None
    }
}

/// `<Field name="..." table="..."/>` 要素からフィールド参照を抽出する。
pub(super) fn parse_field_elem_attrs(
    e: &quick_xml::events::BytesStart<'_>,
) -> Option<LayoutFieldRef> {
    let table_occurrence = get_attr(e, b"table").unwrap_or_default();
    let field_name = get_attr(e, b"name").unwrap_or_default();
    if !table_occurrence.is_empty() && !field_name.is_empty() {
        Some(LayoutFieldRef {
            table_occurrence,
            field_name,
        })
    } else {
        None
    }
}

/// `<ToolTip>` や `<HideCondition>` の子要素 `<Calculation>` から計算式文字列を取得する。
///
/// 呼び出し時点: 親の開始タグ直後。親の閉じタグまで消費して返す。
pub(super) fn read_calculation_from_container<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Option<String>, ParseError> {
    let mut result: Option<String> = None;

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"Calculation" => {
                let text = read_text_content(reader, buf)?;
                if !text.is_empty() {
                    result = Some(text);
                }
            }
            Event::Empty(ref e) if e.name().as_ref() == b"Calculation" => {}
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break,
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok(result)
}

/// `<FieldObj>` 内を走査して `<DDRInfo><Field name="..." table="..."/>` を返す。
pub(super) fn scan_field_obj<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<(Option<String>, Option<String>), ParseError> {
    let mut table_occ: Option<String> = None;
    let mut field_name: Option<String> = None;

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"DDRInfo" => {
                let (t, f) = scan_ddr_info(reader, buf)?;
                table_occ = t;
                field_name = f;
            }
            Event::Empty(ref e) if e.name().as_ref() == b"DDRInfo" => {}
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </FieldObj>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }
    Ok((table_occ, field_name))
}

/// `<DDRInfo>` 内を走査して `<Field name="..." table="..."/>` を返す。
fn scan_ddr_info<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<(Option<String>, Option<String>), ParseError> {
    let mut table_occ: Option<String> = None;
    let mut field_name: Option<String> = None;

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Empty(ref e) if e.name().as_ref() == b"Field" => {
                table_occ = get_attr(e, b"table").ok();
                field_name = get_attr(e, b"name").ok();
            }
            Event::Start(ref e) if e.name().as_ref() == b"Field" => {
                table_occ = get_attr(e, b"table").ok();
                field_name = get_attr(e, b"name").ok();
                skip_element(reader, buf)?;
            }
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </DDRInfo>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }
    Ok((table_occ, field_name))
}

/// `<TextObj>` 内の最初の `<Data>` テキストを取り出す（ボタンラベル取得用）。
/// 呼び出し時点で `<TextObj>` の開始タグは消費済み。
pub(super) fn read_text_obj_label<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Option<String>, ParseError> {
    let mut depth: u32 = 1;
    let mut in_data = false;
    let mut label: Option<String> = None;

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) => {
                if e.name().as_ref() == b"Data" && label.is_none() {
                    in_data = true;
                }
                depth += 1;
            }
            Event::Text(ref e) if in_data => {
                if let Ok(decoded) = e.decode() {
                    let text = quick_xml::escape::unescape(&decoded)
                        .unwrap_or_default()
                        .trim()
                        .to_string();
                    if !text.is_empty() {
                        label = Some(text);
                    }
                }
                in_data = false;
            }
            Event::End(_) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
                in_data = false;
            }
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {
                in_data = false;
            }
        }
    }

    Ok(label)
}

/// `<Bounds top="..." left="..." bottom="..." right="..."/>` 属性から `Bounds` を生成する。
pub(super) fn parse_bounds(e: &quick_xml::events::BytesStart<'_>) -> Bounds {
    let mut top = 0.0f64;
    let mut left = 0.0f64;
    let mut bottom = 0.0f64;
    let mut right = 0.0f64;
    for attr in e.attributes().flatten() {
        let val: f64 = std::str::from_utf8(attr.value.as_ref())
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        match attr.key.as_ref() {
            b"top" => top = val,
            b"left" => left = val,
            b"bottom" => bottom = val,
            b"right" => right = val,
            _ => {}
        }
    }
    Bounds {
        top,
        left,
        bottom,
        right,
    }
}
