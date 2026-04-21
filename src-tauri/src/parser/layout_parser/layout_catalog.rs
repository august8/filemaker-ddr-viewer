use std::io::BufRead;

use quick_xml::{events::Event, Reader};

use crate::parser::{
    helpers::{get_attr, skip_element},
    models::{Layout, LayoutFieldRef, LayoutId, LayoutObject, ScriptTrigger},
    ParseError,
};

use super::{
    object_scanner::{scan_object, scan_object_list},
    script_and_format::parse_script_trigger_elements,
};

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
