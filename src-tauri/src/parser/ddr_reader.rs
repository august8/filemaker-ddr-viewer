use quick_xml::{events::Event, Reader};

use crate::parser::{
    catalog_parser::{
        parse_accounts, parse_custom_functions, parse_external_data_sources, parse_privilege_sets,
        parse_value_lists,
    },
    helpers::{get_attr, skip_element},
    layout_parser::parse_layouts,
    models::DdrFile,
    relationship_parser::parse_relationships,
    script_parser::parse_scripts,
    table_parser::parse_tables,
    version::FmVersion,
    ParseError,
};

/// Parse a FileMaker DDR XML string into a `DdrFile`.
///
/// This is the primary entry point for the parser module.
pub fn parse_ddr(xml: &str) -> Result<DdrFile, ParseError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    // ------------------------------------------------------------------
    // Find <FMPReport> and extract the version attribute
    // ------------------------------------------------------------------
    let fm_version = find_fmp_report(&mut reader, &mut buf)?;

    // ------------------------------------------------------------------
    // Find <File name="..."> inside FMPReport
    // ------------------------------------------------------------------
    let file_name = find_file_element(&mut reader, &mut buf)?;

    // ------------------------------------------------------------------
    // Parse each catalog section
    // ------------------------------------------------------------------
    let mut tables = Vec::new();
    let mut scripts = Vec::new();
    let mut layouts = Vec::new();
    let mut relationships = Vec::new();
    let mut table_occurrences = Vec::new();
    let mut value_lists = Vec::new();
    let mut custom_functions = Vec::new();
    let mut accounts = Vec::new();
    let mut privilege_sets = Vec::new();
    let mut file_script_triggers: Vec<String> = Vec::new();
    let mut external_data_sources = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            // ---- Catalogs that may be non-empty (Start tag) ----
            Event::Start(ref e) => {
                // Capture the tag name before releasing the borrow on `buf`.
                let tag = e.name().as_ref().to_owned();
                match tag.as_slice() {
                    b"BaseTableCatalog" => {
                        tables = parse_tables(&mut reader, &mut buf)?;
                    }
                    b"RelationshipGraph" => {
                        let (rels, occs) = parse_relationships(&mut reader, &mut buf)?;
                        relationships = rels;
                        table_occurrences = occs;
                    }
                    b"LayoutCatalog" => {
                        layouts = parse_layouts(&mut reader, &mut buf)?;
                    }
                    b"ScriptCatalog" => {
                        scripts = parse_scripts(&mut reader, &mut buf)?;
                    }
                    b"ValueListCatalog" => {
                        value_lists = parse_value_lists(&mut reader, &mut buf)?;
                    }
                    b"AccountCatalog" => {
                        accounts = parse_accounts(&mut reader, &mut buf)?;
                    }
                    b"PrivilegesCatalog" => {
                        privilege_sets = parse_privilege_sets(&mut reader, &mut buf)?;
                    }
                    b"CustomFunctionCatalog" => {
                        custom_functions = parse_custom_functions(&mut reader, &mut buf)?;
                    }
                    b"Options" => {
                        file_script_triggers = parse_options(&mut reader, &mut buf)?;
                    }
                    b"ExternalDataSourcesCatalog" => {
                        external_data_sources = parse_external_data_sources(&mut reader, &mut buf)?;
                    }
                    _ => {
                        // Unknown element (e.g. File, ExtendedPrivilegeCatalog) – skip
                        skip_element(&mut reader, &mut buf)?;
                    }
                }
            }

            // ---- Empty catalogs (e.g. <ValueListCatalog/>) ----
            Event::Empty(_) => {
                // Nothing to parse; leave the corresponding Vec empty
            }

            // End of <File> or <FMPReport>
            Event::End(_) => {
                // Could be </File> or </FMPReport> – either way we're done
            }

            Event::Eof => break,
            _ => {}
        }
    }

    Ok(DdrFile {
        file_name,
        fm_version,
        tables,
        scripts,
        layouts,
        relationships,
        table_occurrences,
        value_lists,
        custom_functions,
        accounts,
        privilege_sets,
        file_script_triggers,
        external_data_sources,
    })
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Parse `<Options>` and extract script names from `<WindowTriggers>`.
///
/// 実DDR構造:
/// ```xml
/// <Options>
///   <WindowTriggers>
///     <OnFirstWindowOpen><Script id="56" name="OnFirstWindowOpen"/></OnFirstWindowOpen>
///     <OnLastWindowClose><Script id="57" name="OnLastWindowClose"/></OnLastWindowClose>
///   </WindowTriggers>
/// </Options>
/// ```
fn parse_options(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Result<Vec<String>, ParseError> {
    let mut scripts = Vec::new();
    let mut in_window_triggers = false;
    let mut depth: u32 = 1; // <Options> already consumed

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) => {
                if depth == 1 && e.name().as_ref() == b"WindowTriggers" {
                    in_window_triggers = true;
                } else if in_window_triggers && depth == 2 {
                    // <OnFirstWindowOpen>, <OnLastWindowClose>, etc. — event wrapper elements
                    // depth will become 3 inside them
                }
                depth += 1;
            }
            Event::Empty(ref e)
                if in_window_triggers
                    && (e.name().as_ref() == b"Script"
                        || e.name().as_ref() == b"ScriptReference") =>
            {
                if let Ok(name) = get_attr(e, b"name") {
                    if !name.is_empty() {
                        scripts.push(name);
                    }
                }
            }
            Event::End(_) => {
                depth -= 1;
                if depth == 1 {
                    // </WindowTriggers>
                    in_window_triggers = false;
                }
                if depth == 0 {
                    break; // </Options>
                }
            }
            Event::Eof => return Err(ParseError::UnexpectedEof),
            _ => {}
        }
    }

    Ok(scripts)
}

/// Scan forward until `<FMPReport ...>` is found and return the parsed version.
fn find_fmp_report(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Result<FmVersion, ParseError> {
    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"FMPReport" => {
                let version_str = get_attr(e, b"version")?;
                return FmVersion::parse(&version_str);
            }
            Event::Eof => return Err(ParseError::MissingElement("FMPReport".to_owned())),
            _ => {}
        }
    }
}

/// Scan forward until `<File ...>` is found and return the file name.
fn find_file_element(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Result<String, ParseError> {
    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"File" => {
                let name = get_attr(e, b"name").unwrap_or_default();
                return Ok(name);
            }
            Event::Empty(ref e) if e.name().as_ref() == b"File" => {
                let name = get_attr(e, b"name").unwrap_or_default();
                return Ok(name);
            }
            Event::End(_) => {
                // Exited FMPReport without finding File
                return Err(ParseError::MissingElement("File".to_owned()));
            }
            Event::Eof => return Err(ParseError::MissingElement("File".to_owned())),
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// The minimal fixture relative to the workspace root.
    const MINIMAL_XML: &str = include_str!("../../../tests/fixtures/minimal.xml");

    #[test]
    fn parse_minimal_fixture_succeeds() {
        let ddr = parse_ddr(MINIMAL_XML).expect("parse should succeed");
        assert_eq!(ddr.file_name, "TestDB");
        assert_eq!(ddr.fm_version.major, 21);
    }

    #[test]
    fn minimal_has_expected_tables() {
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        assert_eq!(ddr.tables.len(), 1, "expected 1 table");
        assert_eq!(ddr.tables[0].name, "Contact");
        assert_eq!(ddr.tables[0].fields.len(), 1);
        assert_eq!(ddr.tables[0].fields[0].name, "FirstName");
    }

    #[test]
    fn minimal_has_expected_scripts() {
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        assert_eq!(ddr.scripts.len(), 1);
        assert_eq!(ddr.scripts[0].name, "Hello World");
        assert_eq!(ddr.scripts[0].steps.len(), 2);
    }

    #[test]
    fn minimal_has_expected_layouts() {
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        assert_eq!(ddr.layouts.len(), 1);
        assert_eq!(ddr.layouts[0].name, "Contact List");
        assert_eq!(ddr.layouts[0].script_triggers.len(), 1);
    }

    #[test]
    fn minimal_has_expected_value_lists() {
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        assert_eq!(ddr.value_lists.len(), 1);
        assert_eq!(ddr.value_lists[0].name, "Status Values");
        assert_eq!(ddr.value_lists[0].custom_values.len(), 2);
    }

    #[test]
    fn minimal_has_expected_custom_functions() {
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        assert_eq!(ddr.custom_functions.len(), 1);
        assert_eq!(ddr.custom_functions[0].name, "MyFunc");
        assert_eq!(ddr.custom_functions[0].parameters.len(), 2);
    }

    #[test]
    fn minimal_has_expected_accounts() {
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        assert_eq!(ddr.accounts.len(), 1);
        assert_eq!(ddr.accounts[0].name, "Admin");
    }

    #[test]
    fn minimal_has_expected_privilege_sets() {
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        assert_eq!(ddr.privilege_sets.len(), 1);
        assert_eq!(ddr.privilege_sets[0].name, "[Full Access]");
    }

    #[test]
    fn minimal_has_expected_relationships() {
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        assert_eq!(ddr.relationships.len(), 1);
        assert_eq!(ddr.relationships[0].left_table, "Contact");
        assert_eq!(ddr.relationships[0].right_table, "Project");
    }

    #[test]
    fn missing_fmpreport_returns_error() {
        let xml = r#"<?xml version="1.0"?><NotAReport/>"#;
        assert!(parse_ddr(xml).is_err());
    }

    #[test]
    fn empty_catalogs_parse_successfully() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <FMPReport type="Summary" version="21.0v1">
          <File name="Empty">
            <BaseTableCatalog/>
            <RelationshipGraph/>
            <LayoutCatalog/>
            <ScriptCatalog/>
            <ValueListCatalog/>
            <AccountCatalog/>
            <PrivilegesCatalog/>
            <ExtendedPrivilegeCatalog/>
            <CustomFunctionCatalog/>
            <Options/>
          </File>
        </FMPReport>"#;
        let ddr = parse_ddr(xml).expect("empty catalogs should parse OK");
        assert!(ddr.tables.is_empty());
        assert!(ddr.scripts.is_empty());
        assert!(ddr.layouts.is_empty());
        assert!(ddr.value_lists.is_empty());
        assert!(ddr.custom_functions.is_empty());
        assert!(ddr.accounts.is_empty());
        assert!(ddr.privilege_sets.is_empty());
    }
}
