use serde::{Deserialize, Serialize};

use crate::parser::ParseError;

/// Parsed representation of a FileMaker version string such as `"21.0v1"` or `"19.6v2"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FmVersion {
    pub major: u32,
    pub minor: u32,
    /// The patch/variant suffix, e.g. `"v1"`, `"v2"`, or `""`.
    pub patch: String,
}

impl FmVersion {
    /// Parse a version string of the form `"<major>.<minor>v<patch>"`.
    ///
    /// Examples: `"21.0v1"`, `"19.6v2"`, `"14.0v5"`, `"12.0v1"`.
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        // Split on '.' to separate major from the rest.
        let mut parts = s.splitn(2, '.');
        let major_str = parts
            .next()
            .ok_or_else(|| ParseError::InvalidVersion(s.to_owned()))?;
        let rest = parts
            .next()
            .ok_or_else(|| ParseError::InvalidVersion(s.to_owned()))?;

        let major = major_str
            .parse::<u32>()
            .map_err(|_| ParseError::InvalidVersion(s.to_owned()))?;

        // `rest` は "0v1"（旧形式）、"3.2"（X.Y.Z 形式）、"0" のいずれか。
        // 'v' が先にあれば旧形式、'.' があれば X.Y.Z 形式として処理する。
        let (minor_str, patch) = if let Some(v_pos) = rest.to_ascii_lowercase().find('v') {
            (&rest[..v_pos], rest[v_pos..].to_owned())
        } else if let Some(dot_pos) = rest.find('.') {
            (&rest[..dot_pos], rest[dot_pos..].to_owned())
        } else {
            (rest, String::new())
        };

        let minor = minor_str
            .parse::<u32>()
            .map_err(|_| ParseError::InvalidVersion(s.to_owned()))?;

        Ok(FmVersion {
            major,
            minor,
            patch,
        })
    }

    /// Returns `true` for FileMaker 19 and later (Claris era).
    pub fn is_modern(&self) -> bool {
        self.major >= 19
    }

    /// FTS5 support was introduced in FM 16.
    pub fn supports_fts5(&self) -> bool {
        self.major >= 16
    }
}

impl std::fmt::Display for FmVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}{}", self.major, self.minor, self.patch)
    }
}

// ---------------------------------------------------------------------------
// VersionAdapter — normalises version-specific XML differences
// ---------------------------------------------------------------------------

/// Adapts parser behaviour to version-specific XML schema differences.
pub struct VersionAdapter {
    pub version: FmVersion,
}

impl VersionAdapter {
    pub fn new(version: FmVersion) -> Self {
        Self { version }
    }

    /// XML tag that contains fields inside a `<BaseTable>` element.
    pub fn field_catalog_tag(&self) -> &'static str {
        "FieldCatalog"
    }

    /// XML tag for individual script steps inside a `<Script>` element.
    pub fn script_step_tag(&self) -> &'static str {
        "Step"
    }

    /// XML tag for the list of steps inside a `<Script>` element.
    pub fn step_list_tag(&self) -> &'static str {
        "StepList"
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("21.0v1", 21, 0, "v1")]
    #[case("19.6v2", 19, 6, "v2")]
    #[case("14.0v5", 14, 0, "v5")]
    #[case("12.0v1", 12, 0, "v1")]
    #[case("17.0v3", 17, 0, "v3")]
    // X.Y.Z 形式（Claris 世代）
    #[case("20.3.2", 20, 3, ".2")]
    // X.Y 形式（patch なし）
    #[case("21.0", 21, 0, "")]
    fn test_parse_valid(
        #[case] input: &str,
        #[case] major: u32,
        #[case] minor: u32,
        #[case] patch: &str,
    ) {
        let v = FmVersion::parse(input).expect("should parse");
        assert_eq!(v.major, major);
        assert_eq!(v.minor, minor);
        assert_eq!(v.patch, patch);
    }

    #[rstest]
    #[case("")]
    #[case("notaversion")]
    #[case("abc.0v1")]
    #[case("21.xv1")]
    fn test_parse_invalid(#[case] input: &str) {
        assert!(FmVersion::parse(input).is_err());
    }

    #[rstest]
    #[case("21.0v1", true)]
    #[case("19.0v1", true)]
    #[case("18.0v3", false)]
    #[case("14.0v5", false)]
    fn test_is_modern(#[case] input: &str, #[case] expected: bool) {
        let v = FmVersion::parse(input).unwrap();
        assert_eq!(v.is_modern(), expected);
    }

    #[rstest]
    #[case("21.0v1", true)]
    #[case("16.0v1", true)]
    #[case("15.0v1", false)]
    #[case("14.0v5", false)]
    fn test_supports_fts5(#[case] input: &str, #[case] expected: bool) {
        let v = FmVersion::parse(input).unwrap();
        assert_eq!(v.supports_fts5(), expected);
    }

    #[test]
    fn test_display() {
        let v = FmVersion::parse("21.0v1").unwrap();
        assert_eq!(v.to_string(), "21.0v1");
    }

    #[test]
    fn test_display_xyz_format() {
        let v = FmVersion::parse("20.3.2").unwrap();
        assert_eq!(v.to_string(), "20.3.2");
    }

    #[test]
    fn version_adapter_tags() {
        let v = FmVersion::parse("21.0v1").unwrap();
        let adapter = VersionAdapter::new(v);
        assert_eq!(adapter.field_catalog_tag(), "FieldCatalog");
        assert_eq!(adapter.script_step_tag(), "Step");
        assert_eq!(adapter.step_list_tag(), "StepList");
    }
}
