use std::io::BufRead;

use quick_xml::{events::Event, Reader};

use crate::parser::{
    helpers::{get_attr, read_text_content, skip_element},
    models::{
        Account, AccountId, CustomFunction, CustomFunctionId, PrivilegeSet, PrivilegeSetId,
        ValueList, ValueListFieldRef, ValueListId, ValueListSource,
    },
    ParseError,
};

// ---------------------------------------------------------------------------
// Value Lists
// ---------------------------------------------------------------------------

/// Parse `<ValueListCatalog>` content (opening tag already consumed).
pub fn parse_value_lists<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Vec<ValueList>, ParseError> {
    let mut value_lists = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"ValueList" => {
                let id = get_attr(e, b"id")
                    .and_then(|v| {
                        v.parse::<u64>()
                            .map_err(|_| ParseError::InvalidValue(format!("ValueList id: {v}")))
                    })
                    .unwrap_or(0);
                let name = get_attr(e, b"name").unwrap_or_default();
                let (source, custom_values, field_refs) = parse_value_list_children(reader, buf)?;
                value_lists.push(ValueList {
                    id: ValueListId(id),
                    name,
                    source,
                    custom_values,
                    field_refs,
                });
            }
            Event::Empty(ref e) if e.name().as_ref() == b"ValueList" => {
                let id = get_attr(e, b"id")
                    .and_then(|v| {
                        v.parse::<u64>()
                            .map_err(|_| ParseError::InvalidValue(format!("ValueList id: {v}")))
                    })
                    .unwrap_or(0);
                let name = get_attr(e, b"name").unwrap_or_default();
                value_lists.push(ValueList {
                    id: ValueListId(id),
                    name,
                    source: ValueListSource::Unknown(String::new()),
                    custom_values: Vec::new(),
                    field_refs: Vec::new(),
                });
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

    Ok(value_lists)
}

/// Parse children of a `<ValueList>` element (opening tag already consumed).
///
/// 実DDR 形式:
/// ```xml
/// <ValueList id="7" name="Status">
///   <Source value="Custom"/>        ← 子要素として source
///   <CustomValues>
///     <Text>Active</Text>           ← <Text> タグ
///   </CustomValues>
/// </ValueList>
/// ```
fn parse_value_list_children<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<(ValueListSource, Vec<String>, Vec<ValueListFieldRef>), ParseError> {
    let mut source = ValueListSource::Unknown(String::new());
    let mut custom_values = Vec::new();
    let mut field_refs = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            // 実DDR: <Source value="Custom"/> または <Source value="Field"/>
            Event::Empty(ref e) if e.name().as_ref() == b"Source" => {
                source = ValueListSource::parse_xml(&get_attr(e, b"value").unwrap_or_default());
            }
            Event::Start(ref e) if e.name().as_ref() == b"CustomValues" => {
                custom_values = parse_custom_values(reader, buf)?;
            }
            Event::Empty(ref e) if e.name().as_ref() == b"CustomValues" => {}
            // フィールドソース: <PrimaryField> / <SecondaryField>
            Event::Start(ref e)
                if e.name().as_ref() == b"PrimaryField"
                    || e.name().as_ref() == b"SecondaryField" =>
            {
                if let Some(field_ref) = parse_value_list_field_ref(reader, buf)? {
                    field_refs.push(field_ref);
                }
            }
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </ValueList>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok((source, custom_values, field_refs))
}

/// `<PrimaryField>` または `<SecondaryField>` 内の `<Field table=".." name=".."/>` をパースする。
fn parse_value_list_field_ref<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Option<ValueListFieldRef>, ParseError> {
    let mut result = None;

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Empty(ref e) if e.name().as_ref() == b"Field" => {
                let table_occurrence = get_attr(e, b"table").unwrap_or_default();
                let field_name = get_attr(e, b"name").unwrap_or_default();
                if !table_occurrence.is_empty() && !field_name.is_empty() {
                    result = Some(ValueListFieldRef {
                        table_occurrence,
                        field_name,
                    });
                }
            }
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </PrimaryField> or </SecondaryField>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok(result)
}

/// Parse `<CustomValues>` content (opening tag already consumed).
///
/// 実DDR 形式は `<Text>` タグ、旧フィクスチャは `<Value>` タグ（両方許容）。
fn parse_custom_values<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Vec<String>, ParseError> {
    let mut values = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            // 実DDR: <Text>Active</Text>
            Event::Start(ref e) if e.name().as_ref() == b"Text" => {
                let text = read_text_content(reader, buf)?;
                values.push(text);
            }
            Event::Empty(ref e) if e.name().as_ref() == b"Text" => {
                values.push(String::new());
            }
            // 旧フィクスチャ形式との後方互換: <Value>Active</Value>
            Event::Start(ref e) if e.name().as_ref() == b"Value" => {
                let text = read_text_content(reader, buf)?;
                values.push(text);
            }
            Event::Empty(ref e) if e.name().as_ref() == b"Value" => {
                values.push(String::new());
            }
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </CustomValues>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok(values)
}

// ---------------------------------------------------------------------------
// Custom Functions
// ---------------------------------------------------------------------------

/// Parse `<CustomFunctionCatalog>` content (opening tag already consumed).
pub fn parse_custom_functions<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Vec<CustomFunction>, ParseError> {
    let mut functions = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"CustomFunction" => {
                let id = get_attr(e, b"id")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0);
                let name = get_attr(e, b"name").unwrap_or_default();
                let parameters = parse_parameters(&get_attr(e, b"parameters").unwrap_or_default());
                let calculation = parse_custom_function_body(reader, buf)?;
                functions.push(CustomFunction {
                    id: CustomFunctionId(id),
                    name,
                    parameters,
                    calculation,
                });
            }
            Event::Empty(ref e) if e.name().as_ref() == b"CustomFunction" => {
                let id = get_attr(e, b"id")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0);
                let name = get_attr(e, b"name").unwrap_or_default();
                let parameters = parse_parameters(&get_attr(e, b"parameters").unwrap_or_default());
                functions.push(CustomFunction {
                    id: CustomFunctionId(id),
                    name,
                    parameters,
                    calculation: None,
                });
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

    Ok(functions)
}

fn parse_parameters(params_str: &str) -> Vec<String> {
    if params_str.is_empty() {
        Vec::new()
    } else {
        params_str
            .split(';')
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

/// Parse children of a `<CustomFunction>` element, returning the calculation text.
fn parse_custom_function_body<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Option<String>, ParseError> {
    let mut calculation: Option<String> = None;

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"Calculation" => {
                calculation = Some(read_text_content(reader, buf)?);
            }
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </CustomFunction>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok(calculation)
}

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

/// Parse `<AccountCatalog>` content (opening tag already consumed).
pub fn parse_accounts<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Vec<Account>, ParseError> {
    let mut accounts = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"Account" => {
                let id = get_attr(e, b"id")
                    .and_then(|v| {
                        v.parse::<u64>()
                            .map_err(|_| ParseError::InvalidValue(format!("Account id: {v}")))
                    })
                    .unwrap_or(0);
                let name = get_attr(e, b"name").unwrap_or_default();
                let privilege_set = get_attr(e, b"privilegeSet").ok().filter(|s| !s.is_empty());
                let enabled = get_attr(e, b"enabled")
                    .map(|v| v.eq_ignore_ascii_case("true"))
                    .unwrap_or(true);
                skip_element(reader, buf)?;
                accounts.push(Account {
                    id: AccountId(id),
                    name,
                    privilege_set,
                    enabled,
                });
            }
            Event::Empty(ref e) if e.name().as_ref() == b"Account" => {
                let id = get_attr(e, b"id")
                    .and_then(|v| {
                        v.parse::<u64>()
                            .map_err(|_| ParseError::InvalidValue(format!("Account id: {v}")))
                    })
                    .unwrap_or(0);
                let name = get_attr(e, b"name").unwrap_or_default();
                let privilege_set = get_attr(e, b"privilegeSet").ok().filter(|s| !s.is_empty());
                let enabled = get_attr(e, b"enabled")
                    .map(|v| v.eq_ignore_ascii_case("true"))
                    .unwrap_or(true);
                accounts.push(Account {
                    id: AccountId(id),
                    name,
                    privilege_set,
                    enabled,
                });
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

    Ok(accounts)
}

// ---------------------------------------------------------------------------
// Privilege Sets
// ---------------------------------------------------------------------------

/// Parse `<PrivilegesCatalog>` content (opening tag already consumed).
pub fn parse_privilege_sets<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Vec<PrivilegeSet>, ParseError> {
    let mut privilege_sets = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"PrivilegeSet" => {
                let id = get_attr(e, b"id")
                    .and_then(|v| {
                        v.parse::<u64>()
                            .map_err(|_| ParseError::InvalidValue(format!("PrivilegeSet id: {v}")))
                    })
                    .unwrap_or(0);
                let name = get_attr(e, b"name").unwrap_or_default();
                let comment = get_attr(e, b"comment").ok().filter(|s| !s.is_empty());
                skip_element(reader, buf)?;
                privilege_sets.push(PrivilegeSet {
                    id: PrivilegeSetId(id),
                    name,
                    comment,
                });
            }
            Event::Empty(ref e) if e.name().as_ref() == b"PrivilegeSet" => {
                let id = get_attr(e, b"id")
                    .and_then(|v| {
                        v.parse::<u64>()
                            .map_err(|_| ParseError::InvalidValue(format!("PrivilegeSet id: {v}")))
                    })
                    .unwrap_or(0);
                let name = get_attr(e, b"name").unwrap_or_default();
                let comment = get_attr(e, b"comment").ok().filter(|s| !s.is_empty());
                privilege_sets.push(PrivilegeSet {
                    id: PrivilegeSetId(id),
                    name,
                    comment,
                });
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

    Ok(privilege_sets)
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

    fn consume_opening<R: BufRead>(reader: &mut Reader<R>, buf: &mut Vec<u8>, tag: &[u8]) {
        loop {
            buf.clear();
            match reader.read_event_into(buf).unwrap() {
                Event::Start(ref e) if e.name().as_ref() == tag => break,
                Event::Eof => panic!("tag not found"),
                _ => {}
            }
        }
    }

    // --- Value Lists ---

    #[test]
    fn empty_value_list_catalog() {
        let xml = "<ValueListCatalog></ValueListCatalog>";
        let (mut r, mut buf) = make_reader(xml);
        consume_opening(&mut r, &mut buf, b"ValueListCatalog");
        assert!(parse_value_lists(&mut r, &mut buf).unwrap().is_empty());
    }

    #[test]
    fn custom_value_list_real_ddr_format() {
        // 実DDR 形式: <Source value="..."/> 子要素 + <Text> タグ
        let xml = r#"<ValueListCatalog>
          <ValueList id="1" name="Status Values">
            <Source value="Custom"/>
            <CustomValues>
              <Text>Active</Text>
              <Text>Inactive</Text>
            </CustomValues>
          </ValueList>
        </ValueListCatalog>"#;
        let (mut r, mut buf) = make_reader(xml);
        consume_opening(&mut r, &mut buf, b"ValueListCatalog");
        let vls = parse_value_lists(&mut r, &mut buf).unwrap();
        assert_eq!(vls.len(), 1);
        assert_eq!(vls[0].name, "Status Values");
        assert_eq!(vls[0].source, ValueListSource::Custom);
        assert_eq!(vls[0].custom_values, vec!["Active", "Inactive"]);
    }

    #[test]
    fn custom_value_list_legacy_format() {
        // 旧フィクスチャ形式との後方互換: <Value> タグ
        let xml = r#"<ValueListCatalog>
          <ValueList id="1" name="Status Values">
            <Source value="Custom"/>
            <CustomValues>
              <Value>Active</Value>
              <Value>Inactive</Value>
            </CustomValues>
          </ValueList>
        </ValueListCatalog>"#;
        let (mut r, mut buf) = make_reader(xml);
        consume_opening(&mut r, &mut buf, b"ValueListCatalog");
        let vls = parse_value_lists(&mut r, &mut buf).unwrap();
        assert_eq!(vls[0].custom_values, vec!["Active", "Inactive"]);
    }

    #[test]
    fn field_value_list_no_primary_field() {
        // Source=Field だが <PrimaryField> がない場合、field_refs は空
        let xml = r#"<ValueListCatalog>
          <ValueList id="2" name="FK Values">
            <Source value="Field"/>
          </ValueList>
        </ValueListCatalog>"#;
        let (mut r, mut buf) = make_reader(xml);
        consume_opening(&mut r, &mut buf, b"ValueListCatalog");
        let vls = parse_value_lists(&mut r, &mut buf).unwrap();
        assert_eq!(vls[0].source, ValueListSource::Field);
        assert!(vls[0].field_refs.is_empty());
    }

    #[test]
    fn field_value_list_with_primary_field() {
        // 実DDR 形式: <PrimaryField><Field table=".." name=".."/></PrimaryField>
        let xml = r#"<ValueListCatalog>
          <ValueList id="1" name="新規値一覧">
            <Source value="Field"/>
            <PrimaryField show="True" sort="True">
              <Field table="名称未設定" id="6" name="TEST"/>
            </PrimaryField>
            <ShowRelated value="False"/>
          </ValueList>
        </ValueListCatalog>"#;
        let (mut r, mut buf) = make_reader(xml);
        consume_opening(&mut r, &mut buf, b"ValueListCatalog");
        let vls = parse_value_lists(&mut r, &mut buf).unwrap();
        assert_eq!(vls[0].source, ValueListSource::Field);
        assert_eq!(vls[0].field_refs.len(), 1);
        assert_eq!(vls[0].field_refs[0].table_occurrence, "名称未設定");
        assert_eq!(vls[0].field_refs[0].field_name, "TEST");
    }

    #[test]
    fn field_value_list_with_primary_and_secondary() {
        // Primary + Secondary の 2 フィールドを持つバリューリスト
        let xml = r#"<ValueListCatalog>
          <ValueList id="3" name="Two Fields">
            <Source value="Field"/>
            <PrimaryField show="True" sort="True">
              <Field table="Customer" id="1" name="ID"/>
            </PrimaryField>
            <SecondaryField show="True">
              <Field table="Customer" id="2" name="Name"/>
            </SecondaryField>
          </ValueList>
        </ValueListCatalog>"#;
        let (mut r, mut buf) = make_reader(xml);
        consume_opening(&mut r, &mut buf, b"ValueListCatalog");
        let vls = parse_value_lists(&mut r, &mut buf).unwrap();
        assert_eq!(vls[0].field_refs.len(), 2);
        assert_eq!(vls[0].field_refs[0].table_occurrence, "Customer");
        assert_eq!(vls[0].field_refs[0].field_name, "ID");
        assert_eq!(vls[0].field_refs[1].table_occurrence, "Customer");
        assert_eq!(vls[0].field_refs[1].field_name, "Name");
    }

    // --- Custom Functions ---

    #[test]
    fn custom_function_with_params() {
        let xml = r#"<CustomFunctionCatalog>
          <CustomFunction id="1" name="MyFunc" parameters="param1; param2">
            <Calculation>param1 + param2</Calculation>
          </CustomFunction>
        </CustomFunctionCatalog>"#;
        let (mut r, mut buf) = make_reader(xml);
        consume_opening(&mut r, &mut buf, b"CustomFunctionCatalog");
        let cfs = parse_custom_functions(&mut r, &mut buf).unwrap();
        assert_eq!(cfs.len(), 1);
        assert_eq!(cfs[0].name, "MyFunc");
        assert_eq!(cfs[0].parameters, vec!["param1", "param2"]);
        assert_eq!(cfs[0].calculation.as_deref(), Some("param1 + param2"));
    }

    #[test]
    fn custom_function_no_params() {
        let xml = r#"<CustomFunctionCatalog>
          <CustomFunction id="2" name="NoArgs" parameters=""/>
        </CustomFunctionCatalog>"#;
        let (mut r, mut buf) = make_reader(xml);
        consume_opening(&mut r, &mut buf, b"CustomFunctionCatalog");
        let cfs = parse_custom_functions(&mut r, &mut buf).unwrap();
        assert!(cfs[0].parameters.is_empty());
    }

    // --- Accounts ---

    #[test]
    fn account_enabled() {
        let xml = r#"<AccountCatalog>
          <Account id="1" name="Admin" type="FileMaker" privilegeSet="[Full Access]" enabled="True"/>
        </AccountCatalog>"#;
        let (mut r, mut buf) = make_reader(xml);
        consume_opening(&mut r, &mut buf, b"AccountCatalog");
        let accounts = parse_accounts(&mut r, &mut buf).unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].name, "Admin");
        assert_eq!(accounts[0].privilege_set.as_deref(), Some("[Full Access]"));
        assert!(accounts[0].enabled);
    }

    #[test]
    fn account_disabled() {
        let xml = r#"<AccountCatalog>
          <Account id="2" name="OldUser" type="FileMaker" privilegeSet="[Read-Only Access]" enabled="False"/>
        </AccountCatalog>"#;
        let (mut r, mut buf) = make_reader(xml);
        consume_opening(&mut r, &mut buf, b"AccountCatalog");
        let accounts = parse_accounts(&mut r, &mut buf).unwrap();
        assert!(!accounts[0].enabled);
    }

    // --- Privilege Sets ---

    #[test]
    fn privilege_set_with_comment() {
        let xml = r#"<PrivilegesCatalog>
          <PrivilegeSet id="1" name="[Full Access]" comment="Full access"/>
        </PrivilegesCatalog>"#;
        let (mut r, mut buf) = make_reader(xml);
        consume_opening(&mut r, &mut buf, b"PrivilegesCatalog");
        let ps = parse_privilege_sets(&mut r, &mut buf).unwrap();
        assert_eq!(ps.len(), 1);
        assert_eq!(ps[0].name, "[Full Access]");
        assert_eq!(ps[0].comment.as_deref(), Some("Full access"));
    }

    #[rstest]
    #[case(0)]
    #[case(2)]
    fn privilege_set_count(#[case] count: usize) {
        let inner: String = (1..=count)
            .map(|i| format!(r#"<PrivilegeSet id="{i}" name="PS{i}" comment=""/>"#))
            .collect();
        let xml = format!("<PrivilegesCatalog>{inner}</PrivilegesCatalog>");
        let (mut r, mut buf) = make_reader(&xml);
        consume_opening(&mut r, &mut buf, b"PrivilegesCatalog");
        let ps = parse_privilege_sets(&mut r, &mut buf).unwrap();
        assert_eq!(ps.len(), count);
    }
}
