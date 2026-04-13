use std::io::BufRead;

use quick_xml::{events::Event, Reader};

use crate::parser::{
    helpers::{get_attr, skip_element},
    models::{JoinPredicate, Relationship, RelationshipId, TableOccurrence},
    ParseError,
};

/// Parse `<RelationshipGraph>` content.
///
/// Returns `(relationships, table_occurrences)`.
/// The caller must have already consumed the opening `<RelationshipGraph>` tag.
pub fn parse_relationships<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<(Vec<Relationship>, Vec<TableOccurrence>), ParseError> {
    let mut relationships = Vec::new();
    let mut table_occurrences = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"TableList" => {
                table_occurrences = parse_table_list(reader, buf)?;
            }
            Event::Empty(ref e) if e.name().as_ref() == b"TableList" => {}
            Event::Start(ref e) if e.name().as_ref() == b"RelationshipList" => {
                relationships = parse_relationship_list(reader, buf)?;
            }
            Event::Empty(ref e) if e.name().as_ref() == b"RelationshipList" => {
                // No relationships
            }
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </RelationshipGraph>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok((relationships, table_occurrences))
}

/// Parse `<TableList>` to extract table occurrence → base table name mappings.
fn parse_table_list<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Vec<TableOccurrence>, ParseError> {
    let mut occurrences = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Empty(ref e) if e.name().as_ref() == b"Table" => {
                let occ_name = get_attr(e, b"name").unwrap_or_default();
                // baseTable がない場合はオカレンス名をそのままベーステーブル名とみなす
                let base_name = get_attr(e, b"baseTable").unwrap_or_else(|_| occ_name.clone());
                if !occ_name.is_empty() {
                    occurrences.push(TableOccurrence {
                        occurrence_name: occ_name,
                        base_table_name: base_name,
                        source_file: None,
                    });
                }
            }
            Event::Start(ref e) if e.name().as_ref() == b"Table" => {
                let occ_name = get_attr(e, b"name").unwrap_or_default();
                let base_name = get_attr(e, b"baseTable").unwrap_or_else(|_| occ_name.clone());
                let source_file = extract_file_reference(reader, buf)?;
                if !occ_name.is_empty() {
                    occurrences.push(TableOccurrence {
                        occurrence_name: occ_name,
                        base_table_name: base_name,
                        source_file,
                    });
                }
            }
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </TableList>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok(occurrences)
}

/// Parse the content of a `<RelationshipList>` element (already consumed).
fn parse_relationship_list<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Vec<Relationship>, ParseError> {
    let mut relationships = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"Relationship" => {
                let id = get_attr(e, b"id")
                    .and_then(|v| {
                        v.parse::<u64>()
                            .map_err(|_| ParseError::InvalidValue(format!("Relationship id: {v}")))
                    })
                    .unwrap_or(0);
                let name = get_attr(e, b"name").unwrap_or_default();
                let rel = parse_relationship_children(reader, buf, id, name)?;
                relationships.push(rel);
            }
            Event::Empty(ref e) if e.name().as_ref() == b"Relationship" => {
                let id = get_attr(e, b"id")
                    .and_then(|v| {
                        v.parse::<u64>()
                            .map_err(|_| ParseError::InvalidValue(format!("Relationship id: {v}")))
                    })
                    .unwrap_or(0);
                let name = get_attr(e, b"name").unwrap_or_default();
                relationships.push(Relationship {
                    id: RelationshipId(id),
                    name,
                    left_table: String::new(),
                    right_table: String::new(),
                    predicates: Vec::new(),
                });
            }
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </RelationshipList>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok(relationships)
}

/// Parse children of a `<Relationship>` element (opening tag already consumed).
fn parse_relationship_children<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    id: u64,
    name: String,
) -> Result<Relationship, ParseError> {
    let mut left_table = String::new();
    let mut right_table = String::new();
    let mut predicates = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"LeftTable" => {
                left_table = get_attr(e, b"name").unwrap_or_default();
                skip_element(reader, buf)?;
            }
            Event::Empty(ref e) if e.name().as_ref() == b"LeftTable" => {
                left_table = get_attr(e, b"name").unwrap_or_default();
            }
            Event::Start(ref e) if e.name().as_ref() == b"RightTable" => {
                right_table = get_attr(e, b"name").unwrap_or_default();
                skip_element(reader, buf)?;
            }
            Event::Empty(ref e) if e.name().as_ref() == b"RightTable" => {
                right_table = get_attr(e, b"name").unwrap_or_default();
            }
            Event::Start(ref e) if e.name().as_ref() == b"JoinPredicateList" => {
                predicates = parse_join_predicate_list(reader, buf)?;
            }
            Event::Empty(ref e) if e.name().as_ref() == b"JoinPredicateList" => {}
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </Relationship>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok(Relationship {
        id: RelationshipId(id),
        name,
        left_table,
        right_table,
        predicates,
    })
}

/// Parse the content of a `<JoinPredicateList>` element (already consumed).
fn parse_join_predicate_list<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Vec<JoinPredicate>, ParseError> {
    let mut predicates = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"JoinPredicate" => {
                let operator = get_attr(e, b"type").unwrap_or_default();
                let pred = parse_join_predicate_children(reader, buf, operator)?;
                predicates.push(pred);
            }
            Event::Empty(ref e) if e.name().as_ref() == b"JoinPredicate" => {
                predicates.push(JoinPredicate {
                    left_field: String::new(),
                    right_field: String::new(),
                    operator: get_attr(e, b"type").unwrap_or_default(),
                });
            }
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </JoinPredicateList>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok(predicates)
}

/// Parse children of a `<JoinPredicate>` element (opening tag already consumed).
fn parse_join_predicate_children<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    operator: String,
) -> Result<JoinPredicate, ParseError> {
    let mut left_field = String::new();
    let mut right_field = String::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            // 実際のDDR XML: <LeftField><Field name="..."/></LeftField>
            Event::Start(ref e) if e.name().as_ref() == b"LeftField" => {
                left_field = extract_field_name(reader, buf)?;
            }
            // 古い形式 / テスト用: <LeftField name="..."/>
            Event::Empty(ref e) if e.name().as_ref() == b"LeftField" => {
                left_field = get_attr(e, b"name").unwrap_or_default();
            }
            Event::Start(ref e) if e.name().as_ref() == b"RightField" => {
                right_field = extract_field_name(reader, buf)?;
            }
            Event::Empty(ref e) if e.name().as_ref() == b"RightField" => {
                right_field = get_attr(e, b"name").unwrap_or_default();
            }
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </JoinPredicate>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok(JoinPredicate {
        left_field,
        right_field,
        operator,
    })
}

/// `<Table>` 要素の子から `<FileReference name="..."/>` を探して参照元ファイル名を返す。
/// 開始タグは既に消費済み。FileReference が見つからなければ None を返す。
fn extract_file_reference<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<Option<String>, ParseError> {
    let mut source_file: Option<String> = None;
    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Empty(ref e) if e.name().as_ref() == b"FileReference" => {
                source_file = get_attr(e, b"name").ok();
            }
            Event::Start(ref e) if e.name().as_ref() == b"FileReference" => {
                source_file = get_attr(e, b"name").ok();
                skip_element(reader, buf)?;
            }
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </Table>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }
    Ok(source_file)
}

/// `<LeftField>` / `<RightField>` コンテナの中にある `<Field name="..."/>` から
/// フィールド名を取り出す。コンテナの開始タグは既に消費済み。
fn extract_field_name<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<String, ParseError> {
    let mut name = String::new();
    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"Field" => {
                name = get_attr(e, b"name").unwrap_or_default();
                skip_element(reader, buf)?;
            }
            Event::Empty(ref e) if e.name().as_ref() == b"Field" => {
                name = get_attr(e, b"name").unwrap_or_default();
            }
            Event::Start(_) => {
                skip_element(reader, buf)?;
            }
            Event::Empty(_) => {}
            Event::End(_) => break, // </LeftField> or </RightField>
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }
    Ok(name)
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

    fn parse(xml: &str) -> Result<Vec<Relationship>, ParseError> {
        let full = format!("<RelationshipGraph>{xml}</RelationshipGraph>");
        let (mut reader, mut buf) = make_reader(&full);
        loop {
            buf.clear();
            match reader.read_event_into(&mut buf).unwrap() {
                Event::Start(ref e) if e.name().as_ref() == b"RelationshipGraph" => break,
                _ => {}
            }
        }
        parse_relationships(&mut reader, &mut buf).map(|(rels, _)| rels)
    }

    #[test]
    fn empty_graph() {
        assert!(parse("").unwrap().is_empty());
    }

    #[test]
    fn empty_relationship_list() {
        assert!(parse("<RelationshipList/>").unwrap().is_empty());
    }

    #[test]
    fn single_relationship() {
        let xml = r#"
        <TableList>
          <Table id="1" name="Contact" baseTable="Contact" baseTableID="1"/>
        </TableList>
        <RelationshipList>
          <Relationship id="1" name="Contact_Project">
            <LeftTable id="1" name="Contact" baseTable="Contact"/>
            <RightTable id="2" name="Project" baseTable="Project"/>
            <JoinPredicateList>
              <JoinPredicate id="1" type="Equal">
                <LeftField id="5" name="_kf_ContactID" repetition="1"/>
                <RightField id="5" name="_kf_ContactID" repetition="1"/>
              </JoinPredicate>
            </JoinPredicateList>
          </Relationship>
        </RelationshipList>
        "#;
        let rels = parse(xml).unwrap();
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].name, "Contact_Project");
        assert_eq!(rels[0].left_table, "Contact");
        assert_eq!(rels[0].right_table, "Project");
        assert_eq!(rels[0].predicates.len(), 1);
        assert_eq!(rels[0].predicates[0].operator, "Equal");
        assert_eq!(rels[0].predicates[0].left_field, "_kf_ContactID");
        assert_eq!(rels[0].predicates[0].right_field, "_kf_ContactID");
    }

    #[rstest]
    #[case(0)]
    #[case(1)]
    #[case(3)]
    fn relationship_count(#[case] count: usize) {
        let inner: String = (1..=count)
            .map(|i| {
                format!(
                    r#"<Relationship id="{i}" name="Rel{i}">
                      <LeftTable id="1" name="T1" baseTable="T1"/>
                      <RightTable id="2" name="T2" baseTable="T2"/>
                      <JoinPredicateList/>
                    </Relationship>"#
                )
            })
            .collect();
        let xml = format!("<RelationshipList>{inner}</RelationshipList>");
        let rels = parse(&xml).unwrap();
        assert_eq!(rels.len(), count);
    }

    /// 実際のDDR XML形式: <LeftField><Field name="..."/></LeftField>
    #[test]
    fn predicate_with_nested_field_elements() {
        let xml = r#"
        <TableList>
          <Table id="1" name="A" baseTable="A" baseTableID="1"/>
          <Table id="2" name="B" baseTable="B" baseTableID="2"/>
        </TableList>
        <RelationshipList>
          <Relationship id="1" name="A_B">
            <LeftTable id="1" name="A" baseTable="A"/>
            <RightTable id="2" name="B" baseTable="B"/>
            <JoinPredicateList>
              <JoinPredicate id="0" type="Equal">
                <LeftField>
                  <Field table="A" id="5" name="Q_No" repetition="1"/>
                </LeftField>
                <RightField>
                  <Field table="B" id="3" name="customer_id" repetition="1"/>
                </RightField>
              </JoinPredicate>
            </JoinPredicateList>
          </Relationship>
        </RelationshipList>"#;
        let rels = parse(xml).unwrap();
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].predicates.len(), 1);
        assert_eq!(rels[0].predicates[0].left_field, "Q_No");
        assert_eq!(rels[0].predicates[0].right_field, "customer_id");
        assert_eq!(rels[0].predicates[0].operator, "Equal");
    }
}
