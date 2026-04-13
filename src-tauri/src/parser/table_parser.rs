use std::io::BufRead;

use quick_xml::{events::Event, Reader};

use crate::parser::{
    helpers::{get_attr, read_text_content, skip_element},
    models::{DataType, Field, FieldId, FieldKind, Table, TableId},
    ParseError,
};

/// Parse `<BaseTableCatalog>` content.
///
/// The caller must have already consumed the opening `<BaseTableCatalog>` tag.
/// Parsing stops when the matching `</BaseTableCatalog>` end tag is encountered.
pub fn parse_tables<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Vec<Table>, ParseError> {
    let mut tables = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"BaseTable" => {
                let id = get_attr(e, b"id")
                    .and_then(|v| {
                        v.parse::<u64>()
                            .map_err(|_| ParseError::InvalidValue(format!("BaseTable id: {v}")))
                    })
                    .unwrap_or(0);
                let name = get_attr(e, b"name").unwrap_or_default();
                // Drop the borrow of `buf` via `e` before passing `buf` to sub-parser.
                let fields = parse_base_table_fields(reader, buf)?;
                tables.push(Table {
                    id: TableId(id),
                    name,
                    fields,
                });
            }
            // An empty <BaseTable .../> has no fields
            Event::Empty(ref e) if e.name().as_ref() == b"BaseTable" => {
                let id = get_attr(e, b"id")
                    .and_then(|v| {
                        v.parse::<u64>()
                            .map_err(|_| ParseError::InvalidValue(format!("BaseTable id: {v}")))
                    })
                    .unwrap_or(0);
                let name = get_attr(e, b"name").unwrap_or_default();
                tables.push(Table {
                    id: TableId(id),
                    name,
                    fields: Vec::new(),
                });
            }
            // Skip unknown start tags and their children
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </BaseTableCatalog>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok(tables)
}

/// Parse the content of a single `<BaseTable>` element (already consumed).
///
/// Reads `<FieldCatalog>` → `<Field>` children, then consumes `</BaseTable>`.
fn parse_base_table_fields<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Vec<Field>, ParseError> {
    let mut fields = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"FieldCatalog" => {
                fields = parse_field_catalog(reader, buf)?;
            }
            Event::Empty(ref e) if e.name().as_ref() == b"FieldCatalog" => {
                // Empty catalog – no fields
            }
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </BaseTable>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok(fields)
}

/// Parse the content of a `<FieldCatalog>` element (already consumed).
fn parse_field_catalog<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Vec<Field>, ParseError> {
    let mut fields = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"Field" => {
                // Extract attributes before dropping the borrow on `buf`.
                let attrs = extract_field_attrs(e)?;
                let field = parse_field_children(reader, buf, attrs)?;
                fields.push(field);
            }
            Event::Empty(ref e) if e.name().as_ref() == b"Field" => {
                let attrs = extract_field_attrs(e)?;
                fields.push(Field {
                    id: attrs.0,
                    name: attrs.1,
                    data_type: attrs.2,
                    field_type: attrs.3,
                    comment: attrs.4,
                    is_global: false,
                    max_repeat: 1,
                    calculation: None,
                    auto_enter_type: String::new(),
                    auto_enter_calc: None,
                    auto_enter_allow_editing: true,
                    val_not_empty: false,
                    val_unique: false,
                    val_existing: false,
                    val_max_length: None,
                    val_value_list: None,
                    val_calc: None,
                    val_range_from: None,
                    val_range_to: None,
                    val_always: false,
                    val_error_message: None,
                    index_type: String::new(),
                });
            }
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </FieldCatalog>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok(fields)
}

type FieldAttrs = (FieldId, String, DataType, FieldKind, String);

fn extract_field_attrs(e: &quick_xml::events::BytesStart<'_>) -> Result<FieldAttrs, ParseError> {
    let id = get_attr(e, b"id")
        .and_then(|v| {
            v.parse::<u64>()
                .map_err(|_| ParseError::InvalidValue(format!("Field id: {v}")))
        })
        .unwrap_or(0);
    let name = get_attr(e, b"name").unwrap_or_default();
    let data_type = DataType::parse_xml(&get_attr(e, b"dataType").unwrap_or_default());
    let field_type = FieldKind::parse_xml(&get_attr(e, b"fieldType").unwrap_or_default());
    let comment = get_attr(e, b"comment").unwrap_or_default();
    Ok((FieldId(id), name, data_type, field_type, comment))
}

/// Build a `Field` from pre-extracted attrs, then consume children.
fn parse_field_children<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    attrs: FieldAttrs,
) -> Result<Field, ParseError> {
    let (id, name, data_type, field_type, comment) = attrs;
    let mut is_global = false;
    let mut max_repeat: u32 = 1;
    let mut calculation: Option<String> = None;
    let mut auto_enter_type = String::new();
    let mut auto_enter_calc: Option<String> = None;
    let mut auto_enter_allow_editing = true;
    let mut index_type = String::new();
    // Validation
    let mut val_not_empty = false;
    let mut val_unique = false;
    let mut val_existing = false;
    let mut val_max_length: Option<i64> = None;
    let mut val_value_list: Option<String> = None;
    let mut val_calc: Option<String> = None;
    let mut val_range_from: Option<String> = None;
    let mut val_range_to: Option<String> = None;
    let mut val_always = false;
    let mut val_error_message: Option<String> = None;

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"AutoEnter" => {
                let (ae_type_attr, allow) = extract_auto_enter_attrs(e);
                auto_enter_allow_editing = allow;
                let (type_override, val) = parse_auto_enter_children(reader, buf)?;
                auto_enter_type = type_override.unwrap_or(ae_type_attr);
                auto_enter_calc = val;
            }
            Event::Empty(ref e) if e.name().as_ref() == b"AutoEnter" => {
                let (ae_type_attr, allow) = extract_auto_enter_attrs(e);
                auto_enter_type = ae_type_attr;
                auto_enter_allow_editing = allow;
            }
            Event::Start(ref e) if e.name().as_ref() == b"Validation" => {
                let is_always = get_attr(e, b"type")
                    .map(|v| v.eq_ignore_ascii_case("Always"))
                    .unwrap_or(false);
                let vinfo = parse_validation(reader, buf, is_always)?;
                val_not_empty = vinfo.val_not_empty;
                val_unique = vinfo.val_unique;
                val_existing = vinfo.val_existing;
                val_max_length = vinfo.val_max_length;
                val_value_list = vinfo.val_value_list;
                val_calc = vinfo.val_calc;
                val_range_from = vinfo.val_range_from;
                val_range_to = vinfo.val_range_to;
                val_always = vinfo.val_always;
                val_error_message = vinfo.val_error_message;
            }
            Event::Empty(ref e) if e.name().as_ref() == b"Validation" => {
                // 属性のみの空要素（type のみ取得）
                val_always = get_attr(e, b"type")
                    .map(|v| v.eq_ignore_ascii_case("Always"))
                    .unwrap_or(false);
            }
            Event::Start(ref e) if e.name().as_ref() == b"Storage" => {
                extract_storage_attrs(e, &mut is_global, &mut max_repeat, &mut index_type);
                skip_element(reader, buf)?;
            }
            Event::Empty(ref e) if e.name().as_ref() == b"Storage" => {
                extract_storage_attrs(e, &mut is_global, &mut max_repeat, &mut index_type);
            }
            Event::Start(ref e) if e.name().as_ref() == b"Calculation" => {
                calculation = Some(read_text_content(reader, buf)?);
            }
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </Field>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok(Field {
        id,
        name,
        data_type,
        field_type,
        comment,
        is_global,
        max_repeat,
        calculation,
        auto_enter_type,
        auto_enter_calc,
        auto_enter_allow_editing,
        val_not_empty,
        val_unique,
        val_existing,
        val_max_length,
        val_value_list,
        val_calc,
        val_range_from,
        val_range_to,
        val_always,
        val_error_message,
        index_type,
    })
}

/// `<Storage>` 属性から global / max_repeat / index_type を更新する。
fn extract_storage_attrs(
    e: &quick_xml::events::BytesStart<'_>,
    is_global: &mut bool,
    max_repeat: &mut u32,
    index_type: &mut String,
) {
    if let Ok(g) = get_attr(e, b"global") {
        *is_global = g.eq_ignore_ascii_case("true");
    }
    let mr_str = get_attr(e, b"maxRepetition")
        .or_else(|_| get_attr(e, b"maxRepeat"))
        .unwrap_or_default();
    if let Ok(n) = mr_str.parse::<u32>() {
        *max_repeat = n;
    }
    *index_type = get_attr(e, b"index").unwrap_or_default();
}

/// `<AutoEnter>` の属性から (auto_enter_type, allow_editing) を返す。
fn extract_auto_enter_attrs(e: &quick_xml::events::BytesStart<'_>) -> (String, bool) {
    let allow_editing = get_attr(e, b"allowEditing")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(true);

    let ae_type = if get_attr(e, b"calculation")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        "Calculation".to_owned()
    } else if get_attr(e, b"constant")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        "ConstantData".to_owned()
    } else if get_attr(e, b"lookup")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        "Lookup".to_owned()
    } else {
        // value 属性があればその値（例: "ModificationTimeStamp"）
        // Serial は子要素 <Serial> で検出するためここには出現しない
        get_attr(e, b"value").unwrap_or_default()
    };

    (ae_type, allow_editing)
}

/// `<AutoEnter>` の子要素を消費し、`(type_override, value)` を返す。
///
/// - `<Serial>` 子要素が見つかれば type_override = Some("Serial")、value にシリアル情報文字列
/// - `<Calculation>` があれば value に計算式
/// - `<ConstantData>text</ConstantData>` があれば value に定数値テキスト
fn parse_auto_enter_children<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<(Option<String>, Option<String>), ParseError> {
    let mut type_override: Option<String> = None;
    let mut value: Option<String> = None;
    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            // <Serial increment="1" nextValue="16" generate="OnCreation"/>
            Event::Empty(ref e) if e.name().as_ref() == b"Serial" => {
                let next = get_attr(e, b"nextValue").unwrap_or_default();
                let inc = get_attr(e, b"increment").unwrap_or_default();
                let gen = get_attr(e, b"generate").unwrap_or_default();
                type_override = Some("Serial".to_owned());
                value = Some(format!("nextValue={next},increment={inc},generate={gen}"));
            }
            Event::Start(ref e) if e.name().as_ref() == b"Calculation" => {
                value = Some(read_text_content(reader, buf)?);
            }
            // <ConstantData>text</ConstantData>
            Event::Start(ref e) if e.name().as_ref() == b"ConstantData" => {
                let text = read_text_content(reader, buf)?;
                if !text.is_empty() {
                    value = Some(text);
                }
            }
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </AutoEnter>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }
    Ok((type_override, value))
}

/// `<Validation>` 要素（開始タグ消費済み）の子要素をパースして ValidationInfo を返す。
struct ValidationInfo {
    val_not_empty: bool,
    val_unique: bool,
    val_existing: bool,
    val_max_length: Option<i64>,
    val_value_list: Option<String>,
    val_calc: Option<String>,
    val_range_from: Option<String>,
    val_range_to: Option<String>,
    val_always: bool,
    val_error_message: Option<String>,
}

fn parse_validation<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    is_always: bool,
) -> Result<ValidationInfo, ParseError> {
    let val_always = is_always;
    let mut info = ValidationInfo {
        val_not_empty: false,
        val_unique: false,
        val_existing: false,
        val_max_length: None,
        val_value_list: None,
        val_calc: None,
        val_range_from: None,
        val_range_to: None,
        val_always,
        val_error_message: None,
    };
    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Empty(ref e) if e.name().as_ref() == b"NotEmpty" => {
                info.val_not_empty = get_attr(e, b"value")
                    .map(|v| v.eq_ignore_ascii_case("true"))
                    .unwrap_or(false);
            }
            Event::Empty(ref e) if e.name().as_ref() == b"Unique" => {
                info.val_unique = get_attr(e, b"value")
                    .map(|v| v.eq_ignore_ascii_case("true"))
                    .unwrap_or(false);
            }
            Event::Empty(ref e) if e.name().as_ref() == b"Existing" => {
                info.val_existing = get_attr(e, b"value")
                    .map(|v| v.eq_ignore_ascii_case("true"))
                    .unwrap_or(false);
            }
            Event::Empty(ref e) if e.name().as_ref() == b"Range" => {
                info.val_range_from = get_attr(e, b"from").ok();
                info.val_range_to = get_attr(e, b"to").ok();
            }
            Event::Empty(ref e) if e.name().as_ref() == b"ValueList" => {
                info.val_value_list = get_attr(e, b"name").ok();
            }
            Event::Start(ref e) if e.name().as_ref() == b"Calculation" => {
                info.val_calc = Some(read_text_content(reader, buf)?);
            }
            Event::Start(ref e) if e.name().as_ref() == b"MaxLength" => {
                if let Ok(n) = read_text_content(reader, buf)?.trim().parse::<i64>() {
                    info.val_max_length = Some(n);
                }
            }
            Event::Start(ref e) if e.name().as_ref() == b"ErrorMessage" => {
                let msg = read_text_content(reader, buf)?;
                if !msg.is_empty() {
                    info.val_error_message = Some(msg);
                }
            }
            Event::Start(_) => skip_element(reader, buf)?,
            Event::Empty(_) => {}
            Event::End(_) => break, // </Validation>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }
    Ok(info)
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
        let buf = Vec::new();
        (reader, buf)
    }

    fn parse(xml: &str) -> Result<Vec<Table>, ParseError> {
        let full = format!("<BaseTableCatalog>{xml}</BaseTableCatalog>");
        let (mut reader, mut buf) = make_reader(&full);
        // consume opening <BaseTableCatalog>
        loop {
            buf.clear();
            match reader.read_event_into(&mut buf).unwrap() {
                Event::Start(ref e) if e.name().as_ref() == b"BaseTableCatalog" => break,
                _ => {}
            }
        }
        parse_tables(&mut reader, &mut buf)
    }

    #[test]
    fn empty_catalog_returns_empty_vec() {
        let tables = parse("").unwrap();
        assert!(tables.is_empty());
    }

    #[test]
    fn single_table_no_fields() {
        let tables = parse(r#"<BaseTable id="1" name="Contact" color="white"/>"#).unwrap();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].name, "Contact");
        assert_eq!(tables[0].id, TableId(1));
        assert!(tables[0].fields.is_empty());
    }

    #[test]
    fn table_with_two_fields() {
        let xml = r#"
        <BaseTable id="1" name="Contact" color="white">
          <FieldCatalog>
            <Field id="1" dataType="Text" fieldType="Normal" name="FirstName" comment="">
              <AutoEnter allowEditing="True" constant="" lookup="False" serial="False"/>
              <Validation maxLength="False" notEmpty="False" unique="False"/>
              <Storage global="False" indexLanguage="English" maxRepeat="1"/>
            </Field>
            <Field id="2" dataType="Number" fieldType="Normal" name="Age" comment="">
              <Storage global="True" indexLanguage="English" maxRepeat="2"/>
            </Field>
          </FieldCatalog>
        </BaseTable>
        "#;
        let tables = parse(xml).unwrap();
        assert_eq!(tables.len(), 1);
        let fields = &tables[0].fields;
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name, "FirstName");
        assert_eq!(fields[0].data_type, DataType::Text);
        assert!(!fields[0].is_global);
        assert_eq!(fields[0].max_repeat, 1);
        assert_eq!(fields[1].name, "Age");
        assert!(fields[1].is_global);
        assert_eq!(fields[1].max_repeat, 2);
    }

    #[rstest]
    #[case("Text", DataType::Text)]
    #[case("Number", DataType::Number)]
    #[case("Date", DataType::Date)]
    #[case("Container", DataType::Container)]
    fn field_data_types(#[case] type_str: &str, #[case] expected: DataType) {
        let xml = format!(
            "<BaseTable id=\"1\" name=\"T\"><FieldCatalog>\
              <Field id=\"1\" dataType=\"{type_str}\" fieldType=\"Normal\" name=\"F\" comment=\"\"/>\
            </FieldCatalog></BaseTable>"
        );
        let tables = parse(&xml).unwrap();
        assert_eq!(tables[0].fields[0].data_type, expected);
    }

    #[test]
    fn calculated_field_has_calculation() {
        let xml = r#"
        <BaseTable id="1" name="T">
          <FieldCatalog>
            <Field id="1" dataType="Text" fieldType="Calculated" name="FullName" comment="">
              <Calculation>FirstName</Calculation>
              <Storage global="False" indexLanguage="English" maxRepeat="1"/>
            </Field>
          </FieldCatalog>
        </BaseTable>
        "#;
        let tables = parse(xml).unwrap();
        let field = &tables[0].fields[0];
        assert_eq!(field.field_type, FieldKind::Calculated);
        assert!(field.calculation.is_some());
    }

    #[test]
    fn storage_max_repetition_real_ddr_format() {
        // 実DDR は maxRepetition 属性を使用する
        let xml = r#"
        <BaseTable id="1" name="T">
          <FieldCatalog>
            <Field id="1" dataType="Number" fieldType="Normal" name="Rep">
              <Storage global="False" indexLanguage="Japanese" maxRepetition="5"/>
            </Field>
          </FieldCatalog>
        </BaseTable>
        "#;
        let tables = parse(xml).unwrap();
        assert_eq!(tables[0].fields[0].max_repeat, 5);
    }

    #[test]
    fn multiple_tables() {
        let xml = r#"
        <BaseTable id="1" name="Contact"><FieldCatalog/></BaseTable>
        <BaseTable id="2" name="Project"><FieldCatalog/></BaseTable>
        "#;
        let tables = parse(xml).unwrap();
        assert_eq!(tables.len(), 2);
        assert_eq!(tables[0].name, "Contact");
        assert_eq!(tables[1].name, "Project");
    }

    #[test]
    fn auto_enter_serial_detected_from_child_element() {
        let xml = r#"
        <BaseTable id="1" name="T"><FieldCatalog>
          <Field id="1" dataType="Number" fieldType="Normal" name="Seq" comment="">
            <AutoEnter allowEditing="True" constant="False" furigana="False" lookup="False" calculation="False">
              <Serial increment="1" nextValue="16" generate="OnCreation"/>
              <ConstantData/>
            </AutoEnter>
            <Storage global="False" indexLanguage="Japanese" maxRepetition="1"/>
          </Field>
        </FieldCatalog></BaseTable>
        "#;
        let tables = parse(xml).unwrap();
        let field = &tables[0].fields[0];
        assert_eq!(field.auto_enter_type, "Serial");
        assert!(field.auto_enter_allow_editing);
        let calc = field.auto_enter_calc.as_deref().unwrap_or("");
        assert!(calc.contains("nextValue=16"), "calc = {calc}");
        assert!(calc.contains("increment=1"), "calc = {calc}");
        assert!(calc.contains("generate=OnCreation"), "calc = {calc}");
    }

    #[test]
    fn auto_enter_constant_value_extracted() {
        let xml = r#"
        <BaseTable id="1" name="T"><FieldCatalog>
          <Field id="1" dataType="Number" fieldType="Normal" name="Flag" comment="">
            <AutoEnter allowEditing="False" value="ConstantData" constant="True"
                       furigana="False" lookup="False" calculation="False">
              <ConstantData>1</ConstantData>
            </AutoEnter>
            <Storage global="False" maxRepetition="1"/>
          </Field>
        </FieldCatalog></BaseTable>
        "#;
        let tables = parse(xml).unwrap();
        let field = &tables[0].fields[0];
        assert_eq!(field.auto_enter_type, "ConstantData");
        assert_eq!(field.auto_enter_calc.as_deref(), Some("1"));
        assert!(!field.auto_enter_allow_editing);
    }

    #[test]
    fn validation_not_empty_and_unique() {
        let xml = r#"
        <BaseTable id="1" name="T"><FieldCatalog>
          <Field id="1" dataType="Text" fieldType="Normal" name="Name" comment="">
            <Validation message="False" maxLength="False" valuelist="False"
                        calculation="False" alwaysValidateCalculation="False"
                        type="OnlyDuringDataEntry">
              <NotEmpty value="True"/>
              <Unique value="True"/>
              <Existing value="False"/>
              <StrictValidation value="False"/>
            </Validation>
            <Storage global="False" maxRepetition="1"/>
          </Field>
        </FieldCatalog></BaseTable>
        "#;
        let tables = parse(xml).unwrap();
        let field = &tables[0].fields[0];
        assert!(field.val_not_empty);
        assert!(field.val_unique);
        assert!(!field.val_existing);
        assert!(!field.val_always);
    }

    #[test]
    fn validation_range_extracted() {
        let xml = r#"
        <BaseTable id="1" name="T"><FieldCatalog>
          <Field id="1" dataType="Number" fieldType="Normal" name="Score" comment="">
            <Validation message="True" maxLength="False" valuelist="False"
                        calculation="False" type="Always">
              <NotEmpty value="False"/>
              <Unique value="False"/>
              <Existing value="False"/>
              <Range from="4" to="98"/>
              <StrictValidation value="False"/>
              <ErrorMessage>4から98の範囲で入力してください。</ErrorMessage>
            </Validation>
            <Storage global="False" maxRepetition="1"/>
          </Field>
        </FieldCatalog></BaseTable>
        "#;
        let tables = parse(xml).unwrap();
        let field = &tables[0].fields[0];
        assert_eq!(field.val_range_from.as_deref(), Some("4"));
        assert_eq!(field.val_range_to.as_deref(), Some("98"));
        assert!(field.val_always);
        assert!(field.val_error_message.is_some());
    }

    #[test]
    fn storage_index_type_extracted() {
        let xml = r#"
        <BaseTable id="1" name="T"><FieldCatalog>
          <Field id="1" dataType="Text" fieldType="Normal" name="Code" comment="">
            <Storage index="All" indexLanguage="Japanese" global="False" maxRepetition="1"/>
          </Field>
        </FieldCatalog></BaseTable>
        "#;
        let tables = parse(xml).unwrap();
        let field = &tables[0].fields[0];
        assert_eq!(field.index_type, "All");
    }
}
