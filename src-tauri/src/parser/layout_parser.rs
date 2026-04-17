use std::io::BufRead;

use quick_xml::{events::Event, Reader};

use crate::parser::{
    helpers::{get_attr, read_text_content, skip_element},
    models::{
        Bounds, ConditionalFormat, Layout, LayoutFieldRef, LayoutId, LayoutObject, ScriptTrigger,
    },
    ParseError,
};

/// `scan_object` / `scan_object_list` の戻り値型エイリアス。
type ObjectScanResult = (
    Vec<ScriptTrigger>,
    Vec<String>,
    Vec<LayoutFieldRef>,
    Vec<LayoutObject>,
);

/// Parse `<LayoutCatalog>` content.
///
/// The caller must have already consumed the opening `<LayoutCatalog>` tag.
pub fn parse_layouts<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Vec<Layout>, ParseError> {
    let mut layouts = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"Layout" => {
                let id = get_attr(e, b"id")
                    .and_then(|v| {
                        v.parse::<u64>()
                            .map_err(|_| ParseError::InvalidValue(format!("Layout id: {v}")))
                    })
                    .unwrap_or(0);
                let name = get_attr(e, b"name").unwrap_or_default();
                let attr_table_name = get_attr(e, b"tableOccurrenceName").ok();
                let (
                    child_table_name,
                    script_triggers,
                    button_script_refs,
                    field_refs,
                    layout_objects,
                ) = parse_layout_children(reader, buf)?;
                let table_occurrence_name = attr_table_name.or(child_table_name);
                layouts.push(Layout {
                    id: LayoutId(id),
                    name,
                    table_occurrence_name,
                    script_triggers,
                    button_script_refs,
                    field_refs,
                    layout_objects,
                });
            }
            Event::Empty(ref e) if e.name().as_ref() == b"Layout" => {
                let id = get_attr(e, b"id")
                    .and_then(|v| {
                        v.parse::<u64>()
                            .map_err(|_| ParseError::InvalidValue(format!("Layout id: {v}")))
                    })
                    .unwrap_or(0);
                let name = get_attr(e, b"name").unwrap_or_default();
                let table_occurrence_name = get_attr(e, b"tableOccurrenceName").ok();
                layouts.push(Layout {
                    id: LayoutId(id),
                    name,
                    table_occurrence_name,
                    script_triggers: Vec::new(),
                    button_script_refs: Vec::new(),
                    field_refs: Vec::new(),
                    layout_objects: Vec::new(),
                });
            }
            // 実DDR: <Group> 内にレイアウトをグループ化 → 再帰
            Event::Start(ref e) if e.name().as_ref() == b"Group" => {
                let group_layouts = parse_layouts(reader, buf)?;
                layouts.extend(group_layouts);
            }
            Event::Empty(ref e) if e.name().as_ref() == b"Group" => {}
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </LayoutCatalog> or </Group>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok(layouts)
}

/// Parse children of a `<Layout>` element (opening tag already consumed).
///
/// Returns `(table_occurrence_name, triggers, button_script_refs, field_refs, layout_objects)`.
/// `parse_layout_children` の戻り値型エイリアス。
type LayoutChildrenResult = Result<
    (
        Option<String>,
        Vec<ScriptTrigger>,
        Vec<String>,
        Vec<LayoutFieldRef>,
        Vec<LayoutObject>,
    ),
    ParseError,
>;

fn parse_layout_children<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> LayoutChildrenResult {
    let mut table_occurrence_name: Option<String> = None;
    let mut triggers = Vec::new();
    let mut button_scripts: Vec<String> = Vec::new();
    let mut field_refs: Vec<LayoutFieldRef> = Vec::new();
    let mut layout_objects: Vec<LayoutObject> = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            // 実DDR形式: <Table name="顧客" id="1065089" />
            Event::Empty(ref e)
                if e.name().as_ref() == b"Table" && table_occurrence_name.is_none() =>
            {
                table_occurrence_name = get_attr(e, b"name").ok();
            }
            Event::Start(ref e) if e.name().as_ref() == b"Table" => {
                if table_occurrence_name.is_none() {
                    table_occurrence_name = get_attr(e, b"name").ok();
                }
                skip_element(reader, buf)?;
            }
            // 実DDR形式: <ObjectList>
            Event::Start(ref e) if e.name().as_ref() == b"ObjectList" => {
                let (ol_triggers, ol_scripts, ol_field_refs, ol_objects) =
                    scan_object_list(reader, buf)?;
                triggers.extend(ol_triggers);
                button_scripts.extend(ol_scripts);
                field_refs.extend(ol_field_refs);
                layout_objects.extend(ol_objects);
            }
            Event::Empty(ref e) if e.name().as_ref() == b"ObjectList" => {}
            // レイアウト直下の <Object>（ObjectList に包まれていない場合）
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
            // 実DDR形式: <ScriptTriggerList>
            Event::Start(ref e) if e.name().as_ref() == b"ScriptTriggerList" => {
                triggers.extend(parse_script_trigger_elements(
                    reader,
                    buf,
                    b"ScriptTrigger",
                )?);
            }
            Event::Empty(ref e) if e.name().as_ref() == b"ScriptTriggerList" => {}
            // 旧形式: <ScriptTriggers>
            Event::Start(ref e) if e.name().as_ref() == b"ScriptTriggers" => {
                triggers.extend(parse_script_trigger_elements(reader, buf, b"Trigger")?);
            }
            Event::Empty(ref e) if e.name().as_ref() == b"ScriptTriggers" => {}
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </Layout>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok((
        table_occurrence_name,
        triggers,
        button_scripts,
        field_refs,
        layout_objects,
    ))
}

/// `<ObjectList>` の内容を走査してトリガー・ボタンスクリプト参照・フィールド参照・オブジェクト一覧を収集する。
fn scan_object_list<R: BufRead>(
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

/// `<FieldReference>` 要素から `(tableOccurrence, field)` を抽出する。
fn parse_field_reference_attrs(e: &quick_xml::events::BytesStart<'_>) -> Option<LayoutFieldRef> {
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
fn parse_field_elem_attrs(e: &quick_xml::events::BytesStart<'_>) -> Option<LayoutFieldRef> {
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
fn read_calculation_from_container<R: BufRead>(
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
fn scan_field_obj<R: BufRead>(
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

/// `<Object>` 要素内を走査して ScriptTrigger・ボタンスクリプト参照・フィールド参照・LayoutObject を収集する。
///
/// `object_type` と `object_key` は呼び出し元が `<Object>` 開始タグから読み取って渡す。
fn scan_object<R: BufRead>(
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
                if let Ok(name) = get_attr(e, b"name") {
                    if !name.is_empty() {
                        button_scripts.push(name);
                    }
                }
            }
            // ---- Script (with children) ----
            Event::Start(ref e) if e.name().as_ref() == b"Script" => {
                if let Ok(name) = get_attr(e, b"name") {
                    if !name.is_empty() {
                        button_scripts.push(name);
                    }
                }
                skip_element(reader, buf)?;
            }
            // ---- ScriptReference (self-closing) ----
            Event::Empty(ref e) if e.name().as_ref() == b"ScriptReference" => {
                if let Ok(name) = get_attr(e, b"name") {
                    if !name.is_empty() {
                        button_scripts.push(name);
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

/// `<TextObj>` 内の最初の `<Data>` テキストを取り出す（ボタンラベル取得用）。
/// 呼び出し時点で `<TextObj>` の開始タグは消費済み。
fn read_text_obj_label<R: BufRead>(
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

/// 任意の要素内を深掘りして、スクリプト参照とフィールド参照を両方収集する。
fn deep_scan_for_scripts_and_fields<R: BufRead>(
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
fn parse_script_trigger_elements<R: BufRead>(
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

/// `<Bounds top="..." left="..." bottom="..." right="..."/>` 属性から `Bounds` を生成する。
fn parse_bounds(e: &quick_xml::events::BytesStart<'_>) -> Bounds {
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

/// `<ConditionalFormatting>` ブロックをパースして `ConditionalFormat` のベクタを返す。
/// 呼び出し時点で `<ConditionalFormatting>` 開始タグは消費済み。
fn parse_conditional_formatting<R: BufRead>(
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn make_reader(xml: &str) -> (Reader<&[u8]>, Vec<u8>) {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        (reader, Vec::new())
    }

    fn parse(xml: &str) -> Result<Vec<Layout>, ParseError> {
        let full = format!("<LayoutCatalog>{xml}</LayoutCatalog>");
        let (mut reader, mut buf) = make_reader(&full);
        loop {
            buf.clear();
            match reader.read_event_into(&mut buf).unwrap() {
                Event::Start(ref e) if e.name().as_ref() == b"LayoutCatalog" => break,
                _ => {}
            }
        }
        parse_layouts(&mut reader, &mut buf)
    }

    #[test]
    fn empty_catalog() {
        assert!(parse("").unwrap().is_empty());
    }

    #[test]
    fn single_layout_table_occurrence_as_attribute() {
        let layouts = parse(
            r#"<Layout id="1" encryptionState="NotProtected" name="Contact List" tableOccurrenceName="Contact"/>"#,
        )
        .unwrap();
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].name, "Contact List");
        assert_eq!(layouts[0].id, LayoutId(1));
        assert_eq!(layouts[0].table_occurrence_name.as_deref(), Some("Contact"));
        assert!(layouts[0].script_triggers.is_empty());
    }

    #[test]
    fn single_layout_table_occurrence_as_child_element() {
        let layouts = parse(
            r#"<Layout id="1" name="顧客一覧">
              <Table name="顧客" id="1065089" />
            </Layout>"#,
        )
        .unwrap();
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].table_occurrence_name.as_deref(), Some("顧客"));
    }

    #[test]
    fn layout_with_script_trigger_real_ddr_format() {
        let xml = r#"
        <Layout id="1" name="顧客一覧">
          <Table name="顧客" id="1065089" />
          <ScriptTriggerList>
            <ScriptTrigger event="OnRecordLoad">
              <Script id="4" name="レコード読み込み時" />
            </ScriptTrigger>
          </ScriptTriggerList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].table_occurrence_name.as_deref(), Some("顧客"));
        assert_eq!(layouts[0].script_triggers.len(), 1);
        let trigger = &layouts[0].script_triggers[0];
        assert_eq!(trigger.event, "OnRecordLoad");
        assert_eq!(trigger.script_name, "レコード読み込み時");
    }

    #[test]
    fn layout_with_script_trigger_old_format() {
        let xml = r#"
        <Layout id="1" name="Contact List" tableOccurrenceName="Contact">
          <ScriptTriggers>
            <Trigger event="OnRecordLoad" id="1" triggerFlags="1">
              <Script id="1" name="My Script"/>
              <TriggerText>"My Script"</TriggerText>
            </Trigger>
          </ScriptTriggers>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].script_triggers.len(), 1);
        let trigger = &layouts[0].script_triggers[0];
        assert_eq!(trigger.event, "OnRecordLoad");
        assert_eq!(trigger.script_name, "My Script");
    }

    #[test]
    fn button_script_refs_from_object_list() {
        let xml = r#"
        <Layout id="1" name="顧客一覧">
          <Table name="顧客" id="1065089" />
          <ObjectList>
            <Object type="Field" key="1">
              <FieldObj numOfReps="1" flags="0">
                <DDRInfo>
                  <Field name="顧客名" id="2" repetition="1" maxRepetition="1" table="顧客"/>
                </DDRInfo>
              </FieldObj>
            </Object>
            <Object type="Button" key="2">
              <Script name="顧客登録_メイン" id="1" />
            </Object>
          </ObjectList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        assert_eq!(layouts.len(), 1);
        assert!(layouts[0]
            .button_script_refs
            .contains(&"顧客登録_メイン".to_string()));
        assert_eq!(layouts[0].button_script_refs.len(), 1);
    }

    #[test]
    fn nested_object_script_refs_are_collected() {
        let xml = r#"
        <Layout id="1" name="Detail">
          <Table name="Contact" id="1065089" />
          <ObjectList>
            <Object type="Portal" key="1">
              <ObjectList>
                <Object type="Button" key="2">
                  <Script name="PortalButton" id="5" />
                </Object>
              </ObjectList>
            </Object>
          </ObjectList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        assert!(layouts[0]
            .button_script_refs
            .contains(&"PortalButton".to_string()));
    }

    #[test]
    fn object_level_triggers_are_collected() {
        let xml = r#"
        <Layout id="1" name="Detail">
          <Table name="Contact" id="1065089" />
          <ObjectList>
            <Object type="Field" key="1">
              <ScriptTriggerList>
                <ScriptTrigger event="OnObjectValidate">
                  <Script id="42" name="Validate Script"/>
                </ScriptTrigger>
              </ScriptTriggerList>
            </Object>
          </ObjectList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        assert_eq!(layouts[0].table_occurrence_name.as_deref(), Some("Contact"));
        assert_eq!(layouts[0].script_triggers.len(), 1);
        assert_eq!(layouts[0].script_triggers[0].script_name, "Validate Script");
        assert_eq!(layouts[0].script_triggers[0].event, "OnObjectValidate");
    }

    #[test]
    fn empty_script_trigger_list_is_ignored() {
        let xml = r#"
        <Layout id="1" name="L">
          <ScriptTriggerList/>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        assert!(layouts[0].script_triggers.is_empty());
    }

    // 実DDR形式: <FieldObj><DDRInfo><Field name table> でフィールド参照を収集
    #[test]
    fn field_object_captures_ddr_info() {
        let xml = r#"
        <Layout id="1" name="Invoice Detail">
          <Table name="Invoice" id="1065089" />
          <ObjectList>
            <Object type="Field" key="68">
              <FieldObj numOfReps="1" flags="32">
                <Name>Grobal::__ONE</Name>
                <DDRInfo>
                  <Field name="__ONE" id="55" repetition="1" maxRepetition="1" table="Grobal"/>
                </DDRInfo>
              </FieldObj>
            </Object>
          </ObjectList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        assert_eq!(layouts.len(), 1);
        let objs = &layouts[0].layout_objects;
        assert_eq!(objs.len(), 1);
        assert_eq!(objs[0].object_type, "Field");
        assert_eq!(objs[0].object_key, 68);
        assert_eq!(objs[0].field_table_occurrence.as_deref(), Some("Grobal"));
        assert_eq!(objs[0].field_name.as_deref(), Some("__ONE"));
        // layout_field_refs にも反映されること
        assert_eq!(layouts[0].field_refs.len(), 1);
        assert_eq!(layouts[0].field_refs[0].field_name, "__ONE");
    }

    // <FieldReference tableOccurrence="..." field="..."/> 形式も引き続き動作する
    #[test]
    fn field_reference_elem_refs_are_collected() {
        let xml = r#"
        <Layout id="1" name="Contact List">
          <Table name="Contact" id="1065089" />
          <ObjectList>
            <Object type="Field" key="1">
              <FieldReference tableOccurrence="Contact" field="FirstName" fieldId="3" />
            </Object>
          </ObjectList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        assert_eq!(layouts[0].field_refs.len(), 1);
        assert_eq!(layouts[0].field_refs[0].field_name, "FirstName");
        assert_eq!(layouts[0].field_refs[0].table_occurrence, "Contact");
    }

    // <Field name="..." table="..."/> 形式 (DDRSample 実測形式)
    #[test]
    fn field_elem_refs_are_collected() {
        let xml = r#"
        <Layout id="1" name="Invoice Detail">
          <Table name="Invoice" id="1065089" />
          <ObjectList>
            <Object type="Field" key="1">
              <Field name="CurrencyCode" id="13" repetition="1" maxRepetition="1" table="CurrencyMasterImportLog"/>
            </Object>
            <Object type="Field" key="2">
              <Field name="Amount" id="5" repetition="1" maxRepetition="1" table="Invoice"/>
            </Object>
          </ObjectList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        assert_eq!(layouts.len(), 1);
        let refs = &layouts[0].field_refs;
        assert!(refs
            .iter()
            .any(|r| r.field_name == "CurrencyCode"
                && r.table_occurrence == "CurrencyMasterImportLog"));
        assert!(refs
            .iter()
            .any(|r| r.field_name == "Amount" && r.table_occurrence == "Invoice"));
    }

    // ToolTip の収集
    #[test]
    fn object_tooltip_is_captured() {
        let xml = r#"
        <Layout id="1" name="L">
          <Table name="T" id="1" />
          <ObjectList>
            <Object type="Button" key="316">
              <ToolTip>
                <Calculation><![CDATA["新規アカウントの追加を行います。"]]></Calculation>
                <DisplayCalculation><![CDATA["新規アカウントの追加を行います。"]]></DisplayCalculation>
              </ToolTip>
            </Object>
          </ObjectList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        let objs = &layouts[0].layout_objects;
        assert_eq!(objs.len(), 1);
        assert_eq!(objs[0].object_type, "Button");
        assert_eq!(objs[0].object_key, 316);
        assert_eq!(
            objs[0].tooltip.as_deref(),
            Some("\"新規アカウントの追加を行います。\"")
        );
        assert!(objs[0].hide_condition.is_none());
    }

    // HideCondition の収集
    #[test]
    fn object_hide_condition_is_captured() {
        let xml = r#"
        <Layout id="1" name="L">
          <Table name="T" id="1" />
          <ObjectList>
            <Object type="Rect" key="25">
              <HideCondition findMode="False">
                <Calculation><![CDATA[1]]></Calculation>
                <DisplayCalculation><![CDATA[1]]></DisplayCalculation>
              </HideCondition>
            </Object>
          </ObjectList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        let objs = &layouts[0].layout_objects;
        assert_eq!(objs.len(), 1);
        assert_eq!(objs[0].object_type, "Rect");
        assert_eq!(objs[0].hide_condition.as_deref(), Some("1"));
    }

    // <Bounds> タグから位置情報が取得できる
    #[test]
    fn object_bounds_are_captured() {
        let xml = r#"
        <Layout id="1" name="L">
          <Table name="T" id="1" />
          <ObjectList>
            <Object type="Field" key="68" flags="8320" rotation="0">
              <Bounds top="25.0000000" left="120.0000000" bottom="47.0000000" right="240.0000000"/>
              <FieldObj numOfReps="1" flags="32">
                <DDRInfo>
                  <Field name="FirstName" id="3" repetition="1" maxRepetition="1" table="Contact"/>
                </DDRInfo>
              </FieldObj>
            </Object>
          </ObjectList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        let objs = &layouts[0].layout_objects;
        assert_eq!(objs.len(), 1);
        let b = objs[0].bounds.as_ref().expect("bounds should be Some");
        assert!((b.top - 25.0).abs() < 1e-6);
        assert!((b.left - 120.0).abs() < 1e-6);
        assert!((b.bottom - 47.0).abs() < 1e-6);
        assert!((b.right - 240.0).abs() < 1e-6);
    }

    // 非フィールドオブジェクトはフィールド情報を持たない
    #[test]
    fn non_field_object_has_no_field_info() {
        let xml = r#"
        <Layout id="1" name="L">
          <Table name="T" id="1" />
          <ObjectList>
            <Object type="Button" key="1">
              <Script name="MyScript" id="2"/>
            </Object>
          </ObjectList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        let objs = &layouts[0].layout_objects;
        assert_eq!(objs.len(), 1);
        assert!(objs[0].field_table_occurrence.is_none());
        assert!(objs[0].field_name.is_none());
    }

    #[rstest]
    #[case(1)]
    #[case(3)]
    #[case(5)]
    fn multiple_layouts(#[case] count: usize) {
        let inner: String = (1..=count)
            .map(|i| {
                format!(
                    r#"<Layout id="{i}" name="Layout{i}"><Table name="T{i}" id="{i}"/></Layout>"#
                )
            })
            .collect();
        let layouts = parse(&inner).unwrap();
        assert_eq!(layouts.len(), count);
        for (i, layout) in layouts.iter().enumerate() {
            assert_eq!(
                layout.table_occurrence_name.as_deref(),
                Some(format!("T{}", i + 1).as_str())
            );
        }
    }

    #[test]
    fn parse_conditional_format_from_xml() {
        let xml = r#"
        <Layout id="1" name="TestLayout">
          <ObjectList>
            <Object type="Text" key="400">
              <ConditionalFormatting>
                <Item id="0" flags="3">
                  <Condition op="0">
                    <Calculation><![CDATA[Table::Field = 0]]></Calculation>
                  </Condition>
                  <Format><Styles><LocalCSS>color: rgba(33%,55%,15%,1);</LocalCSS></Styles></Format>
                </Item>
                <Item id="1" flags="3">
                  <Condition op="0">
                    <Calculation><![CDATA[Table::Field = 1]]></Calculation>
                  </Condition>
                  <Format><Styles><LocalCSS>color: rgba(86%,12%,40%,1);</LocalCSS></Styles></Format>
                </Item>
              </ConditionalFormatting>
              <Bounds top="0" left="0" bottom="20" right="100"/>
            </Object>
          </ObjectList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        assert_eq!(layouts.len(), 1);
        let obj = &layouts[0].layout_objects[0];
        assert_eq!(
            obj.conditional_formats.len(),
            2,
            "条件付き書式が2件パースされること"
        );
        assert_eq!(obj.conditional_formats[0].rule_order, 0);
        assert_eq!(obj.conditional_formats[0].calculation, "Table::Field = 0");
        assert!(
            obj.conditional_formats[0].format_css.contains("rgba"),
            "format_css に CSS が含まれること"
        );
        assert_eq!(obj.conditional_formats[1].rule_order, 1);
        assert_eq!(obj.conditional_formats[1].calculation, "Table::Field = 1");
    }

    #[test]
    fn object_without_conditional_format_has_empty_vec() {
        let xml = r#"
        <Layout id="1" name="TestLayout">
          <ObjectList>
            <Object type="Field" key="1">
              <Bounds top="0" left="0" bottom="20" right="100"/>
            </Object>
          </ObjectList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        assert_eq!(layouts.len(), 1);
        let obj = &layouts[0].layout_objects[0];
        assert!(
            obj.conditional_formats.is_empty(),
            "条件付き書式なしのオブジェクトは空ベクタ"
        );
    }

    // -----------------------------------------------------------------------
    // Group 再帰テスト
    // -----------------------------------------------------------------------

    #[test]
    fn group_layouts_are_flattened() {
        let xml = r#"
        <Group name="GroupA">
          <Layout id="2" name="Inner" tableOccurrenceName="Order"/>
        </Group>
        <Layout id="1" name="Top" tableOccurrenceName="Invoice"/>
        "#;
        let layouts = parse(xml).unwrap();
        assert_eq!(layouts.len(), 2);
        let names: Vec<_> = layouts.iter().map(|l| l.name.as_str()).collect();
        assert!(names.contains(&"Inner"));
        assert!(names.contains(&"Top"));
    }

    #[test]
    fn empty_group_tag_is_ignored() {
        let xml = r#"
        <Group/>
        <Layout id="1" name="Only" tableOccurrenceName="T"/>
        "#;
        let layouts = parse(xml).unwrap();
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].name, "Only");
    }

    #[test]
    fn nested_group_recursive() {
        let xml = r#"
        <Group name="Outer">
          <Group name="Inner">
            <Layout id="3" name="Deep" tableOccurrenceName="T"/>
          </Group>
        </Group>
        "#;
        let layouts = parse(xml).unwrap();
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].name, "Deep");
    }

    // -----------------------------------------------------------------------
    // button_label / TextObj テスト
    // -----------------------------------------------------------------------

    #[test]
    fn button_label_is_captured_from_text_obj() {
        let xml = r#"
        <Layout id="1" name="L" tableOccurrenceName="T">
          <ObjectList>
            <Object type="Button" key="10">
              <TextObj>
                <Data>保存</Data>
              </TextObj>
            </Object>
          </ObjectList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        let obj = &layouts[0].layout_objects[0];
        assert_eq!(obj.button_label.as_deref(), Some("保存"));
    }

    #[test]
    fn button_label_is_none_when_text_obj_has_no_data() {
        let xml = r#"
        <Layout id="1" name="L" tableOccurrenceName="T">
          <ObjectList>
            <Object type="Button" key="10">
              <TextObj/>
            </Object>
          </ObjectList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        let obj = &layouts[0].layout_objects[0];
        assert!(obj.button_label.is_none());
    }

    // -----------------------------------------------------------------------
    // FieldReference の children 形式（self-closing でない）
    // -----------------------------------------------------------------------

    #[test]
    fn field_reference_with_children_is_parsed() {
        let xml = r#"
        <Layout id="1" name="L" tableOccurrenceName="T">
          <ObjectList>
            <Object type="Field" key="1">
              <FieldReference tableOccurrence="Invoice" field="Amount" fieldId="3">
                <SomeChild/>
              </FieldReference>
            </Object>
          </ObjectList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        assert_eq!(layouts[0].field_refs.len(), 1);
        assert_eq!(layouts[0].field_refs[0].field_name, "Amount");
        assert_eq!(layouts[0].field_refs[0].table_occurrence, "Invoice");
    }

    #[test]
    fn field_reference_with_table_attr_fallback() {
        // tableOccurrence がなく table 属性のみの場合のフォールバック
        let xml = r#"
        <Layout id="1" name="L" tableOccurrenceName="T">
          <ObjectList>
            <Object type="Field" key="1">
              <FieldReference table="Contact" field="Name"/>
            </Object>
          </ObjectList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        assert_eq!(layouts[0].field_refs.len(), 1);
        assert_eq!(layouts[0].field_refs[0].table_occurrence, "Contact");
        assert_eq!(layouts[0].field_refs[0].field_name, "Name");
    }

    // -----------------------------------------------------------------------
    // ScriptReference テスト
    // -----------------------------------------------------------------------

    #[test]
    fn script_reference_elem_is_collected() {
        let xml = r#"
        <Layout id="1" name="L" tableOccurrenceName="T">
          <ObjectList>
            <Object type="Button" key="5">
              <ScriptReference name="OnClick Script" id="7"/>
            </Object>
          </ObjectList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        assert!(
            layouts[0]
                .button_script_refs
                .contains(&"OnClick Script".to_string()),
            "ScriptReference が button_script_refs に含まれる"
        );
    }

    // -----------------------------------------------------------------------
    // Script with children (non-empty tag)
    // -----------------------------------------------------------------------

    #[test]
    fn script_with_children_is_collected() {
        let xml = r#"
        <Layout id="1" name="L" tableOccurrenceName="T">
          <ObjectList>
            <Object type="Button" key="1">
              <Script name="MyScript" id="2">
                <SomeChild/>
              </Script>
            </Object>
          </ObjectList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        assert!(layouts[0]
            .button_script_refs
            .contains(&"MyScript".to_string()));
    }

    // -----------------------------------------------------------------------
    // FieldObj on non-Field type → deep_scan_for_scripts_and_fields
    // -----------------------------------------------------------------------

    #[test]
    fn field_obj_in_non_field_type_deep_scans_scripts() {
        let xml = r#"
        <Layout id="1" name="L" tableOccurrenceName="T">
          <ObjectList>
            <Object type="Portal" key="99">
              <FieldObj numOfReps="1">
                <DDRInfo>
                  <Field name="Amount" table="Invoice"/>
                </DDRInfo>
              </FieldObj>
            </Object>
          </ObjectList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        // Portal は Field 型ではないので field_table_occurrence は設定されないが
        // deep_scan が FieldReference/Field を収集する
        let obj = &layouts[0].layout_objects[0];
        assert_eq!(obj.object_type, "Portal");
    }

    // -----------------------------------------------------------------------
    // Field with children in scan_object
    // -----------------------------------------------------------------------

    #[test]
    fn field_elem_with_children_is_parsed() {
        let xml = r#"
        <Layout id="1" name="L" tableOccurrenceName="T">
          <ObjectList>
            <Object type="Field" key="1">
              <Field name="Amount" table="Invoice" id="5">
                <SomeChild/>
              </Field>
            </Object>
          </ObjectList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        assert!(layouts[0]
            .field_refs
            .iter()
            .any(|r| r.field_name == "Amount" && r.table_occurrence == "Invoice"));
    }

    // -----------------------------------------------------------------------
    // Object 直下（ObjectList なし）
    // -----------------------------------------------------------------------

    #[test]
    fn object_directly_under_layout_is_collected() {
        let xml = r#"
        <Layout id="1" name="L" tableOccurrenceName="T">
          <Object type="Button" key="7">
            <Script name="DirectScript" id="1"/>
          </Object>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        assert_eq!(layouts[0].layout_objects.len(), 1);
        assert!(layouts[0]
            .button_script_refs
            .contains(&"DirectScript".to_string()));
    }

    // -----------------------------------------------------------------------
    // Empty ToolTip / HideCondition タグ
    // -----------------------------------------------------------------------

    #[test]
    fn empty_tooltip_tag_is_ignored() {
        let xml = r#"
        <Layout id="1" name="L" tableOccurrenceName="T">
          <ObjectList>
            <Object type="Button" key="1">
              <ToolTip/>
            </Object>
          </ObjectList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        assert!(layouts[0].layout_objects[0].tooltip.is_none());
    }

    #[test]
    fn empty_hide_condition_tag_is_ignored() {
        let xml = r#"
        <Layout id="1" name="L" tableOccurrenceName="T">
          <ObjectList>
            <Object type="Button" key="1">
              <HideCondition/>
            </Object>
          </ObjectList>
        </Layout>
        "#;
        let layouts = parse(xml).unwrap();
        assert!(layouts[0].layout_objects[0].hide_condition.is_none());
    }
}
