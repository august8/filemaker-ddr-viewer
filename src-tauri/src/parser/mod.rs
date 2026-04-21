//! DDR XML parser module.
//!
//! The primary entry point is [`ddr_reader::parse_ddr`].
//!
//! ## Module layout
//! - [`models`] — all data structures
//! - [`version`] — `FmVersion` + `VersionAdapter`
//! - [`ddr_reader`] — main `parse_ddr()` function + shared XML helpers (via `helpers`)
//! - [`table_parser`] — `<BaseTableCatalog>` parsing
//! - [`script_parser`] — `<ScriptCatalog>` parsing
//! - [`layout_parser`] — `<LayoutCatalog>` parsing
//! - [`relationship_parser`] — `<RelationshipGraph>` parsing
//! - [`catalog_parser`] — value lists, custom functions, accounts, privilege sets

// Internal helpers – not part of the public API
pub(crate) mod helpers;

// Sub-modules
pub mod catalog_parser;
pub mod ddr_reader;
pub mod layout_parser;
pub mod models;
pub mod relationship_parser;
pub mod script_parser;
pub mod summary_parser;
pub mod table_parser;
pub mod version;

// ---------------------------------------------------------------------------
// Public error type
// ---------------------------------------------------------------------------

/// Errors that can occur during DDR XML parsing.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("Attribute error: {0}")]
    Attr(#[from] quick_xml::events::attributes::AttrError),

    #[error("Missing attribute '{0}'")]
    MissingAttribute(String),

    #[error("Missing element '{0}'")]
    MissingElement(String),

    #[error("Invalid version: {0}")]
    InvalidVersion(String),

    #[error("Unexpected end of file")]
    UnexpectedEof,

    #[error("Invalid value: {0}")]
    InvalidValue(String),
}

// ---------------------------------------------------------------------------
// Public re-exports (convenience)
// ---------------------------------------------------------------------------

pub use ddr_reader::parse_ddr;
pub use models::*;
pub use summary_parser::{normalize_link, parse_summary, SummaryEntry};
pub use version::{FmVersion, VersionAdapter};

// ---------------------------------------------------------------------------
// DDR バイト列デコード
// ---------------------------------------------------------------------------

/// DDR ファイルのバイト列を UTF-8 文字列にデコードする。
///
/// FileMaker DDR は UTF-16 LE BOM (`\xFF\xFE`) または UTF-8 で出力される。
/// UTF-16 BE BOM (`\xFE\xFF`) および UTF-8 BOM (`\xEF\xBB\xBF`) にも対応。
pub fn decode_ddr_bytes(bytes: &[u8]) -> Result<String, ParseError> {
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        // UTF-16 LE
        let words: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16(&words)
            .or_else(|_| Ok::<String, ParseError>(String::from_utf16_lossy(&words)))
    } else if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        // UTF-16 BE
        let words: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16(&words)
            .or_else(|_| Ok::<String, ParseError>(String::from_utf16_lossy(&words)))
    } else if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        // UTF-8 BOM
        String::from_utf8(bytes[3..].to_vec())
            .map_err(|e| ParseError::InvalidValue(format!("UTF-8 デコードエラー: {e}")))
    } else {
        // UTF-8 (BOM なし)
        String::from_utf8(bytes.to_vec())
            .map_err(|e| ParseError::InvalidValue(format!("UTF-8 デコードエラー: {e}")))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_error_display_xml() {
        // Ensure the error Display works (smoke test)
        let e = ParseError::MissingAttribute("name".to_owned());
        assert!(e.to_string().contains("name"));
    }

    #[test]
    fn parse_error_display_missing_element() {
        let e = ParseError::MissingElement("FMPReport".to_owned());
        assert!(e.to_string().contains("FMPReport"));
    }

    #[test]
    fn parse_error_display_invalid_version() {
        let e = ParseError::InvalidVersion("bad".to_owned());
        assert!(e.to_string().contains("bad"));
    }

    #[test]
    fn decode_ddr_bytes_utf8() {
        let xml = b"<?xml version=\"1.0\"?><root/>";
        let result = decode_ddr_bytes(xml).unwrap();
        assert!(result.contains("<root/>"));
    }

    #[test]
    fn decode_ddr_bytes_utf16_le_bom() {
        // UTF-16 LE BOM + "AB"
        let mut bytes: Vec<u8> = vec![0xFF, 0xFE];
        for ch in "AB".encode_utf16() {
            bytes.extend_from_slice(&ch.to_le_bytes());
        }
        let result = decode_ddr_bytes(&bytes).unwrap();
        assert_eq!(result, "AB");
    }

    #[test]
    fn decode_ddr_bytes_utf8_bom() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(b"hello");
        let result = decode_ddr_bytes(&bytes).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn decode_ddr_bytes_utf16_le_invalid_surrogate_falls_back_lossy() {
        // UTF-16 LE BOM + 孤立サロゲート 0xD800 + 'A'
        let mut bytes: Vec<u8> = vec![0xFF, 0xFE];
        bytes.extend_from_slice(&0xD800_u16.to_le_bytes());
        bytes.extend_from_slice(&('A' as u16).to_le_bytes());
        let result = decode_ddr_bytes(&bytes);
        assert!(
            result.is_ok(),
            "不正UTF-16でもlossyフォールバックでOkを返すこと"
        );
        let s = result.unwrap();
        assert!(s.contains('A'), "有効な文字はそのまま含まれること");
        assert!(
            s.contains('\u{FFFD}'),
            "不正サロゲートはU+FFDDに置換されること"
        );
    }

    #[test]
    fn decode_ddr_bytes_utf16_be_invalid_surrogate_falls_back_lossy() {
        // UTF-16 BE BOM + 孤立サロゲート 0xDC00 + 'B'
        let mut bytes: Vec<u8> = vec![0xFE, 0xFF];
        bytes.extend_from_slice(&0xDC00_u16.to_be_bytes());
        bytes.extend_from_slice(&('B' as u16).to_be_bytes());
        let result = decode_ddr_bytes(&bytes);
        assert!(
            result.is_ok(),
            "BE版でも不正UTF-16でlossyフォールバックすること"
        );
        let s = result.unwrap();
        assert!(s.contains('B'));
        assert!(s.contains('\u{FFFD}'));
    }
}
