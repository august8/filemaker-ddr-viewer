//! 壊れた参照（存在しないスクリプトへの Perform Script / ScriptTrigger）の検出。

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::parser::models::DdrFile;

// ---------------------------------------------------------------------------
// 公開型
// ---------------------------------------------------------------------------

/// 壊れた参照の種別。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BrokenRefKind {
    /// Perform Script ステップが存在しないスクリプトを参照している。
    PerformScript,
    /// ScriptTrigger が存在しないスクリプトを参照している。
    ScriptTrigger,
}

/// 単一の壊れた参照。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokenRef {
    pub kind: BrokenRefKind,
    /// 参照元スクリプト名（PerformScript の場合）または参照元レイアウト名（ScriptTrigger の場合）。
    pub source_name: String,
    /// 参照しようとしているスクリプト名。
    pub target_script_name: String,
}

// ---------------------------------------------------------------------------
// ロジック
// ---------------------------------------------------------------------------

/// DDR ファイル内の壊れた参照をすべて検出して返す。
///
/// 同一ファイル内の参照のみを対象とする（外部ファイル参照は除外）。
pub fn find_broken_refs(ddr: &DdrFile) -> Vec<BrokenRef> {
    // 存在するスクリプト名のセット（区切り線 "-" は除外）
    let known: HashSet<&str> = ddr
        .scripts
        .iter()
        .filter(|s| s.name != "-")
        .map(|s| s.name.as_str())
        .collect();

    let mut result = Vec::new();

    // Perform Script ステップの確認（区切り線スクリプト "-" からの参照は除外）
    for script in ddr.scripts.iter().filter(|s| s.name != "-") {
        for step in &script.steps {
            // 無効（コメントアウト）ステップは除外
            if !step.enabled {
                continue;
            }
            let Some(ref script_ref) = step.script_ref else {
                continue;
            };
            // 外部ファイル参照・空参照・FileMaker内部プレースホルダー（<不明>等）は除外
            if !script_ref.file_name.is_empty()
                || script_ref.name.is_empty()
                || script_ref.name.starts_with('<')
            {
                continue;
            }
            if !known.contains(script_ref.name.as_str()) {
                result.push(BrokenRef {
                    kind: BrokenRefKind::PerformScript,
                    source_name: script.name.clone(),
                    target_script_name: script_ref.name.clone(),
                });
            }
        }
    }

    // ScriptTrigger の確認（区切り線レイアウト "-" からの参照は除外）
    for layout in ddr.layouts.iter().filter(|l| l.name != "-") {
        for trigger in &layout.script_triggers {
            if !trigger.file_name.is_empty() || trigger.script_name.is_empty() {
                continue;
            }
            if !known.contains(trigger.script_name.as_str()) {
                result.push(BrokenRef {
                    kind: BrokenRefKind::ScriptTrigger,
                    source_name: layout.name.clone(),
                    target_script_name: trigger.script_name.clone(),
                });
            }
        }
    }

    result
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
    fn minimal_has_known_broken_refs() {
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        // minimal.xml には意図的に壊れた参照が含まれている:
        //   - "Hello World" → "Another Script"（PerformScript、存在しない）
        //   - Layout "Contact List" → "My Script"（ScriptTrigger、存在しない）
        let refs = find_broken_refs(&ddr);
        assert!(
            refs.iter().any(|r| r.kind == BrokenRefKind::PerformScript
                && r.target_script_name == "Another Script"),
            "expected broken PerformScript ref: {refs:?}"
        );
        assert!(
            refs.iter()
                .any(|r| r.kind == BrokenRefKind::ScriptTrigger
                    && r.target_script_name == "My Script"),
            "expected broken ScriptTrigger ref: {refs:?}"
        );
    }

    #[test]
    fn detects_broken_perform_script() {
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
                name: "Caller".into(),
                run_with_full_access: false,
                steps: vec![ScriptStep {
                    step_id: 89,
                    name: "Perform Script".into(),
                    enabled: true,
                    script_ref: Some(ScriptRef {
                        name: "NonExistent".into(),
                        file_name: "".into(),
                    }),
                    calculation: None,
                    step_text: None,
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
        };

        let refs = find_broken_refs(&ddr);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].kind, BrokenRefKind::PerformScript);
        assert_eq!(refs[0].source_name, "Caller");
        assert_eq!(refs[0].target_script_name, "NonExistent");
    }

    #[test]
    fn external_file_refs_ignored() {
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
                name: "Caller".into(),
                run_with_full_access: false,
                steps: vec![ScriptStep {
                    step_id: 89,
                    name: "Perform Script".into(),
                    enabled: true,
                    script_ref: Some(ScriptRef {
                        name: "ExternalScript".into(),
                        file_name: "OtherFile".into(), // 外部ファイル
                    }),
                    calculation: None,
                    step_text: None,
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
        };

        let refs = find_broken_refs(&ddr);
        assert!(refs.is_empty(), "external refs should be ignored");
    }

    #[test]
    fn detects_broken_script_trigger() {
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
            scripts: vec![],
            layouts: vec![Layout {
                id: LayoutId(1),
                name: "MainLayout".into(),
                table_occurrence_name: None,
                script_triggers: vec![ScriptTrigger {
                    event: "OnRecordLoad".into(),
                    script_name: "MissingScript".into(),
                    file_name: "".into(),
                }],
                button_script_refs: vec![],
                field_refs: vec![],
                layout_objects: vec![],
            }],
            relationships: vec![],
            value_lists: vec![],
            custom_functions: vec![],
            accounts: vec![],
            privilege_sets: vec![],
            table_occurrences: vec![],
            file_script_triggers: vec![],
        };

        let refs = find_broken_refs(&ddr);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].kind, BrokenRefKind::ScriptTrigger);
        assert_eq!(refs[0].source_name, "MainLayout");
        assert_eq!(refs[0].target_script_name, "MissingScript");
    }
}
