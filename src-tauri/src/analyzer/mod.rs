//! DDR 解析エンジン。
//!
//! - `reference_graph` — スクリプト参照グラフの構築（petgraph）
//! - `broken_refs`     — 壊れた参照（存在しないスクリプト/トリガー先）の検出
//! - `orphans`         — 未使用スクリプトの検出
//! - `call_chain`      — スクリプト呼び出しチェーン（DFS + 循環検出）
//! - `report_card`     — システム健全性レポートの生成
//! - `diff_engine`     — 2 つの DDR ファイル間の差分比較

pub mod broken_refs;
pub mod call_chain;
pub mod diff_engine;
pub mod orphans;
pub mod reference_graph;
pub mod report_card;

pub use broken_refs::{find_broken_refs, BrokenRef, BrokenRefKind};
pub use call_chain::{build_call_chain, find_callers, CallChainNode};
pub use diff_engine::{diff_ddr, DiffItem, DiffKind, DiffResult};
pub use orphans::{find_orphan_scripts, OrphanScript};
pub use reference_graph::{build_script_graph, ScriptGraph};
pub use report_card::{generate_report_card, ReportCard, ReportIssue, Severity};

// ---------------------------------------------------------------------------
// 共通エラー型
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("script not found: id={0}")]
    ScriptNotFound(u64),

    #[error("cycle detected in call chain starting from script id={0}")]
    CycleDetected(u64),
}
