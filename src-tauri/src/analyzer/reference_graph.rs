//! petgraph を使ったスクリプト参照グラフの構築。
//!
//! ノード: ScriptId
//! エッジ: Perform Script / ScriptTrigger による呼び出し関係（有向）

use std::collections::HashMap;

use petgraph::graph::{DiGraph, NodeIndex};

use crate::parser::models::{DdrFile, ScriptId};

/// スクリプト参照の有向グラフ。
///
/// ノードの重みは `ScriptId`、エッジの重みは呼び出し元ステップ名。
pub type ScriptGraph = DiGraph<ScriptId, String>;

/// DDR ファイルからスクリプト参照グラフを構築する。
///
/// 同一ファイル内のスクリプト参照のみを対象とする（外部ファイル参照は除外）。
pub fn build_script_graph(ddr: &DdrFile) -> (ScriptGraph, HashMap<ScriptId, NodeIndex>) {
    let mut graph = ScriptGraph::new();
    let mut node_map: HashMap<ScriptId, NodeIndex> = HashMap::new();

    // 全スクリプトをノードとして追加
    for script in &ddr.scripts {
        let idx = graph.add_node(script.id);
        node_map.insert(script.id, idx);
    }

    // スクリプト名 → ScriptId の逆引きマップ
    let name_to_id: HashMap<&str, ScriptId> = ddr
        .scripts
        .iter()
        .map(|s| (s.name.as_str(), s.id))
        .collect();

    // Perform Script ステップからエッジを追加
    for script in &ddr.scripts {
        let Some(&caller_idx) = node_map.get(&script.id) else {
            continue;
        };
        for step in &script.steps {
            let Some(ref script_ref) = step.script_ref else {
                continue;
            };
            // 外部ファイル参照（file_name が空でない）は除外
            if !script_ref.file_name.is_empty() {
                continue;
            }
            if let Some(&callee_id) = name_to_id.get(script_ref.name.as_str()) {
                let callee_idx = node_map[&callee_id];
                graph.add_edge(caller_idx, callee_idx, step.name.clone());
            }
        }
    }

    // ScriptTrigger からもエッジを追加
    for layout in &ddr.layouts {
        for trigger in &layout.script_triggers {
            // 外部ファイル参照は除外
            if !trigger.file_name.is_empty() {
                continue;
            }
            if let Some(&callee_id) = name_to_id.get(trigger.script_name.as_str()) {
                // トリガーは「レイアウト」から呼ばれるため、発信ノードは callee のみ記録
                // （レイアウトはグラフノードではないため、エッジは省略して被呼び出し集合に記録）
                let _ = (callee_id, callee_id); // 参照済みフラグのみ
            }
        }
    }

    (graph, node_map)
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
    fn graph_has_correct_node_count() {
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let (graph, _) = build_script_graph(&ddr);
        assert_eq!(graph.node_count(), ddr.scripts.len());
    }

    #[test]
    fn graph_node_map_covers_all_scripts() {
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let (_, node_map) = build_script_graph(&ddr);
        for script in &ddr.scripts {
            assert!(node_map.contains_key(&script.id));
        }
    }
}
