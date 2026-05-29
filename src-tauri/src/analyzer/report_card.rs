//! システム健全性レポートの生成。
//!
//! 壊れた参照・未使用スクリプト等の集計結果を `ReportCard` にまとめる。

use serde::{Deserialize, Serialize};

use crate::parser::models::DdrFile;

use super::{broken_refs::find_broken_refs, orphans::find_orphan_scripts};

// ---------------------------------------------------------------------------
// 公開型
// ---------------------------------------------------------------------------

/// 問題の深刻度。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

/// 単一の問題レポート項目。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportIssue {
    pub severity: Severity,
    pub category: String,
    pub message: String,
    /// 遷移先の要素種別（"script" / "layout"）。遷移不要な項目は None。
    pub element_kind: Option<String>,
    /// 遷移先の要素名。フロントエンドでリストから ID を引く。
    pub element_name: Option<String>,
}

/// システム健全性レポート全体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportCard {
    pub issues: Vec<ReportIssue>,
    /// エラー件数
    pub error_count: usize,
    /// 警告件数
    pub warning_count: usize,
    /// 情報件数
    pub info_count: usize,
}

impl ReportCard {
    fn new(issues: Vec<ReportIssue>) -> Self {
        let error_count = issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .count();
        let warning_count = issues
            .iter()
            .filter(|i| i.severity == Severity::Warning)
            .count();
        let info_count = issues
            .iter()
            .filter(|i| i.severity == Severity::Info)
            .count();
        ReportCard {
            issues,
            error_count,
            warning_count,
            info_count,
        }
    }

    /// 問題が 0 件なら true。
    pub fn is_healthy(&self) -> bool {
        self.error_count == 0 && self.warning_count == 0
    }
}

// ---------------------------------------------------------------------------
// ロジック
// ---------------------------------------------------------------------------

/// DDR ファイルのシステム健全性レポートを生成する。
pub fn generate_report_card(ddr: &DdrFile) -> ReportCard {
    let mut issues = Vec::new();

    // 壊れた参照
    let broken = find_broken_refs(ddr);
    for r in &broken {
        let (element_kind, element_name) = match r.kind {
            crate::analyzer::broken_refs::BrokenRefKind::PerformScript => {
                (Some("script".into()), Some(r.source_name.clone()))
            }
            crate::analyzer::broken_refs::BrokenRefKind::ScriptTrigger => {
                (Some("layout".into()), Some(r.source_name.clone()))
            }
            crate::analyzer::broken_refs::BrokenRefKind::BrokenFieldRef
            | crate::analyzer::broken_refs::BrokenRefKind::BrokenLayoutRef
            | crate::analyzer::broken_refs::BrokenRefKind::UnknownRef => {
                (Some("script".into()), Some(r.source_name.clone()))
            }
            crate::analyzer::broken_refs::BrokenRefKind::BrokenFieldPlacement => {
                (Some("layout".into()), Some(r.source_name.clone()))
            }
        };
        issues.push(ReportIssue {
            severity: Severity::Error,
            category: "broken_ref".into(),
            message: format!(
                "[{:?}] '{}' → '{}'",
                r.kind, r.source_name, r.target_script_name
            ),
            element_kind,
            element_name,
        });
    }

    // 未使用スクリプト
    let orphans = find_orphan_scripts(ddr);
    for o in &orphans {
        issues.push(ReportIssue {
            severity: Severity::Warning,
            category: "orphan_script".into(),
            message: format!("Script '{}' is never called", o.script_name),
            element_kind: Some("script".into()),
            element_name: Some(o.script_name.clone()),
        });
    }

    // スクリプト数の情報
    issues.push(ReportIssue {
        severity: Severity::Info,
        category: "stats".into(),
        message: format!(
            "Project has {} scripts, {} tables, {} layouts",
            ddr.scripts.len(),
            ddr.tables.len(),
            ddr.layouts.len()
        ),
        element_kind: None,
        element_name: None,
    });

    ReportCard::new(issues)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_ddr;

    const MINIMAL_XML: &str = include_str!("../../../tests/fixtures/minimal.xml");

    #[test]
    fn report_card_has_at_least_one_info() {
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let card = generate_report_card(&ddr);
        assert!(card.info_count >= 1);
    }

    #[test]
    fn report_card_counts_match_issues() {
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let card = generate_report_card(&ddr);
        let total = card.error_count + card.warning_count + card.info_count;
        assert_eq!(total, card.issues.len());
    }

    #[test]
    fn severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
    }

    #[test]
    fn healthy_db_has_no_errors() {
        use crate::parser::models::*;
        use crate::parser::version::FmVersion;

        let ddr = DdrFile {
            file_name: "Clean".into(),
            fm_version: FmVersion {
                major: 21,
                minor: 0,
                patch: "v1".into(),
            },
            tables: vec![],
            scripts: vec![],
            layouts: vec![],
            relationships: vec![],
            value_lists: vec![],
            custom_functions: vec![],
            accounts: vec![],
            privilege_sets: vec![],
            table_occurrences: vec![],
            file_script_triggers: vec![],
            external_data_sources: vec![],
        };
        let card = generate_report_card(&ddr);
        assert_eq!(card.error_count, 0);
        assert!(card.is_healthy());
    }

    #[test]
    fn broken_ref_increments_error_count() {
        use crate::parser::models::*;
        use crate::parser::version::FmVersion;

        let ddr = DdrFile {
            file_name: "Test".into(),
            fm_version: FmVersion {
                major: 21,
                minor: 0,
                patch: "v1".into(),
            },
            tables: vec![],
            scripts: vec![Script {
                id: ScriptId(1),
                name: "A".into(),
                run_with_full_access: false,
                steps: vec![ScriptStep {
                    step_id: 89,
                    name: "Perform Script".into(),
                    enabled: true,
                    script_ref: Some(ScriptRef {
                        name: "Ghost".into(),
                        file_name: "".into(),
                    }),
                    calculation: None,
                    step_text: None,
                    broken_field_table: None,
                    has_broken_layout_ref: false,
                }],
            }],
            layouts: vec![],
            relationships: vec![],
            value_lists: vec![],
            custom_functions: vec![],
            accounts: vec![],
            privilege_sets: vec![],
            table_occurrences: vec![],
            file_script_triggers: vec![],
            external_data_sources: vec![],
        };
        let card = generate_report_card(&ddr);
        assert!(card.error_count >= 1);
        assert!(!card.is_healthy());
    }
}
