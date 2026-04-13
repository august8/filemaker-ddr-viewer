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
}
