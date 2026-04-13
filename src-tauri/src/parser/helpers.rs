use std::io::BufRead;

use quick_xml::{events::Event, Reader};

use crate::parser::ParseError;

/// Skip the current element and all of its descendants.
///
/// Must be called immediately after consuming the **opening** start tag.
/// The function reads until the matching end tag (depth returns to 0).
pub fn skip_element<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<(), ParseError> {
    let mut depth: usize = 1;
    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(_) => depth += 1,
            Event::End(_) => {
                depth -= 1;
                if depth == 0 {
                    return Ok(());
                }
            }
            Event::Eof => return Err(ParseError::UnexpectedEof),
            // Empty elements do not change depth.
            _ => {}
        }
    }
}

/// Extract the value of an attribute by key from a start/empty tag.
///
/// Returns `Err(ParseError::MissingAttribute)` if the attribute is absent.
pub fn get_attr(e: &quick_xml::events::BytesStart<'_>, key: &[u8]) -> Result<String, ParseError> {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key)
        .ok_or_else(|| ParseError::MissingAttribute(String::from_utf8_lossy(key).into_owned()))
        .and_then(|a| {
            a.unescape_value()
                .map(|v| v.into_owned())
                .map_err(ParseError::Xml)
        })
}

/// Read the text content (including CDATA) of the current element and consume its closing tag.
///
/// Must be called after the opening `<Tag>` has been consumed.
/// CR (`\r`) and CRLF (`\r\n`) are normalized to LF (`\n`) to handle FileMaker's
/// CDATA sections which use CR-only line endings.
pub fn read_text_content<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> Result<String, ParseError> {
    // Entity 参照（&quot; 等）前後の空白が trim_text によって失われないよう、
    // テキスト内容の読み取り中はトリミングを無効化する。
    let trim_start = reader.config().trim_text_start;
    let trim_end = reader.config().trim_text_end;
    reader.config_mut().trim_text(false);

    let mut text = String::new();
    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Text(ref t) => {
                let decoded = t
                    .decode()
                    .map_err(|e| ParseError::Xml(quick_xml::Error::Encoding(e)))?;
                let unescaped = quick_xml::escape::unescape(&decoded)
                    .map_err(|_| ParseError::InvalidValue(decoded.to_string()))?;
                // push_str で追記する（quick-xml は &quot; 等のエンティティを含む
                // テキストを複数の Text イベントに分割することがあるため、
                // 代入ではなく追記しなければ先行するテキストが失われる）
                text.push_str(&normalize_line_endings(unescaped.into_owned()));
            }
            // CDATA sections (e.g. `<![CDATA[Get ( UUID )]]>`) in real DDR files
            Event::CData(ref cd) => {
                let raw = std::str::from_utf8(cd.as_ref())
                    .map_err(|_| ParseError::InvalidValue("invalid UTF-8 in CDATA".into()))?;
                text.push_str(&normalize_line_endings(raw.to_owned()));
            }
            // quick-xml 0.39 は &quot; などのエンティティ参照を
            // Event::GeneralRef として個別にemitする。
            // 事前定義エンティティと文字参照を解決してテキストに追記する。
            Event::GeneralRef(ref r) => {
                if let Some(ch) = r.resolve_char_ref().ok().flatten() {
                    text.push(ch);
                } else if let Ok(name) = r.decode() {
                    let resolved = match name.as_ref() {
                        "quot" => "\"",
                        "amp" => "&",
                        "apos" => "'",
                        "lt" => "<",
                        "gt" => ">",
                        _ => "",
                    };
                    text.push_str(resolved);
                }
            }
            Event::End(_) => break,
            Event::Eof => {
                reader.config_mut().trim_text_start = trim_start;
                reader.config_mut().trim_text_end = trim_end;
                return Err(ParseError::UnexpectedEof);
            }
            _ => {}
        }
    }
    reader.config_mut().trim_text_start = trim_start;
    reader.config_mut().trim_text_end = trim_end;
    Ok(normalize_line_endings(text))
}

/// CR+LF および CR-only を LF に正規化する。
/// FileMaker DDR の CDATA は CR のみを改行として使用することがある。
fn normalize_line_endings(s: String) -> String {
    // \r\n → \n を先に処理し、残った \r → \n
    if s.contains('\r') {
        s.replace("\r\n", "\n").replace('\r', "\n")
    } else {
        s
    }
}

/// Like `get_attr`, but intended for attributes that are mandatory per the DDR schema.
/// Currently an alias for `get_attr`.
#[allow(dead_code)]
pub fn require_attr(
    e: &quick_xml::events::BytesStart<'_>,
    key: &[u8],
) -> Result<String, ParseError> {
    get_attr(e, key)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::events::Event;
    use quick_xml::Reader;

    fn make_reader(xml: &str) -> (Reader<&[u8]>, Vec<u8>) {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        (reader, Vec::new())
    }

    #[test]
    fn get_attr_present() {
        let xml = r#"<Tag name="Hello" id="42"/>"#;
        let (mut reader, mut buf) = make_reader(xml);
        buf.clear();
        match reader.read_event_into(&mut buf).unwrap() {
            Event::Empty(ref e) => {
                assert_eq!(get_attr(e, b"name").unwrap(), "Hello");
                assert_eq!(get_attr(e, b"id").unwrap(), "42");
            }
            other => panic!("expected Empty event, got {other:?}"),
        }
    }

    #[test]
    fn get_attr_missing_returns_err() {
        let xml = r#"<Tag name="Hello"/>"#;
        let (mut reader, mut buf) = make_reader(xml);
        buf.clear();
        match reader.read_event_into(&mut buf).unwrap() {
            Event::Empty(ref e) => {
                assert!(get_attr(e, b"notexist").is_err());
            }
            other => panic!("expected Empty event, got {other:?}"),
        }
    }

    #[test]
    fn skip_element_single_level() {
        // After consuming <Outer>, skip_element should consume everything up to </Outer>.
        let xml = "<Outer><Inner1/><Inner2>text</Inner2></Outer><Next/>";
        let (mut reader, mut buf) = make_reader(xml);
        // consume <Outer>
        buf.clear();
        reader.read_event_into(&mut buf).unwrap();
        // skip the element
        skip_element(&mut reader, &mut buf).unwrap();
        // next event should be <Next/>
        buf.clear();
        match reader.read_event_into(&mut buf).unwrap() {
            Event::Empty(ref e) => assert_eq!(e.name().as_ref(), b"Next"),
            other => panic!("expected <Next/>, got {other:?}"),
        }
    }

    #[test]
    fn skip_element_nested() {
        let xml = "<A><B><C/></B><D/></A><After/>";
        let (mut reader, mut buf) = make_reader(xml);
        buf.clear();
        reader.read_event_into(&mut buf).unwrap(); // <A>
        skip_element(&mut reader, &mut buf).unwrap();
        buf.clear();
        match reader.read_event_into(&mut buf).unwrap() {
            Event::Empty(ref e) => assert_eq!(e.name().as_ref(), b"After"),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn read_text_content_with_xml_entities_is_complete() {
        // StepText containing &quot; entities — quick-xml may emit multiple Text events,
        // so the content must be appended (not overwritten).
        let xml = r#"<Root><StepText>Hello &quot;World&quot; ) ]</StepText></Root>"#;
        let (mut reader, mut buf) = make_reader(xml);
        buf.clear();
        reader.read_event_into(&mut buf).unwrap(); // <Root>
        buf.clear();
        reader.read_event_into(&mut buf).unwrap(); // <StepText>
        let text = read_text_content(&mut reader, &mut buf).unwrap();
        assert_eq!(text, r#"Hello "World" ) ]"#);
    }

    #[test]
    fn read_text_content_with_multiple_entities() {
        // 実DDR形式: JSONGetElement ( $record; &quot;PIC&quot; ) ]
        let xml = r#"<Root><StepText>フィールド設定 [ T::F; JSONGetElement ( $r; &quot;PIC&quot; ) ]</StepText></Root>"#;
        let (mut reader, mut buf) = make_reader(xml);
        buf.clear();
        reader.read_event_into(&mut buf).unwrap(); // <Root>
        buf.clear();
        reader.read_event_into(&mut buf).unwrap(); // <StepText>
        let text = read_text_content(&mut reader, &mut buf).unwrap();
        assert_eq!(
            text,
            r#"フィールド設定 [ T::F; JSONGetElement ( $r; "PIC" ) ]"#
        );
    }
}
