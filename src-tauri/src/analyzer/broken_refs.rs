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
    /// Set Field 等のステップが存在しないフィールドを参照している。
    BrokenFieldRef,
    /// Go to Layout 等のステップが存在しないレイアウトを参照している。
    BrokenLayoutRef,
    /// step_text に `<不明>` が含まれており、他の broken ref 種別で未検出のケース。
    /// 例: ファイルを開く・Perform Script の外部参照先不明 等。
    UnknownRef,
    /// レイアウト上のフィールドオブジェクトが存在しないフィールドを参照している
    /// （フィールド削除済み・外部データソース切断など）。
    BrokenFieldPlacement,
}

/// 単一の壊れた参照。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokenRef {
    pub kind: BrokenRefKind,
    /// 参照元スクリプト名（PerformScript の場合）または参照元レイアウト名（ScriptTrigger の場合）。
    pub source_name: String,
    /// 参照しようとしているスクリプト名。
    pub target_script_name: String,
    /// 参照元要素の SQLite DB ID（コマンド層で解決される。解決不能な場合は None）。
    pub source_id: Option<i64>,
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
                    source_id: None,
                });
            }
        }

        // 壊れたフィールド参照（Set Field 等）、レイアウト参照、
        // および step_text に <不明> を含む未分類の参照切れを検出
        for step in script.steps.iter().filter(|s| s.enabled) {
            if step.broken_field_table.is_some() {
                let target = step.step_text.as_deref().unwrap_or("(broken field ref)");
                result.push(BrokenRef {
                    kind: BrokenRefKind::BrokenFieldRef,
                    source_name: script.name.clone(),
                    target_script_name: target.to_string(),
                    source_id: None,
                });
            } else if step.has_broken_layout_ref {
                let target = step.step_text.as_deref().unwrap_or("(broken layout ref)");
                result.push(BrokenRef {
                    kind: BrokenRefKind::BrokenLayoutRef,
                    source_name: script.name.clone(),
                    target_script_name: target.to_string(),
                    source_id: None,
                });
            } else {
                // step_text に <不明> が含まれるケース（ファイルを開く・Perform Script 外部参照等）
                let is_script_ref_unknown = step
                    .script_ref
                    .as_ref()
                    .is_some_and(|r| r.name.starts_with('<'));
                let has_unknown_in_text = step
                    .step_text
                    .as_deref()
                    .is_some_and(|t| t.contains("<不明>"));
                if has_unknown_in_text || is_script_ref_unknown {
                    let target = step.step_text.clone().unwrap_or_else(|| step.name.clone());
                    result.push(BrokenRef {
                        kind: BrokenRefKind::UnknownRef,
                        source_name: script.name.clone(),
                        target_script_name: target,
                        source_id: None,
                    });
                }
            }
        }
    }

    // ScriptTrigger と BrokenFieldPlacement の確認（区切り線レイアウト "-" は除外）
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
                    source_id: None,
                });
            }
        }
        for obj in &layout.layout_objects {
            if obj.object_type == "Field" {
                let is_broken = obj
                    .field_table_occurrence
                    .as_deref()
                    .is_some_and(|s| s.is_empty())
                    || obj.field_name.as_deref().is_some_and(|s| s.is_empty());
                if is_broken {
                    result.push(BrokenRef {
                        kind: BrokenRefKind::BrokenFieldPlacement,
                        source_name: layout.name.clone(),
                        target_script_name: format!(
                            "フィールドが見つかりません #{}",
                            obj.object_key
                        ),
                        source_id: None,
                    });
                }
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
    fn detects_broken_set_field_step() {
        use crate::parser::models::*;
        use crate::parser::version::FmVersion;

        let ddr = DdrFile {
            file_name: "Test".into(),
            fm_version: FmVersion {
                major: 22,
                minor: 0,
                patch: "v1".into(),
            },
            tables: vec![],
            scripts: vec![Script {
                id: ScriptId(1),
                name: "MainScript".into(),
                run_with_full_access: false,
                steps: vec![ScriptStep {
                    step_id: 76,
                    name: "フィールド設定".into(),
                    enabled: true,
                    script_ref: None,
                    calculation: None,
                    step_text: Some(
                        "フィールド設定 [ BaseFile::<フィールドが見つかりません> ]".into(),
                    ),
                    broken_field_table: Some("BaseFile".into()),
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

        let refs = find_broken_refs(&ddr);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].kind, BrokenRefKind::BrokenFieldRef);
        assert_eq!(refs[0].source_name, "MainScript");
        assert!(refs[0]
            .target_script_name
            .contains("フィールドが見つかりません"));
    }

    #[test]
    fn detects_broken_go_to_layout_step() {
        use crate::parser::models::*;
        use crate::parser::version::FmVersion;

        let ddr = DdrFile {
            file_name: "Test".into(),
            fm_version: FmVersion {
                major: 22,
                minor: 0,
                patch: "v1".into(),
            },
            tables: vec![],
            scripts: vec![Script {
                id: ScriptId(1),
                name: "NavScript".into(),
                run_with_full_access: false,
                steps: vec![ScriptStep {
                    step_id: 6,
                    name: "レイアウト切り替え".into(),
                    enabled: true,
                    script_ref: None,
                    calculation: None,
                    step_text: Some("レイアウト切り替え [ <不明> ]".into()),
                    broken_field_table: None,
                    has_broken_layout_ref: true,
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

        let refs = find_broken_refs(&ddr);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].kind, BrokenRefKind::BrokenLayoutRef);
        assert_eq!(refs[0].source_name, "NavScript");
        assert!(refs[0].target_script_name.contains("不明"));
    }

    #[test]
    fn disabled_broken_step_is_ignored() {
        use crate::parser::models::*;
        use crate::parser::version::FmVersion;

        let ddr = DdrFile {
            file_name: "Test".into(),
            fm_version: FmVersion {
                major: 22,
                minor: 0,
                patch: "v1".into(),
            },
            tables: vec![],
            scripts: vec![Script {
                id: ScriptId(1),
                name: "S".into(),
                run_with_full_access: false,
                steps: vec![ScriptStep {
                    step_id: 76,
                    name: "フィールド設定".into(),
                    enabled: false, // disabled → 除外
                    script_ref: None,
                    calculation: None,
                    step_text: None,
                    broken_field_table: Some("BaseFile".into()),
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

        let refs = find_broken_refs(&ddr);
        assert!(refs.is_empty(), "disabled steps should be ignored");
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
            external_data_sources: vec![],
        };

        let refs = find_broken_refs(&ddr);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].kind, BrokenRefKind::ScriptTrigger);
        assert_eq!(refs[0].source_name, "MainLayout");
        assert_eq!(refs[0].target_script_name, "MissingScript");
    }

    #[test]
    fn detects_open_file_unknown_step() {
        use crate::parser::models::*;
        use crate::parser::version::FmVersion;

        let ddr = DdrFile {
            file_name: "Test".into(),
            fm_version: FmVersion {
                major: 22,
                minor: 0,
                patch: "v1".into(),
            },
            tables: vec![],
            scripts: vec![Script {
                id: ScriptId(1),
                name: "物件管理を開く！".into(),
                run_with_full_access: false,
                steps: vec![ScriptStep {
                    step_id: 148,
                    name: "ファイルを開く".into(),
                    enabled: true,
                    script_ref: None,
                    calculation: None,
                    step_text: Some("ファイルを開く [ <不明> ]".into()),
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

        let refs = find_broken_refs(&ddr);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].kind, BrokenRefKind::UnknownRef);
        assert_eq!(refs[0].source_name, "物件管理を開く！");
        assert!(refs[0].target_script_name.contains("不明"));
    }

    #[test]
    fn detects_perform_script_with_angle_bracket_ref() {
        use crate::parser::models::*;
        use crate::parser::version::FmVersion;

        let ddr = DdrFile {
            file_name: "Test".into(),
            fm_version: FmVersion {
                major: 22,
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
                        name: "<不明>".into(),
                        file_name: "".into(),
                    }),
                    calculation: None,
                    step_text: Some("Perform Script [ \"<不明>\"; off ]".into()),
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

        let refs = find_broken_refs(&ddr);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].kind, BrokenRefKind::UnknownRef);
        assert_eq!(refs[0].source_name, "Caller");
    }

    #[test]
    fn no_duplicate_for_broken_layout_ref() {
        use crate::parser::models::*;
        use crate::parser::version::FmVersion;

        // has_broken_layout_ref=true のステップは BrokenLayoutRef のみ（UnknownRef との重複なし）
        let ddr = DdrFile {
            file_name: "Test".into(),
            fm_version: FmVersion {
                major: 22,
                minor: 0,
                patch: "v1".into(),
            },
            tables: vec![],
            scripts: vec![Script {
                id: ScriptId(1),
                name: "NavScript".into(),
                run_with_full_access: false,
                steps: vec![ScriptStep {
                    step_id: 6,
                    name: "レイアウト切り替え".into(),
                    enabled: true,
                    script_ref: None,
                    calculation: None,
                    step_text: Some("レイアウト切り替え [ <不明> ]".into()),
                    broken_field_table: None,
                    has_broken_layout_ref: true,
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

        let refs = find_broken_refs(&ddr);
        assert_eq!(
            refs.len(),
            1,
            "BrokenLayoutRef のみ、UnknownRef との重複なし: {refs:?}"
        );
        assert_eq!(refs[0].kind, BrokenRefKind::BrokenLayoutRef);
    }

    #[test]
    fn detects_broken_layout_field_placement() {
        use crate::parser::models::*;
        use crate::parser::version::FmVersion;

        let ddr = DdrFile {
            file_name: "Test".into(),
            fm_version: FmVersion {
                major: 22,
                minor: 0,
                patch: "v1".into(),
            },
            tables: vec![],
            scripts: vec![],
            layouts: vec![Layout {
                id: LayoutId(1),
                name: "CustomerLayout".into(),
                table_occurrence_name: None,
                script_triggers: vec![],
                button_script_refs: vec![],
                field_refs: vec![],
                layout_objects: vec![LayoutObject {
                    object_type: "Field".into(),
                    object_key: 7,
                    object_name: None,
                    button_label: None,
                    field_table_occurrence: Some("".into()),
                    field_name: Some("".into()),
                    tooltip: None,
                    hide_condition: None,
                    bounds: None,
                    conditional_formats: vec![],
                }],
            }],
            relationships: vec![],
            value_lists: vec![],
            custom_functions: vec![],
            accounts: vec![],
            privilege_sets: vec![],
            table_occurrences: vec![],
            file_script_triggers: vec![],
            external_data_sources: vec![],
        };

        let refs = find_broken_refs(&ddr);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].kind, BrokenRefKind::BrokenFieldPlacement);
        assert_eq!(refs[0].source_name, "CustomerLayout");
        assert!(
            refs[0]
                .target_script_name
                .contains("フィールドが見つかりません"),
            "expected description, got: {}",
            refs[0].target_script_name
        );
        assert!(
            refs[0].target_script_name.contains("#7"),
            "expected object key #7, got: {}",
            refs[0].target_script_name
        );
    }

    #[test]
    fn non_field_objects_not_detected_as_broken_placement() {
        use crate::parser::models::*;
        use crate::parser::version::FmVersion;

        let ddr = DdrFile {
            file_name: "Test".into(),
            fm_version: FmVersion {
                major: 22,
                minor: 0,
                patch: "v1".into(),
            },
            tables: vec![],
            scripts: vec![],
            layouts: vec![Layout {
                id: LayoutId(1),
                name: "CustomerLayout".into(),
                table_occurrence_name: None,
                script_triggers: vec![],
                button_script_refs: vec![],
                field_refs: vec![],
                layout_objects: vec![LayoutObject {
                    object_type: "Text".into(), // Text object, not Field
                    object_key: 3,
                    object_name: None,
                    button_label: None,
                    field_table_occurrence: None,
                    field_name: None,
                    tooltip: None,
                    hide_condition: None,
                    bounds: None,
                    conditional_formats: vec![],
                }],
            }],
            relationships: vec![],
            value_lists: vec![],
            custom_functions: vec![],
            accounts: vec![],
            privilege_sets: vec![],
            table_occurrences: vec![],
            file_script_triggers: vec![],
            external_data_sources: vec![],
        };

        let refs = find_broken_refs(&ddr);
        assert!(
            refs.is_empty(),
            "Text objects should not be detected as broken: {refs:?}"
        );
    }

    #[test]
    fn disabled_perform_script_is_ignored() {
        use crate::parser::models::*;
        use crate::parser::version::FmVersion;

        let ddr = DdrFile {
            file_name: "Test".into(),
            fm_version: FmVersion {
                major: 22,
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
                    enabled: false, // disabled → 除外
                    script_ref: Some(ScriptRef {
                        name: "NonExistent".into(),
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

        let refs = find_broken_refs(&ddr);
        assert!(refs.is_empty(), "disabled PerformScript should be ignored");
    }
}
