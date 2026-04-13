//! 未使用スクリプト（どこからも呼ばれていないスクリプト）の検出。

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::parser::models::{DdrFile, ScriptId};

// ---------------------------------------------------------------------------
// 公開型
// ---------------------------------------------------------------------------

/// 未使用スクリプトのエントリ。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrphanScript {
    pub script_id: u64,
    pub script_name: String,
}

// ---------------------------------------------------------------------------
// ロジック
// ---------------------------------------------------------------------------

/// どこからも呼ばれていないスクリプトを検出して返す。
///
/// 「呼ばれている」と判定する条件:
/// 1. 同一ファイル内の別スクリプトの Perform Script ステップで参照されている
/// 2. 同一ファイル内のレイアウトの ScriptTrigger で参照されている
/// 3. 同一ファイル内のレイアウトのボタン・オブジェクトで参照されている
///
/// 外部ファイルからの参照は追跡不可能なため、除外。
/// 名前が "-" のスクリプトは FileMaker の区切り線扱いのため除外。
pub fn find_orphan_scripts(ddr: &DdrFile) -> Vec<OrphanScript> {
    // 呼ばれているスクリプト名のセット
    let mut called: HashSet<&str> = HashSet::new();

    // Perform Script ステップによる参照
    for script in &ddr.scripts {
        for step in &script.steps {
            let Some(ref script_ref) = step.script_ref else {
                continue;
            };
            if script_ref.file_name.is_empty() {
                called.insert(script_ref.name.as_str());
            }
        }
    }

    // ScriptTrigger による参照
    for layout in &ddr.layouts {
        for trigger in &layout.script_triggers {
            if trigger.file_name.is_empty() {
                called.insert(trigger.script_name.as_str());
            }
        }
        // ボタン・オブジェクトからのスクリプト参照
        for btn_script in &layout.button_script_refs {
            called.insert(btn_script.as_str());
        }
    }

    // ファイルオプション > WindowTriggers（OnFirstWindowOpen / OnLastWindowClose 等）
    for name in &ddr.file_script_triggers {
        called.insert(name.as_str());
    }

    // 呼ばれていないスクリプトを収集（区切り線 "-" は除外）
    ddr.scripts
        .iter()
        .filter(|s| s.name != "-" && !called.contains(s.name.as_str()))
        .map(|s| OrphanScript {
            script_id: s.id.0,
            script_name: s.name.clone(),
        })
        .collect()
}

/// script_id で特定スクリプトの呼び出し元スクリプト名一覧を返す。
pub fn find_callers_by_name<'a>(ddr: &'a DdrFile, target_name: &str) -> Vec<&'a str> {
    let mut callers = Vec::new();
    for script in &ddr.scripts {
        for step in &script.steps {
            let Some(ref script_ref) = step.script_ref else {
                continue;
            };
            if script_ref.file_name.is_empty() && script_ref.name == target_name {
                callers.push(script.name.as_str());
                break; // 同一スクリプトからの重複登録を防ぐ
            }
        }
    }
    callers
}

/// `ScriptId` でスクリプトの呼び出し元スクリプト ID 一覧を返す。
pub fn find_callers_of(ddr: &DdrFile, target_id: ScriptId) -> Vec<ScriptId> {
    let target_name = ddr
        .scripts
        .iter()
        .find(|s| s.id == target_id)
        .map(|s| s.name.as_str())
        .unwrap_or("");
    if target_name.is_empty() {
        return vec![];
    }

    let caller_names: HashSet<&str> = find_callers_by_name(ddr, target_name).into_iter().collect();

    ddr.scripts
        .iter()
        .filter(|s| caller_names.contains(s.name.as_str()))
        .map(|s| s.id)
        .collect()
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
    fn minimal_orphans_count_is_reasonable() {
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let orphans = find_orphan_scripts(&ddr);
        // 合計スクリプト数以下
        assert!(orphans.len() <= ddr.scripts.len());
    }

    #[test]
    fn called_script_is_not_orphan() {
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
            scripts: vec![
                Script {
                    id: ScriptId(1),
                    name: "Caller".into(),
                    run_with_full_access: false,
                    steps: vec![ScriptStep {
                        step_id: 89,
                        name: "Perform Script".into(),
                        enabled: true,
                        script_ref: Some(ScriptRef {
                            name: "Callee".into(),
                            file_name: "".into(),
                        }),
                        calculation: None,
                        step_text: None,
                    }],
                },
                Script {
                    id: ScriptId(2),
                    name: "Callee".into(),
                    run_with_full_access: false,
                    steps: vec![],
                },
            ],
            layouts: vec![],
            relationships: vec![],
            value_lists: vec![],
            custom_functions: vec![],
            accounts: vec![],
            privilege_sets: vec![],
            table_occurrences: vec![],
            file_script_triggers: vec![],
        };

        let orphans = find_orphan_scripts(&ddr);
        // "Callee" は呼ばれているので孤立ではない
        assert!(!orphans.iter().any(|o| o.script_name == "Callee"));
        // "Caller" はどこからも呼ばれていないので孤立
        assert!(orphans.iter().any(|o| o.script_name == "Caller"));
    }

    #[test]
    fn trigger_called_script_is_not_orphan() {
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
                name: "OnLoad".into(),
                run_with_full_access: false,
                steps: vec![],
            }],
            layouts: vec![Layout {
                id: LayoutId(1),
                name: "Main".into(),
                table_occurrence_name: None,
                script_triggers: vec![ScriptTrigger {
                    event: "OnRecordLoad".into(),
                    script_name: "OnLoad".into(),
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

        let orphans = find_orphan_scripts(&ddr);
        assert!(!orphans.iter().any(|o| o.script_name == "OnLoad"));
    }

    #[test]
    fn find_callers_of_returns_correct_ids() {
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
            scripts: vec![
                Script {
                    id: ScriptId(1),
                    name: "A".into(),
                    run_with_full_access: false,
                    steps: vec![ScriptStep {
                        step_id: 89,
                        name: "Perform Script".into(),
                        enabled: true,
                        script_ref: Some(ScriptRef {
                            name: "B".into(),
                            file_name: "".into(),
                        }),
                        calculation: None,
                        step_text: None,
                    }],
                },
                Script {
                    id: ScriptId(2),
                    name: "B".into(),
                    run_with_full_access: false,
                    steps: vec![],
                },
            ],
            layouts: vec![],
            relationships: vec![],
            value_lists: vec![],
            custom_functions: vec![],
            accounts: vec![],
            privilege_sets: vec![],
            table_occurrences: vec![],
            file_script_triggers: vec![],
        };

        let callers = find_callers_of(&ddr, ScriptId(2));
        assert_eq!(callers, vec![ScriptId(1)]);
    }
}
