//! 概要.xml（Summary XML）パーサー。
//!
//! FileMaker DDR の概要ファイルを解析し、各 DBファイルへのリンク一覧を返す。

use quick_xml::{events::Event, Reader};

use crate::parser::{
    helpers::{get_attr, skip_element},
    ParseError,
};

// ---------------------------------------------------------------------------
// 公開型
// ---------------------------------------------------------------------------

/// 概要.xml 内の1ファイルエントリ。
#[derive(Debug, Clone, PartialEq)]
pub struct SummaryEntry {
    /// ファイル名（例: "BOSS.fmp12"）
    pub name: String,
    /// 詳細 XML へのリンク（例: ".//BOSS_fmp12.xml"）
    pub link: String,
}

// ---------------------------------------------------------------------------
// 公開関数
// ---------------------------------------------------------------------------

/// 概要 XML 文字列を解析し、`SummaryEntry` の一覧を返す。
///
/// `<FMPReport type="Summary">` 以外が渡された場合はエラーを返す。
pub fn parse_summary(xml: &str) -> Result<Vec<SummaryEntry>, ParseError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut entries: Vec<SummaryEntry> = Vec::new();
    let mut found_root = false;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Eof => break,

            Event::Start(ref e) => {
                let tag = e.name().as_ref().to_owned();
                match tag.as_slice() {
                    b"FMPReport" => {
                        found_root = true;
                        let report_type = get_attr(e, b"type")?;
                        if report_type != "Summary" {
                            return Err(ParseError::InvalidValue(format!(
                                "Expected Summary type, got: {report_type}"
                            )));
                        }
                        // Continue parsing children
                    }
                    b"File" => {
                        let name = get_attr(e, b"name")?;
                        let link = get_attr(e, b"link")?;
                        entries.push(SummaryEntry { name, link });
                        // Skip all children of <File> (BaseTables, Scripts, etc.)
                        skip_element(&mut reader, &mut buf)?;
                    }
                    _ => {
                        skip_element(&mut reader, &mut buf)?;
                    }
                }
            }

            Event::Empty(ref e) => {
                let tag = e.name().as_ref().to_owned();
                match tag.as_slice() {
                    b"FMPReport" => {
                        found_root = true;
                        let report_type = get_attr(e, b"type")?;
                        if report_type != "Summary" {
                            return Err(ParseError::InvalidValue(format!(
                                "Expected Summary type, got: {report_type}"
                            )));
                        }
                        // No children — done
                    }
                    b"File" => {
                        let name = get_attr(e, b"name")?;
                        let link = get_attr(e, b"link")?;
                        entries.push(SummaryEntry { name, link });
                    }
                    _ => {}
                }
            }

            Event::End(_) => {}
            _ => {}
        }
    }

    if !found_root {
        return Err(ParseError::MissingElement("FMPReport".to_string()));
    }

    Ok(entries)
}

/// `link` 属性の先頭 `./` や `.//` プレフィックスを除去してファイル名部分を返す。
///
/// 例:
/// - `.//BOSS_fmp12.xml` → `BOSS_fmp12.xml`
/// - `./BOSS_fmp12.xml`  → `BOSS_fmp12.xml`
/// - `BOSS_fmp12.xml`    → `BOSS_fmp12.xml`
pub fn normalize_link(link: &str) -> &str {
    let s = link.strip_prefix(".//").unwrap_or(link);
    let s = s.strip_prefix("./").unwrap_or(s);
    s
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- parse_summary ----

    #[test]
    fn empty_summary_returns_empty() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<FMPReport type="Summary" version="21.0v1"/>"#;
        let entries = parse_summary(xml).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn single_file_entry() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<FMPReport type="Summary" version="21.0v1">
  <File link=".//BOSS_fmp12.xml" name="BOSS.fmp12" path="FMS20.local"/>
</FMPReport>"#;
        let entries = parse_summary(xml).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "BOSS.fmp12");
        assert_eq!(entries[0].link, ".//BOSS_fmp12.xml");
    }

    #[test]
    fn multiple_file_entries() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<FMPReport type="Summary" version="21.0v1">
  <File link=".//A_fmp12.xml" name="A.fmp12" path="localhost"/>
  <File link=".//B_fmp12.xml" name="B.fmp12" path="localhost"/>
</FMPReport>"#;
        let entries = parse_summary(xml).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "A.fmp12");
        assert_eq!(entries[1].name, "B.fmp12");
    }

    #[test]
    fn file_children_are_skipped() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<FMPReport type="Summary" version="21.0v1">
  <File link=".//BOSS_fmp12.xml" name="BOSS.fmp12" path="FMS20.local">
    <BaseTables count="13"/>
    <Scripts count="5"/>
  </File>
</FMPReport>"#;
        let entries = parse_summary(xml).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "BOSS.fmp12");
    }

    #[test]
    fn wrong_type_returns_error() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<FMPReport type="Report" version="21.0v1">
  <File name="TestDB"/>
</FMPReport>"#;
        let result = parse_summary(xml);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("Summary"),
            "error should mention 'Summary', got: {msg}"
        );
    }

    #[test]
    fn missing_fmpreport_returns_error() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<NotFMPReport type="Summary" version="21.0v1"/>"#;
        let result = parse_summary(xml);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("FMPReport"),
            "error should mention 'FMPReport', got: {msg}"
        );
    }

    // ---- normalize_link ----

    #[test]
    fn normalize_link_double_slash() {
        assert_eq!(normalize_link(".//BOSS_fmp12.xml"), "BOSS_fmp12.xml");
    }

    #[test]
    fn normalize_link_single_slash() {
        assert_eq!(normalize_link("./BOSS_fmp12.xml"), "BOSS_fmp12.xml");
    }

    #[test]
    fn normalize_link_no_prefix() {
        assert_eq!(normalize_link("BOSS_fmp12.xml"), "BOSS_fmp12.xml");
    }
}
