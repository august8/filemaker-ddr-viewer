use std::io::BufRead;

use quick_xml::{events::Event, Reader};

use crate::parser::{
    helpers::{get_attr, read_text_content, skip_element},
    models::{Script, ScriptId, ScriptRef, ScriptStep},
    ParseError,
};

/// Parse `<ScriptCatalog>` content.
///
/// The caller must have already consumed the opening `<ScriptCatalog>` tag.
pub fn parse_scripts<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Vec<Script>, ParseError> {
    let mut scripts = Vec::new();
    parse_scripts_in_container(reader, buf, &mut scripts)?;
    Ok(scripts)
}

/// 再帰的にスクリプトを収集する内部関数。
///
/// `<ScriptCatalog>` および `<Group>` 内の `<Script>` / `<Group>` を処理する。
fn parse_scripts_in_container<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    scripts: &mut Vec<Script>,
) -> Result<(), ParseError> {
    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"Script" => {
                let id = get_attr(e, b"id")
                    .and_then(|v| {
                        v.parse::<u64>()
                            .map_err(|_| ParseError::InvalidValue(format!("Script id: {v}")))
                    })
                    .unwrap_or(0);
                let name = get_attr(e, b"name").unwrap_or_default();
                // 実DDR: runWithFullAccessPrivileges / 中間形式: runFullAccess / 旧フィクスチャ: runWithFullAccess の順で試みる
                let run_with_full_access = get_attr(e, b"runWithFullAccessPrivileges")
                    .or_else(|_| get_attr(e, b"runFullAccess"))
                    .or_else(|_| get_attr(e, b"runWithFullAccess"))
                    .map(|v| v.eq_ignore_ascii_case("true"))
                    .unwrap_or(false);
                let steps = parse_script_children(reader, buf)?;
                scripts.push(Script {
                    id: ScriptId(id),
                    name,
                    run_with_full_access,
                    steps,
                });
            }
            Event::Empty(ref e) if e.name().as_ref() == b"Script" => {
                let id = get_attr(e, b"id")
                    .and_then(|v| {
                        v.parse::<u64>()
                            .map_err(|_| ParseError::InvalidValue(format!("Script id: {v}")))
                    })
                    .unwrap_or(0);
                let name = get_attr(e, b"name").unwrap_or_default();
                let run_with_full_access = get_attr(e, b"runWithFullAccessPrivileges")
                    .or_else(|_| get_attr(e, b"runFullAccess"))
                    .or_else(|_| get_attr(e, b"runWithFullAccess"))
                    .map(|v| v.eq_ignore_ascii_case("true"))
                    .unwrap_or(false);
                scripts.push(Script {
                    id: ScriptId(id),
                    name,
                    run_with_full_access,
                    steps: Vec::new(),
                });
            }
            // 実DDR: <Group> でスクリプトをグループ化している → 再帰
            Event::Start(ref e) if e.name().as_ref() == b"Group" => {
                parse_scripts_in_container(reader, buf, scripts)?;
            }
            Event::Empty(ref e) if e.name().as_ref() == b"Group" => {
                // 空グループ
            }
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </ScriptCatalog> or </Group>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }
    Ok(())
}

/// Parse children of a `<Script>` element (opening tag already consumed).
fn parse_script_children<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Vec<ScriptStep>, ParseError> {
    let mut steps = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"StepList" => {
                steps = parse_step_list(reader, buf)?;
            }
            Event::Empty(ref e) if e.name().as_ref() == b"StepList" => {
                // Empty step list
            }
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </Script>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok(steps)
}

/// Parse the content of a `<StepList>` element (already consumed).
fn parse_step_list<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Vec<ScriptStep>, ParseError> {
    let mut steps = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"Step" => {
                let step_id = get_attr(e, b"id")
                    .and_then(|v| {
                        v.parse::<u32>()
                            .map_err(|_| ParseError::InvalidValue(format!("Step id: {v}")))
                    })
                    .unwrap_or(0);
                let name = get_attr(e, b"name").unwrap_or_default();
                let enabled = get_attr(e, b"enable")
                    .map(|v| v.eq_ignore_ascii_case("true"))
                    .unwrap_or(true);
                let step = parse_step_children(reader, buf, step_id, name, enabled)?;
                steps.push(step);
            }
            Event::Empty(ref e) if e.name().as_ref() == b"Step" => {
                let step_id = get_attr(e, b"id")
                    .and_then(|v| {
                        v.parse::<u32>()
                            .map_err(|_| ParseError::InvalidValue(format!("Step id: {v}")))
                    })
                    .unwrap_or(0);
                let name = get_attr(e, b"name").unwrap_or_default();
                let enabled = get_attr(e, b"enable")
                    .map(|v| v.eq_ignore_ascii_case("true"))
                    .unwrap_or(true);
                steps.push(ScriptStep {
                    step_id,
                    name,
                    enabled,
                    script_ref: None,
                    calculation: None,
                    step_text: None,
                });
            }
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </StepList>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok(steps)
}

/// Parse children of a `<Step>` element (opening tag already consumed).
fn parse_step_children<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    step_id: u32,
    name: String,
    enabled: bool,
) -> Result<ScriptStep, ParseError> {
    let mut script_ref: Option<ScriptRef> = None;
    let mut calculation: Option<String> = None;
    let mut step_text: Option<String> = None;

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            // <StepText> — 実DDR形式: 人間可読な表示テキスト
            Event::Start(ref e) if e.name().as_ref() == b"StepText" => {
                step_text = Some(read_text_content(reader, buf)?);
            }
            // <Script .../> — Perform Script reference (self-closing, 旧形式)
            Event::Empty(ref e) if e.name().as_ref() == b"Script" => {
                let ref_name = get_attr(e, b"name").unwrap_or_default();
                let file_name = get_attr(e, b"file").unwrap_or_default();
                script_ref = Some(ScriptRef {
                    name: ref_name,
                    file_name,
                });
            }
            // <Script ...>...</Script> — Perform Script reference (with children, 旧形式)
            Event::Start(ref e) if e.name().as_ref() == b"Script" => {
                let ref_name = get_attr(e, b"name").unwrap_or_default();
                let file_name = get_attr(e, b"file").unwrap_or_default();
                script_ref = Some(ScriptRef {
                    name: ref_name,
                    file_name,
                });
                skip_element(reader, buf)?;
            }
            // <ScriptReference .../> — 実DDR形式 (self-closing)
            Event::Empty(ref e) if e.name().as_ref() == b"ScriptReference" => {
                let ref_name = get_attr(e, b"name").unwrap_or_default();
                let file_name = get_attr(e, b"file").unwrap_or_default();
                script_ref = Some(ScriptRef {
                    name: ref_name,
                    file_name,
                });
            }
            // <ScriptReference ...>...</ScriptReference> — 実DDR形式 (with children)
            Event::Start(ref e) if e.name().as_ref() == b"ScriptReference" => {
                let ref_name = get_attr(e, b"name").unwrap_or_default();
                let file_name = get_attr(e, b"file").unwrap_or_default();
                script_ref = Some(ScriptRef {
                    name: ref_name,
                    file_name,
                });
                skip_element(reader, buf)?;
            }
            // <Calculation> text/CDATA content (旧形式)
            Event::Start(ref e) if e.name().as_ref() == b"Calculation" => {
                calculation = Some(read_text_content(reader, buf)?);
            }
            // <DisplayCalculation> text/CDATA content (実DDR形式)
            Event::Start(ref e) if e.name().as_ref() == b"DisplayCalculation" => {
                if calculation.is_none() {
                    calculation = Some(read_text_content(reader, buf)?);
                } else {
                    skip_element(reader, buf)?;
                }
            }
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </Step>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok(ScriptStep {
        step_id,
        name,
        enabled,
        script_ref,
        calculation,
        step_text,
    })
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

    fn parse(xml: &str) -> Result<Vec<Script>, ParseError> {
        let full = format!("<ScriptCatalog>{xml}</ScriptCatalog>");
        let (mut reader, mut buf) = make_reader(&full);
        loop {
            buf.clear();
            match reader.read_event_into(&mut buf).unwrap() {
                Event::Start(ref e) if e.name().as_ref() == b"ScriptCatalog" => break,
                _ => {}
            }
        }
        parse_scripts(&mut reader, &mut buf)
    }

    #[test]
    fn empty_catalog() {
        let scripts = parse("").unwrap();
        assert!(scripts.is_empty());
    }

    #[test]
    fn single_script_no_steps() {
        // 実DDR 形式: runFullAccess 属性
        let scripts = parse(
            r#"<Script id="1" name="Hello World" runFullAccess="False" includeInMenu="False"/>"#,
        )
        .unwrap();
        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0].name, "Hello World");
        assert_eq!(scripts[0].id, ScriptId(1));
        assert!(!scripts[0].run_with_full_access);
        assert!(scripts[0].steps.is_empty());
    }

    #[test]
    fn single_script_legacy_attribute() {
        // 旧フィクスチャ形式との後方互換性: runWithFullAccess 属性
        let scripts = parse(
            r#"<Script id="1" name="Hello World" runWithFullAccess="False" includeInMenu="False"/>"#,
        )
        .unwrap();
        assert_eq!(scripts.len(), 1);
        assert!(!scripts[0].run_with_full_access);
    }

    #[test]
    fn script_with_perform_script_step() {
        let xml = r#"
        <Script id="1" name="Main" runFullAccess="False" includeInMenu="False">
          <StepList>
            <Step id="1" name="Perform Script" enable="True">
              <Script id="2" name="Another Script"/>
            </Step>
            <Step id="3" name="Show Custom Dialog" enable="True"/>
          </StepList>
        </Script>
        "#;
        let scripts = parse(xml).unwrap();
        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0].steps.len(), 2);

        let step0 = &scripts[0].steps[0];
        assert_eq!(step0.step_id, 1);
        assert_eq!(step0.name, "Perform Script");
        assert!(step0.enabled);
        assert!(step0.script_ref.is_some());
        let sref = step0.script_ref.as_ref().unwrap();
        assert_eq!(sref.name, "Another Script");

        let step1 = &scripts[0].steps[1];
        assert_eq!(step1.name, "Show Custom Dialog");
        assert!(step1.script_ref.is_none());
    }

    #[test]
    fn script_with_full_access_flag() {
        let xml = r#"
        <Script id="2" name="Admin Script" runFullAccess="True" includeInMenu="False">
          <StepList/>
        </Script>
        "#;
        let scripts = parse(xml).unwrap();
        assert!(scripts[0].run_with_full_access);
    }

    #[test]
    fn scripts_inside_group_are_collected() {
        // 実DDR: <Group> 内のスクリプトも収集されること
        let xml = r#"
        <Group groupCollapsed="False" includeInMenu="False" id="1" name="Dev">
          <Script id="5" name="Script A" runFullAccess="False" includeInMenu="True">
            <StepList/>
          </Script>
          <Script id="6" name="Script B" runFullAccess="False" includeInMenu="False"/>
        </Group>
        <Script id="7" name="Top Level" runFullAccess="False" includeInMenu="True"/>
        "#;
        let scripts = parse(xml).unwrap();
        assert_eq!(scripts.len(), 3);
        assert_eq!(scripts[0].name, "Script A");
        assert_eq!(scripts[1].name, "Script B");
        assert_eq!(scripts[2].name, "Top Level");
    }

    #[test]
    fn nested_groups_are_flattened() {
        let xml = r#"
        <Group id="1" name="Outer">
          <Group id="2" name="Inner">
            <Script id="1" name="Deep Script" runFullAccess="False"/>
          </Group>
        </Group>
        "#;
        let scripts = parse(xml).unwrap();
        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0].name, "Deep Script");
    }

    #[rstest]
    #[case(3)]
    #[case(0)]
    fn script_count(#[case] count: usize) {
        let inner: String = (1..=count)
            .map(|i| format!(r#"<Script id="{i}" name="S{i}"/>"#))
            .collect();
        assert_eq!(parse(&inner).unwrap().len(), count);
    }

    #[test]
    fn disabled_step() {
        let xml = r#"
        <Script id="1" name="S" runFullAccess="False" includeInMenu="False">
          <StepList>
            <Step id="5" name="Comment" enable="False"/>
          </StepList>
        </Script>
        "#;
        let scripts = parse(xml).unwrap();
        assert!(!scripts[0].steps[0].enabled);
    }

    // 実DDR形式: runWithFullAccessPrivileges 属性
    #[test]
    fn real_ddr_run_with_full_access_privileges() {
        let scripts = parse(
            r#"<Script id="3" name="監査ログ記録" includeInMenu="False" runWithFullAccessPrivileges="True"/>"#,
        )
        .unwrap();
        assert!(scripts[0].run_with_full_access);
    }

    // 実DDR形式: <ScriptReference> + <DisplayCalculation>
    #[test]
    fn real_ddr_script_reference_and_display_calculation() {
        let xml = r#"
        <Script id="1" name="顧客登録_メイン" includeInMenu="True" runWithFullAccessPrivileges="False">
          <StepList>
            <Step id="2" enable="True" name="スクリプト実行">
              <StepText>スクリプト実行 ["入力バリデーション"]</StepText>
              <ScriptReference name="入力バリデーション" id="2" />
            </Step>
            <Step id="1" enable="True" name="If">
              <StepText>If [IsEmpty(顧客::顧客名)]</StepText>
              <DisplayCalculation><![CDATA[IsEmpty(顧客::顧客名)]]></DisplayCalculation>
            </Step>
          </StepList>
        </Script>
        "#;
        let scripts = parse(xml).unwrap();
        assert_eq!(scripts.len(), 1);
        assert!(!scripts[0].run_with_full_access);
        assert_eq!(scripts[0].steps.len(), 2);

        let step0 = &scripts[0].steps[0];
        assert_eq!(step0.name, "スクリプト実行");
        let sref = step0.script_ref.as_ref().unwrap();
        assert_eq!(sref.name, "入力バリデーション");

        let step1 = &scripts[0].steps[1];
        assert_eq!(step1.name, "If");
        assert_eq!(step1.calculation.as_deref(), Some("IsEmpty(顧客::顧客名)"));
    }
}
