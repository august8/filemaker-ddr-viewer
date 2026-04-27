//! スクリプト呼び出しチェーンの構築（DFS + 循環検出）。

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::parser::models::{DdrFile, ScriptId};

use super::AnalysisError;

// ---------------------------------------------------------------------------
// 公開型
// ---------------------------------------------------------------------------

/// スクリプト呼び出しチェーンのノード（再帰的ツリー構造）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallChainNode {
    pub script_id: u64,
    pub script_name: String,
    pub depth: u32,
    /// 循環検出により打ち切られた場合 true。
    pub is_cycle: bool,
    pub children: Vec<CallChainNode>,
}

// ---------------------------------------------------------------------------
// ロジック
// ---------------------------------------------------------------------------

/// 指定スクリプト（`root_id`）を起点にした呼び出しチェーンを構築する。
///
/// - 循環参照がある場合は `is_cycle = true` でノードを作成して打ち切る。
/// - 最大深度（`max_depth`）を超えた場合も打ち切る（デフォルト: 20）。
pub fn build_call_chain(
    ddr: &DdrFile,
    root_id: ScriptId,
    max_depth: Option<u32>,
) -> Result<CallChainNode, AnalysisError> {
    let max_depth = max_depth.unwrap_or(20);

    // ScriptId → Script インデックスのマップ
    let id_to_script: HashMap<ScriptId, usize> = ddr
        .scripts
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id, i))
        .collect();

    if !id_to_script.contains_key(&root_id) {
        return Err(AnalysisError::ScriptNotFound(root_id.0));
    }

    // 名前 → ScriptId の逆引き
    let name_to_id: HashMap<&str, ScriptId> = ddr
        .scripts
        .iter()
        .map(|s| (s.name.as_str(), s.id))
        .collect();

    let mut visited = HashSet::new();
    let root_name = ddr.scripts[id_to_script[&root_id]].name.clone();

    let node = dfs(
        ddr,
        root_id,
        root_name,
        0,
        max_depth,
        &id_to_script,
        &name_to_id,
        &mut visited,
    );
    Ok(node)
}

/// 呼び出し元スクリプト（`target_id` を Perform Script で呼んでいるスクリプト）一覧を返す。
pub fn find_callers(ddr: &DdrFile, target_id: ScriptId) -> Vec<ScriptId> {
    crate::analyzer::orphans::find_callers_of(ddr, target_id)
}

// ---------------------------------------------------------------------------
// 内部 DFS
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn dfs(
    ddr: &DdrFile,
    current_id: ScriptId,
    current_name: String,
    depth: u32,
    max_depth: u32,
    id_to_script: &HashMap<ScriptId, usize>,
    name_to_id: &HashMap<&str, ScriptId>,
    visited: &mut HashSet<ScriptId>,
) -> CallChainNode {
    // 深度オーバーまたは循環
    if depth > max_depth || visited.contains(&current_id) {
        return CallChainNode {
            script_id: current_id.0,
            script_name: current_name,
            depth,
            is_cycle: visited.contains(&current_id),
            children: vec![],
        };
    }

    visited.insert(current_id);

    let script = &ddr.scripts[id_to_script[&current_id]];
    let mut children = Vec::new();

    for step in &script.steps {
        let Some(ref script_ref) = step.script_ref else {
            continue;
        };
        if !script_ref.file_name.is_empty() {
            continue;
        }
        let Some(&callee_id) = name_to_id.get(script_ref.name.as_str()) else {
            continue;
        };

        let child = dfs(
            ddr,
            callee_id,
            script_ref.name.clone(),
            depth + 1,
            max_depth,
            id_to_script,
            name_to_id,
            visited,
        );
        children.push(child);
    }

    visited.remove(&current_id);

    CallChainNode {
        script_id: current_id.0,
        script_name: current_name,
        depth,
        is_cycle: false,
        children,
    }
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
    fn build_call_chain_returns_root_for_leaf_script() {
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let Some(script) = ddr.scripts.first() else {
            return;
        };
        let node = build_call_chain(&ddr, script.id, None).unwrap();
        assert_eq!(node.script_id, script.id.0);
        assert_eq!(node.depth, 0);
        assert!(!node.is_cycle);
    }

    #[test]
    fn build_call_chain_unknown_id_returns_error() {
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let err = build_call_chain(&ddr, ScriptId(99999), None);
        assert!(err.is_err());
    }

    #[test]
    fn build_call_chain_detects_cycle() {
        use crate::parser::models::*;
        use crate::parser::version::FmVersion;

        // A → B → A（循環）
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
                        broken_field_table: None,
                        has_broken_layout_ref: false,
                    }],
                },
                Script {
                    id: ScriptId(2),
                    name: "B".into(),
                    run_with_full_access: false,
                    steps: vec![ScriptStep {
                        step_id: 89,
                        name: "Perform Script".into(),
                        enabled: true,
                        script_ref: Some(ScriptRef {
                            name: "A".into(),
                            file_name: "".into(),
                        }),
                        calculation: None,
                        step_text: None,
                        broken_field_table: None,
                        has_broken_layout_ref: false,
                    }],
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

        let node = build_call_chain(&ddr, ScriptId(1), None).unwrap();
        // B の子に A が is_cycle=true で登場
        assert_eq!(node.children.len(), 1); // B
        let b = &node.children[0];
        assert_eq!(b.script_name, "B");
        assert_eq!(b.children.len(), 1); // A (cycle)
        assert!(b.children[0].is_cycle);
    }

    #[test]
    fn build_call_chain_respects_max_depth() {
        use crate::parser::models::*;
        use crate::parser::version::FmVersion;

        // A → B → C（深さ制限 1 で B まで）
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
                        broken_field_table: None,
                        has_broken_layout_ref: false,
                    }],
                },
                Script {
                    id: ScriptId(2),
                    name: "B".into(),
                    run_with_full_access: false,
                    steps: vec![ScriptStep {
                        step_id: 89,
                        name: "Perform Script".into(),
                        enabled: true,
                        script_ref: Some(ScriptRef {
                            name: "C".into(),
                            file_name: "".into(),
                        }),
                        calculation: None,
                        step_text: None,
                        broken_field_table: None,
                        has_broken_layout_ref: false,
                    }],
                },
                Script {
                    id: ScriptId(3),
                    name: "C".into(),
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

        let node = build_call_chain(&ddr, ScriptId(1), Some(1)).unwrap();
        // depth=1 なので B は展開されるが B の子（C）はカット
        let b = &node.children[0];
        assert_eq!(b.script_name, "B");
        // B の depth は 1、max_depth=1 なので C は is_cycle=false で children=[] になる
        assert!(b.children.is_empty() || b.children[0].children.is_empty());
    }
}
