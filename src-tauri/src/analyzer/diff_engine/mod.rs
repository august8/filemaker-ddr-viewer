//! 2 つの DDR ファイル間の差分比較。
//!
//! スクリプト・テーブル・レイアウト・バリューリスト・カスタム関数の
//! 追加・削除・変更を名前ベースで検出する。

mod catalog_diff;
mod layout_diff;
mod table_diff;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::parser::models::DdrFile;

// ---------------------------------------------------------------------------
// 公開型
// ---------------------------------------------------------------------------

/// 差分の種別。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffKind {
    Added,
    Removed,
    Modified,
}

/// 単一の差分アイテム。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffItem {
    pub kind: DiffKind,
    /// 対象の要素種別（"script", "table", "layout", "value_list", "custom_function"）
    pub element_type: String,
    /// 要素名
    pub name: String,
    /// 変更内容の説明（Modified の場合のみ）
    pub detail: Option<String>,
    /// 遷移先の project_id（Added/Modified は Target、Removed は Primary）
    pub project_id: Option<i64>,
    /// 比較元の project_id（詳細パネルでのフィールド/オブジェクトハイライト用）
    pub compare_project_id: Option<i64>,
}

/// 比較結果全体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub items: Vec<DiffItem>,
    pub added_count: usize,
    pub removed_count: usize,
    pub modified_count: usize,
}

impl DiffResult {
    pub(crate) fn new(items: Vec<DiffItem>) -> Self {
        let added_count = items.iter().filter(|i| i.kind == DiffKind::Added).count();
        let removed_count = items.iter().filter(|i| i.kind == DiffKind::Removed).count();
        let modified_count = items
            .iter()
            .filter(|i| i.kind == DiffKind::Modified)
            .count();
        DiffResult {
            items,
            added_count,
            removed_count,
            modified_count,
        }
    }
}

// ---------------------------------------------------------------------------
// ロジック
// ---------------------------------------------------------------------------

/// 2 つの DDR ファイルを比較して差分を返す。
///
/// `old` が比較元、`new_ddr` が比較先。
pub fn diff_ddr(old: &DdrFile, new_ddr: &DdrFile) -> DiffResult {
    let mut items = Vec::new();

    diff_named_list(
        "script",
        &old.scripts
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>(),
        &new_ddr
            .scripts
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>(),
        // スクリプトのステップ数が変わったら Modified と見なす
        &old.scripts
            .iter()
            .map(|s| (s.name.as_str(), format!("ステップ: {}", s.steps.len())))
            .collect(),
        &new_ddr
            .scripts
            .iter()
            .map(|s| (s.name.as_str(), format!("ステップ: {}", s.steps.len())))
            .collect(),
        &mut items,
    );

    table_diff::diff_tables(&old.tables, &new_ddr.tables, &mut items);

    layout_diff::diff_layouts(&old.layouts, &new_ddr.layouts, &mut items);

    diff_named_list(
        "value_list",
        &old.value_lists
            .iter()
            .map(|v| v.name.as_str())
            .collect::<Vec<_>>(),
        &new_ddr
            .value_lists
            .iter()
            .map(|v| v.name.as_str())
            .collect::<Vec<_>>(),
        &old.value_lists
            .iter()
            .map(|v| (v.name.as_str(), format!("値: {}", v.custom_values.len())))
            .collect(),
        &new_ddr
            .value_lists
            .iter()
            .map(|v| (v.name.as_str(), format!("値: {}", v.custom_values.len())))
            .collect(),
        &mut items,
    );

    diff_named_list(
        "custom_function",
        &old.custom_functions
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
        &new_ddr
            .custom_functions
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
        &old.custom_functions
            .iter()
            .map(|c| (c.name.as_str(), format!("引数: {}", c.parameters.len())))
            .collect(),
        &new_ddr
            .custom_functions
            .iter()
            .map(|c| (c.name.as_str(), format!("引数: {}", c.parameters.len())))
            .collect(),
        &mut items,
    );

    catalog_diff::diff_table_occurrences(
        &old.table_occurrences,
        &new_ddr.table_occurrences,
        &mut items,
    );

    catalog_diff::diff_relationships(&old.relationships, &new_ddr.relationships, &mut items);

    DiffResult::new(items)
}

// ---------------------------------------------------------------------------
// 共有ユーティリティ
// ---------------------------------------------------------------------------

/// 差分セクション（「追加: A, B他1件」形式）を生成する汎用ヘルパー。
pub(super) fn format_diff_section(label: &str, names: &[String]) -> String {
    const MAX: usize = 3;
    let extra = names.len().saturating_sub(MAX);
    let shown = names[..names.len().min(MAX)].join(", ");
    if extra > 0 {
        format!("{}: {}他{}件", label, shown, extra)
    } else {
        format!("{}: {}", label, shown)
    }
}

/// 汎用の名前リスト差分検出（スクリプト・バリューリスト・カスタム関数）。
fn diff_named_list(
    element_type: &str,
    old_names: &[&str],
    new_names: &[&str],
    old_sigs: &HashMap<&str, String>,
    new_sigs: &HashMap<&str, String>,
    out: &mut Vec<DiffItem>,
) {
    let old_set: std::collections::HashSet<&str> = old_names.iter().copied().collect();
    let new_set: std::collections::HashSet<&str> = new_names.iter().copied().collect();

    // 追加
    for name in new_set.difference(&old_set) {
        out.push(DiffItem {
            kind: DiffKind::Added,
            element_type: element_type.into(),
            name: (*name).to_owned(),
            detail: None,
            project_id: None,
            compare_project_id: None,
        });
    }

    // 削除
    for name in old_set.difference(&new_set) {
        out.push(DiffItem {
            kind: DiffKind::Removed,
            element_type: element_type.into(),
            name: (*name).to_owned(),
            detail: None,
            project_id: None,
            compare_project_id: None,
        });
    }

    // 変更（名前が同一でシグネチャが異なる）
    for name in old_set.intersection(&new_set) {
        let old_sig = old_sigs.get(name).map(|s| s.as_str()).unwrap_or("");
        let new_sig = new_sigs.get(name).map(|s| s.as_str()).unwrap_or("");
        if old_sig != new_sig {
            out.push(DiffItem {
                kind: DiffKind::Modified,
                element_type: element_type.into(),
                name: (*name).to_owned(),
                detail: Some(format!("{old_sig} → {new_sig}")),
                project_id: None,
                compare_project_id: None,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_ddr;

    const MINIMAL_XML: &str = include_str!("../../../../tests/fixtures/minimal.xml");

    #[test]
    fn diff_same_file_is_empty() {
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let result = diff_ddr(&ddr, &ddr);
        assert_eq!(result.added_count, 0);
        assert_eq!(result.removed_count, 0);
        assert_eq!(result.modified_count, 0);
    }

    #[test]
    fn diff_detects_added_script() {
        use crate::parser::models::*;
        use crate::parser::version::FmVersion;

        let base = DdrFile {
            file_name: "A".into(),
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
        };
        let mut updated = base.clone();
        updated.scripts.push(Script {
            id: ScriptId(1),
            name: "NewScript".into(),
            run_with_full_access: false,
            steps: vec![],
        });

        let result = diff_ddr(&base, &updated);
        assert_eq!(result.added_count, 1);
        assert_eq!(result.items[0].kind, DiffKind::Added);
        assert_eq!(result.items[0].name, "NewScript");
    }

    #[test]
    fn diff_detects_removed_table() {
        use crate::parser::models::*;
        use crate::parser::version::FmVersion;

        let mut old = DdrFile {
            file_name: "A".into(),
            fm_version: FmVersion {
                major: 21,
                minor: 0,
                patch: "v1".into(),
            },
            tables: vec![Table {
                id: TableId(1),
                name: "Contacts".into(),
                fields: vec![],
            }],
            scripts: vec![],
            layouts: vec![],
            relationships: vec![],
            value_lists: vec![],
            custom_functions: vec![],
            accounts: vec![],
            privilege_sets: vec![],
            table_occurrences: vec![],
            file_script_triggers: vec![],
        };
        let new_ddr = DdrFile {
            tables: vec![],
            ..old.clone()
        };
        old.file_name = "B".into();

        let result = diff_ddr(&old, &new_ddr);
        assert_eq!(result.removed_count, 1);
        assert_eq!(result.items[0].kind, DiffKind::Removed);
        assert_eq!(result.items[0].name, "Contacts");
    }

    #[test]
    fn diff_counts_match_items() {
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let result = diff_ddr(&ddr, &ddr);
        assert_eq!(
            result.added_count + result.removed_count + result.modified_count,
            result.items.len()
        );
    }
}
