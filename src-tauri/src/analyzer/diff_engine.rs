//! 2 つの DDR ファイル間の差分比較。
//!
//! スクリプト・テーブル・レイアウト・バリューリスト・カスタム関数の
//! 追加・削除・変更を名前ベースで検出する。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::parser::models::{DdrFile, Field, JoinPredicate, Relationship, Table, TableOccurrence};

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

    diff_tables(&old.tables, &new_ddr.tables, &mut items);

    diff_layouts(&old.layouts, &new_ddr.layouts, &mut items);

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

    diff_table_occurrences(
        &old.table_occurrences,
        &new_ddr.table_occurrences,
        &mut items,
    );

    diff_relationships(&old.relationships, &new_ddr.relationships, &mut items);

    DiffResult::new(items)
}

// ---------------------------------------------------------------------------
// 内部ヘルパー
// ---------------------------------------------------------------------------

/// テーブル専用の差分検出。フィールド単位の detail を生成する。
fn diff_tables(old_tables: &[Table], new_tables: &[Table], out: &mut Vec<DiffItem>) {
    let old_map: HashMap<&str, &Table> = old_tables.iter().map(|t| (t.name.as_str(), t)).collect();
    let new_map: HashMap<&str, &Table> = new_tables.iter().map(|t| (t.name.as_str(), t)).collect();

    // 追加
    for name in new_map.keys() {
        if !old_map.contains_key(name) {
            out.push(DiffItem {
                kind: DiffKind::Added,
                element_type: "table".into(),
                name: (*name).to_owned(),
                detail: None,
                project_id: None,
                compare_project_id: None,
            });
        }
    }

    // 削除
    for name in old_map.keys() {
        if !new_map.contains_key(name) {
            out.push(DiffItem {
                kind: DiffKind::Removed,
                element_type: "table".into(),
                name: (*name).to_owned(),
                detail: None,
                project_id: None,
                compare_project_id: None,
            });
        }
    }

    // 変更（フィールド単位で比較）
    for (name, old_t) in &old_map {
        if let Some(new_t) = new_map.get(name) {
            if let Some(detail) = field_diff_detail(&old_t.fields, &new_t.fields) {
                out.push(DiffItem {
                    kind: DiffKind::Modified,
                    element_type: "table".into(),
                    name: (*name).to_owned(),
                    detail: Some(detail),
                    project_id: None,
                    compare_project_id: None,
                });
            }
        }
    }
}

/// フィールド差分の detail 文字列を生成する。変化がなければ `None` を返す。
fn field_diff_detail(old_fields: &[Field], new_fields: &[Field]) -> Option<String> {
    let old_map: HashMap<&str, &Field> = old_fields.iter().map(|f| (f.name.as_str(), f)).collect();
    let new_map: HashMap<&str, &Field> = new_fields.iter().map(|f| (f.name.as_str(), f)).collect();

    let mut added: Vec<String> = Vec::new();
    let mut removed: Vec<String> = Vec::new();
    let mut changed: Vec<String> = Vec::new();

    for name in new_map.keys() {
        if !old_map.contains_key(name) {
            added.push((*name).to_owned());
        }
    }
    for name in old_map.keys() {
        if !new_map.contains_key(name) {
            removed.push((*name).to_owned());
        }
    }
    for (name, old_f) in &old_map {
        if let Some(new_f) = new_map.get(name) {
            if field_signature(old_f) != field_signature(new_f) {
                changed.push((*name).to_owned());
            }
        }
    }

    if added.is_empty() && removed.is_empty() && changed.is_empty() {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();
    if !added.is_empty() {
        parts.push(format_diff_section("追加", &added));
    }
    if !removed.is_empty() {
        parts.push(format_diff_section("削除", &removed));
    }
    if !changed.is_empty() {
        parts.push(format_diff_section("変更", &changed));
    }
    Some(parts.join(" / "))
}

/// フィールドの変化検出用シグネチャ（型・種別・グローバル・繰り返し数）。
fn field_signature(field: &Field) -> String {
    format!(
        "{:?}|{:?}|{}|{}",
        field.data_type, field.field_type, field.is_global, field.max_repeat
    )
}

/// 差分セクション（「追加: A, B他1件」形式）を生成する汎用ヘルパー。
fn format_diff_section(label: &str, names: &[String]) -> String {
    const MAX: usize = 3;
    let extra = names.len().saturating_sub(MAX);
    let shown = names[..names.len().min(MAX)].join(", ");
    if extra > 0 {
        format!("{}: {}他{}件", label, shown, extra)
    } else {
        format!("{}: {}", label, shown)
    }
}

/// レイアウト専用の差分検出。人間が読める detail を生成する。
fn diff_layouts(
    old_layouts: &[crate::parser::models::Layout],
    new_layouts: &[crate::parser::models::Layout],
    out: &mut Vec<DiffItem>,
) {
    let old_map: HashMap<&str, &crate::parser::models::Layout> =
        old_layouts.iter().map(|l| (l.name.as_str(), l)).collect();
    let new_map: HashMap<&str, &crate::parser::models::Layout> =
        new_layouts.iter().map(|l| (l.name.as_str(), l)).collect();

    // 追加
    for name in new_map.keys() {
        if !old_map.contains_key(name) {
            out.push(DiffItem {
                kind: DiffKind::Added,
                element_type: "layout".into(),
                name: (*name).to_owned(),
                detail: None,
                project_id: None,
                compare_project_id: None,
            });
        }
    }

    // 削除
    for name in old_map.keys() {
        if !new_map.contains_key(name) {
            out.push(DiffItem {
                kind: DiffKind::Removed,
                element_type: "layout".into(),
                name: (*name).to_owned(),
                detail: None,
                project_id: None,
                compare_project_id: None,
            });
        }
    }

    // 変更
    for (name, old_l) in &old_map {
        if let Some(new_l) = new_map.get(name) {
            if layout_signature(old_l) != layout_signature(new_l) {
                out.push(DiffItem {
                    kind: DiffKind::Modified,
                    element_type: "layout".into(),
                    name: (*name).to_owned(),
                    detail: Some(layout_diff_detail(old_l, new_l)),
                    project_id: None,
                    compare_project_id: None,
                });
            }
        }
    }
}

/// レイアウトの変化を人間が読めるテキストに変換する。
fn layout_diff_detail(
    old: &crate::parser::models::Layout,
    new_layout: &crate::parser::models::Layout,
) -> String {
    let mut parts: Vec<String> = Vec::new();

    // トリガー数の変化
    let old_trig = old.script_triggers.len();
    let new_trig = new_layout.script_triggers.len();
    if old_trig != new_trig {
        parts.push(format!("トリガー {}→{}", old_trig, new_trig));
    }

    // オブジェクト変化（object_key で比較）
    let old_obj_map: HashMap<u64, &crate::parser::models::LayoutObject> = old
        .layout_objects
        .iter()
        .map(|o| (o.object_key, o))
        .collect();
    let new_obj_map: HashMap<u64, &crate::parser::models::LayoutObject> = new_layout
        .layout_objects
        .iter()
        .map(|o| (o.object_key, o))
        .collect();

    let mut added_objs: Vec<String> = Vec::new();
    let mut removed_objs: Vec<String> = Vec::new();

    for (key, new_obj) in &new_obj_map {
        if !old_obj_map.contains_key(key) {
            added_objs.push(object_display_name(new_obj));
        }
    }
    for (key, old_obj) in &old_obj_map {
        if !new_obj_map.contains_key(key) {
            removed_objs.push(object_display_name(old_obj));
        }
    }
    // 変更: 種別ごとにグループ化（移動 / 計算式変更 / 属性変更）
    let mut changed_move: Vec<String> = Vec::new();
    let mut changed_calc: Vec<String> = Vec::new();
    let mut changed_attr: Vec<String> = Vec::new();

    for (key, old_obj) in &old_obj_map {
        if let Some(new_obj) = new_obj_map.get(key) {
            if object_signature(old_obj) != object_signature(new_obj) {
                let what = changed_what(old_obj, new_obj);
                let display = object_display_name(new_obj);
                if what.contains("移動") && !what.contains("計算式") && !what.contains("属性")
                {
                    changed_move.push(display);
                } else if what.contains("計算式") {
                    changed_calc.push(display);
                } else {
                    changed_attr.push(display);
                }
            }
        }
    }

    if !added_objs.is_empty() {
        parts.push(format_diff_section("追加", &added_objs));
    }
    if !removed_objs.is_empty() {
        parts.push(format_diff_section("削除", &removed_objs));
    }
    if !changed_move.is_empty() {
        parts.push(format_diff_section("移動", &changed_move));
    }
    if !changed_calc.is_empty() {
        parts.push(format_diff_section("計算式変更", &changed_calc));
    }
    if !changed_attr.is_empty() {
        parts.push(format_diff_section("属性変更", &changed_attr));
    }

    if parts.is_empty() {
        "変更あり".into()
    } else {
        parts.join(" / ")
    }
}

/// オブジェクトの人間が読める名前を返す（優先順位順）。
fn object_display_name(obj: &crate::parser::models::LayoutObject) -> String {
    if let Some(n) = &obj.object_name {
        if !n.is_empty() {
            return n.clone();
        }
    }
    if let Some(f) = &obj.field_name {
        return match &obj.field_table_occurrence {
            Some(t) => format!("{}::{}", t, f),
            None => f.clone(),
        };
    }
    if let Some(l) = &obj.button_label {
        if !l.is_empty() {
            return l.chars().take(20).collect();
        }
    }
    obj.object_type.clone()
}

/// オブジェクトの変化検出用シグネチャ（tooltip/hide_condition を含む）。
fn object_signature(obj: &crate::parser::models::LayoutObject) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        obj.object_type,
        obj.object_name.as_deref().unwrap_or(""),
        obj.button_label.as_deref().unwrap_or(""),
        obj.field_table_occurrence.as_deref().unwrap_or(""),
        obj.field_name.as_deref().unwrap_or(""),
        obj.bounds
            .as_ref()
            .map(|b| format!("{:.0},{:.0},{:.0},{:.0}", b.top, b.left, b.bottom, b.right))
            .unwrap_or_default(),
        obj.tooltip.as_deref().unwrap_or(""),
        obj.hide_condition.as_deref().unwrap_or(""),
    )
}

/// 2つのオブジェクト間で何が変わったかを返す（カンマ区切り）。
fn changed_what(
    old: &crate::parser::models::LayoutObject,
    new: &crate::parser::models::LayoutObject,
) -> String {
    let mut what: Vec<&str> = Vec::new();

    let bounds_changed = match (&old.bounds, &new.bounds) {
        (Some(o), Some(n)) => {
            (o.top - n.top).abs() > 0.5
                || (o.left - n.left).abs() > 0.5
                || (o.bottom - n.bottom).abs() > 0.5
                || (o.right - n.right).abs() > 0.5
        }
        (None, None) => false,
        _ => true,
    };
    if bounds_changed {
        what.push("移動");
    }

    if old.tooltip != new.tooltip || old.hide_condition != new.hide_condition {
        what.push("計算式変更");
    }

    if old.object_type != new.object_type || old.object_name != new.object_name {
        what.push("属性変更");
    }

    if what.is_empty() {
        "変更".into()
    } else {
        what.join(",")
    }
}

/// レイアウトの変更検出用シグネチャ（changed の判定に使用）。
fn layout_signature(layout: &crate::parser::models::Layout) -> String {
    let obj_sig: String = layout
        .layout_objects
        .iter()
        .map(object_signature)
        .collect::<Vec<_>>()
        .join("|");
    format!("triggers={};{}", layout.script_triggers.len(), obj_sig)
}

/// テーブルオカレンス専用の差分検出。
fn diff_table_occurrences(
    old_tos: &[TableOccurrence],
    new_tos: &[TableOccurrence],
    out: &mut Vec<DiffItem>,
) {
    let old_map: HashMap<&str, &TableOccurrence> = old_tos
        .iter()
        .map(|t| (t.occurrence_name.as_str(), t))
        .collect();
    let new_map: HashMap<&str, &TableOccurrence> = new_tos
        .iter()
        .map(|t| (t.occurrence_name.as_str(), t))
        .collect();

    for name in new_map.keys() {
        if !old_map.contains_key(name) {
            out.push(DiffItem {
                kind: DiffKind::Added,
                element_type: "table_occurrence".into(),
                name: (*name).to_owned(),
                detail: None,
                project_id: None,
                compare_project_id: None,
            });
        }
    }

    for name in old_map.keys() {
        if !new_map.contains_key(name) {
            out.push(DiffItem {
                kind: DiffKind::Removed,
                element_type: "table_occurrence".into(),
                name: (*name).to_owned(),
                detail: None,
                project_id: None,
                compare_project_id: None,
            });
        }
    }

    for (name, old_to) in &old_map {
        if let Some(new_to) = new_map.get(name) {
            if old_to.base_table_name != new_to.base_table_name {
                out.push(DiffItem {
                    kind: DiffKind::Modified,
                    element_type: "table_occurrence".into(),
                    name: (*name).to_owned(),
                    detail: Some(format!(
                        "ベーステーブル: {} → {}",
                        old_to.base_table_name, new_to.base_table_name
                    )),
                    project_id: None,
                    compare_project_id: None,
                });
            }
        }
    }
}

/// リレーション専用の差分検出。
fn diff_relationships(
    old_rels: &[Relationship],
    new_rels: &[Relationship],
    out: &mut Vec<DiffItem>,
) {
    let old_map: HashMap<&str, &Relationship> =
        old_rels.iter().map(|r| (r.name.as_str(), r)).collect();
    let new_map: HashMap<&str, &Relationship> =
        new_rels.iter().map(|r| (r.name.as_str(), r)).collect();

    for name in new_map.keys() {
        if !old_map.contains_key(name) {
            out.push(DiffItem {
                kind: DiffKind::Added,
                element_type: "relationship".into(),
                name: (*name).to_owned(),
                detail: None,
                project_id: None,
                compare_project_id: None,
            });
        }
    }

    for name in old_map.keys() {
        if !new_map.contains_key(name) {
            out.push(DiffItem {
                kind: DiffKind::Removed,
                element_type: "relationship".into(),
                name: (*name).to_owned(),
                detail: None,
                project_id: None,
                compare_project_id: None,
            });
        }
    }

    for (name, old_rel) in &old_map {
        if let Some(new_rel) = new_map.get(name) {
            let old_sig = relationship_signature(old_rel);
            let new_sig = relationship_signature(new_rel);
            if old_sig != new_sig {
                out.push(DiffItem {
                    kind: DiffKind::Modified,
                    element_type: "relationship".into(),
                    name: (*name).to_owned(),
                    detail: Some(format!("{old_sig} → {new_sig}")),
                    project_id: None,
                    compare_project_id: None,
                });
            }
        }
    }
}

/// リレーションの変化検出用シグネチャ（テーブル・predicates を含む）。
fn relationship_signature(rel: &Relationship) -> String {
    let preds: String = rel
        .predicates
        .iter()
        .map(predicate_signature)
        .collect::<Vec<_>>()
        .join(";");
    format!("{}={}|{}", rel.left_table, rel.right_table, preds)
}

fn predicate_signature(p: &JoinPredicate) -> String {
    format!("{} {} {}", p.left_field, p.operator, p.right_field)
}

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

    const MINIMAL_XML: &str = include_str!("../../../tests/fixtures/minimal.xml");

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

    fn make_field(
        name: &str,
        data_type: crate::parser::models::DataType,
    ) -> crate::parser::models::Field {
        use crate::parser::models::*;
        Field {
            id: FieldId(1),
            name: name.into(),
            data_type,
            field_type: FieldKind::Normal,
            comment: String::new(),
            is_global: false,
            max_repeat: 1,
            calculation: None,
            auto_enter_type: String::new(),
            auto_enter_calc: None,
            auto_enter_allow_editing: true,
            val_not_empty: false,
            val_unique: false,
            val_existing: false,
            val_max_length: None,
            val_value_list: None,
            val_calc: None,
            val_range_from: None,
            val_range_to: None,
            val_always: false,
            val_error_message: None,
            index_type: String::new(),
        }
    }

    fn make_table(
        name: &str,
        fields: Vec<crate::parser::models::Field>,
    ) -> crate::parser::models::Table {
        use crate::parser::models::*;
        Table {
            id: TableId(1),
            name: name.into(),
            fields,
        }
    }

    fn make_base() -> DdrFile {
        use crate::parser::models::*;
        use crate::parser::version::FmVersion;
        DdrFile {
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
        }
    }

    #[test]
    fn diff_table_shows_added_field() {
        use crate::parser::models::DataType;
        let old_t = make_table("T", vec![make_field("FieldA", DataType::Text)]);
        let new_t = make_table(
            "T",
            vec![
                make_field("FieldA", DataType::Text),
                make_field("FieldB", DataType::Number),
            ],
        );
        let mut base = make_base();
        base.tables = vec![old_t];
        let mut updated = base.clone();
        updated.tables = vec![new_t];

        let result = diff_ddr(&base, &updated);
        assert_eq!(result.modified_count, 1);
        let detail = result.items[0].detail.as_deref().unwrap_or("");
        assert!(
            detail.contains("追加"),
            "detail should contain '追加': {detail}"
        );
        assert!(
            detail.contains("FieldB"),
            "detail should contain 'FieldB': {detail}"
        );
    }

    #[test]
    fn diff_table_shows_removed_field() {
        use crate::parser::models::DataType;
        let old_t = make_table(
            "T",
            vec![
                make_field("FieldA", DataType::Text),
                make_field("OldField", DataType::Number),
            ],
        );
        let new_t = make_table("T", vec![make_field("FieldA", DataType::Text)]);
        let mut base = make_base();
        base.tables = vec![old_t];
        let mut updated = base.clone();
        updated.tables = vec![new_t];

        let result = diff_ddr(&base, &updated);
        assert_eq!(result.modified_count, 1);
        let detail = result.items[0].detail.as_deref().unwrap_or("");
        assert!(
            detail.contains("削除"),
            "detail should contain '削除': {detail}"
        );
        assert!(
            detail.contains("OldField"),
            "detail should contain 'OldField': {detail}"
        );
    }

    #[test]
    fn diff_table_shows_modified_field() {
        use crate::parser::models::DataType;
        let old_t = make_table("T", vec![make_field("StatusField", DataType::Text)]);
        let new_t = make_table("T", vec![make_field("StatusField", DataType::Number)]);
        let mut base = make_base();
        base.tables = vec![old_t];
        let mut updated = base.clone();
        updated.tables = vec![new_t];

        let result = diff_ddr(&base, &updated);
        assert_eq!(result.modified_count, 1);
        let detail = result.items[0].detail.as_deref().unwrap_or("");
        assert!(
            detail.contains("変更"),
            "detail should contain '変更': {detail}"
        );
        assert!(
            detail.contains("StatusField"),
            "detail should contain 'StatusField': {detail}"
        );
    }

    #[test]
    fn diff_table_no_change_is_skipped() {
        use crate::parser::models::DataType;
        let t = make_table("T", vec![make_field("FieldA", DataType::Text)]);
        let mut base = make_base();
        base.tables = vec![t.clone()];
        let mut updated = base.clone();
        updated.tables = vec![t];

        let result = diff_ddr(&base, &updated);
        assert_eq!(
            result.modified_count, 0,
            "identical table should not appear as Modified"
        );
    }

    #[test]
    fn diff_table_field_count_overflow() {
        use crate::parser::models::DataType;
        let old_t = make_table("T", vec![make_field("Existing", DataType::Text)]);
        let new_fields = vec![
            make_field("Existing", DataType::Text),
            make_field("NewA", DataType::Text),
            make_field("NewB", DataType::Text),
            make_field("NewC", DataType::Text),
            make_field("NewD", DataType::Text),
        ];
        let new_t = make_table("T", new_fields);
        let mut base = make_base();
        base.tables = vec![old_t];
        let mut updated = base.clone();
        updated.tables = vec![new_t];

        let result = diff_ddr(&base, &updated);
        assert_eq!(result.modified_count, 1);
        let detail = result.items[0].detail.as_deref().unwrap_or("");
        assert!(
            detail.contains("他"),
            "should contain '他N件' for overflow: {detail}"
        );
    }

    fn make_layout_obj(
        key: u64,
        bounds: Option<crate::parser::models::Bounds>,
        tooltip: Option<&str>,
    ) -> crate::parser::models::LayoutObject {
        use crate::parser::models::LayoutObject;
        LayoutObject {
            object_type: "Field".into(),
            object_key: key,
            object_name: None,
            button_label: None,
            field_table_occurrence: None,
            field_name: Some("SomeField".into()),
            tooltip: tooltip.map(|s| s.to_owned()),
            hide_condition: None,
            bounds,
            conditional_formats: vec![],
        }
    }

    fn make_layout(
        name: &str,
        objects: Vec<crate::parser::models::LayoutObject>,
    ) -> crate::parser::models::Layout {
        use crate::parser::models::Layout;
        Layout {
            id: crate::parser::models::LayoutId(1),
            name: name.into(),
            table_occurrence_name: None,
            script_triggers: vec![],
            button_script_refs: vec![],
            field_refs: vec![],
            layout_objects: objects,
        }
    }

    #[test]
    fn layout_diff_detects_tooltip_change() {
        // tooltip 変更は現在の object_signature に含まれていないため検出漏れ → これが Red になること
        use crate::parser::models::Bounds;
        let bounds = Some(Bounds {
            top: 0.0,
            left: 0.0,
            bottom: 10.0,
            right: 100.0,
        });
        let old_obj = make_layout_obj(1, bounds.clone(), Some("古いツールチップ"));
        let new_obj = make_layout_obj(1, bounds, Some("新しいツールチップ"));

        let mut base = make_base();
        base.layouts = vec![make_layout("L", vec![old_obj])];
        let mut updated = base.clone();
        updated.layouts = vec![make_layout("L", vec![new_obj])];

        let result = diff_ddr(&base, &updated);
        assert_eq!(
            result.modified_count, 1,
            "tooltip変更がModifiedとして検出されること"
        );
    }

    #[test]
    fn layout_diff_shows_move_detail() {
        use crate::parser::models::Bounds;
        let old_obj = make_layout_obj(
            1,
            Some(Bounds {
                top: 0.0,
                left: 0.0,
                bottom: 10.0,
                right: 100.0,
            }),
            None,
        );
        let new_obj = make_layout_obj(
            1,
            Some(Bounds {
                top: 50.0,
                left: 50.0,
                bottom: 60.0,
                right: 150.0,
            }),
            None,
        );

        let mut base = make_base();
        base.layouts = vec![make_layout("L", vec![old_obj])];
        let mut updated = base.clone();
        updated.layouts = vec![make_layout("L", vec![new_obj])];

        let result = diff_ddr(&base, &updated);
        assert_eq!(result.modified_count, 1);
        let detail = result.items[0].detail.as_deref().unwrap_or("");
        assert!(
            detail.contains("移動"),
            "位置変更時は '移動' が detail に含まれること: {detail}"
        );
    }

    #[test]
    fn layout_diff_shows_calc_change_detail() {
        use crate::parser::models::Bounds;
        let bounds = Some(Bounds {
            top: 0.0,
            left: 0.0,
            bottom: 10.0,
            right: 100.0,
        });
        let old_obj = make_layout_obj(1, bounds.clone(), Some("古い計算式"));
        let new_obj = make_layout_obj(1, bounds, Some("新しい計算式"));

        let mut base = make_base();
        base.layouts = vec![make_layout("L", vec![old_obj])];
        let mut updated = base.clone();
        updated.layouts = vec![make_layout("L", vec![new_obj])];

        let result = diff_ddr(&base, &updated);
        assert_eq!(result.modified_count, 1);
        let detail = result.items[0].detail.as_deref().unwrap_or("");
        assert!(
            detail.contains("計算式変更"),
            "tooltip変更時は '計算式変更' が detail に含まれること: {detail}"
        );
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

    // ---- テーブルオカレンス差分テスト ----

    fn make_to(
        occurrence_name: &str,
        base_table_name: &str,
    ) -> crate::parser::models::TableOccurrence {
        crate::parser::models::TableOccurrence {
            occurrence_name: occurrence_name.into(),
            base_table_name: base_table_name.into(),
            source_file: None,
        }
    }

    #[test]
    fn diff_table_occurrences_detects_added() {
        let mut base = make_base();
        base.table_occurrences = vec![make_to("Contact", "Contact")];
        let mut updated = base.clone();
        updated.table_occurrences =
            vec![make_to("Contact", "Contact"), make_to("Invoice", "Invoice")];
        let result = diff_ddr(&base, &updated);
        let to_items: Vec<_> = result
            .items
            .iter()
            .filter(|i| i.element_type == "table_occurrence")
            .collect();
        assert_eq!(to_items.len(), 1);
        assert_eq!(to_items[0].kind, DiffKind::Added);
        assert_eq!(to_items[0].name, "Invoice");
    }

    #[test]
    fn diff_table_occurrences_detects_removed() {
        let mut base = make_base();
        base.table_occurrences = vec![make_to("Contact", "Contact"), make_to("OldTO", "OldTable")];
        let mut updated = base.clone();
        updated.table_occurrences = vec![make_to("Contact", "Contact")];
        let result = diff_ddr(&base, &updated);
        let to_items: Vec<_> = result
            .items
            .iter()
            .filter(|i| i.element_type == "table_occurrence")
            .collect();
        assert_eq!(to_items.len(), 1);
        assert_eq!(to_items[0].kind, DiffKind::Removed);
        assert_eq!(to_items[0].name, "OldTO");
    }

    #[test]
    fn diff_table_occurrences_detects_base_table_change() {
        let mut base = make_base();
        base.table_occurrences = vec![make_to("Contact", "OldTable")];
        let mut updated = base.clone();
        updated.table_occurrences = vec![make_to("Contact", "NewTable")];
        let result = diff_ddr(&base, &updated);
        let to_items: Vec<_> = result
            .items
            .iter()
            .filter(|i| i.element_type == "table_occurrence")
            .collect();
        assert_eq!(to_items.len(), 1);
        assert_eq!(to_items[0].kind, DiffKind::Modified);
        assert_eq!(to_items[0].name, "Contact");
        let detail = to_items[0].detail.as_deref().unwrap_or("");
        assert!(
            detail.contains("OldTable"),
            "detail should contain old value: {detail}"
        );
        assert!(
            detail.contains("NewTable"),
            "detail should contain new value: {detail}"
        );
    }

    #[test]
    fn diff_table_occurrences_no_change_is_skipped() {
        let mut base = make_base();
        base.table_occurrences = vec![make_to("Contact", "Contact")];
        let mut updated = base.clone();
        updated.table_occurrences = vec![make_to("Contact", "Contact")];
        let result = diff_ddr(&base, &updated);
        let to_items: Vec<_> = result
            .items
            .iter()
            .filter(|i| i.element_type == "table_occurrence")
            .collect();
        assert!(
            to_items.is_empty(),
            "identical TO should not appear in diff"
        );
    }

    // ---- リレーション差分テスト ----

    fn make_pred(left: &str, right: &str) -> crate::parser::models::JoinPredicate {
        crate::parser::models::JoinPredicate {
            left_field: left.into(),
            right_field: right.into(),
            operator: "Equal".into(),
        }
    }

    fn make_rel(
        name: &str,
        left: &str,
        right: &str,
        preds: Vec<crate::parser::models::JoinPredicate>,
    ) -> crate::parser::models::Relationship {
        use crate::parser::models::RelationshipId;
        crate::parser::models::Relationship {
            id: RelationshipId(1),
            name: name.into(),
            left_table: left.into(),
            right_table: right.into(),
            predicates: preds,
        }
    }

    #[test]
    fn diff_relationships_detects_added() {
        let mut base = make_base();
        base.relationships = vec![make_rel(
            "Rel1",
            "Contact",
            "Invoice",
            vec![make_pred("id", "contact_id")],
        )];
        let mut updated = base.clone();
        updated
            .relationships
            .push(make_rel("Rel2", "Invoice", "LineItem", vec![]));
        let result = diff_ddr(&base, &updated);
        let rel_items: Vec<_> = result
            .items
            .iter()
            .filter(|i| i.element_type == "relationship")
            .collect();
        assert_eq!(rel_items.len(), 1);
        assert_eq!(rel_items[0].kind, DiffKind::Added);
        assert_eq!(rel_items[0].name, "Rel2");
    }

    #[test]
    fn diff_relationships_detects_removed() {
        let mut base = make_base();
        base.relationships = vec![
            make_rel("Rel1", "Contact", "Invoice", vec![]),
            make_rel("OldRel", "A", "B", vec![]),
        ];
        let mut updated = base.clone();
        updated.relationships = vec![make_rel("Rel1", "Contact", "Invoice", vec![])];
        let result = diff_ddr(&base, &updated);
        let rel_items: Vec<_> = result
            .items
            .iter()
            .filter(|i| i.element_type == "relationship")
            .collect();
        assert_eq!(rel_items.len(), 1);
        assert_eq!(rel_items[0].kind, DiffKind::Removed);
        assert_eq!(rel_items[0].name, "OldRel");
    }

    #[test]
    fn diff_relationships_detects_predicate_change() {
        let mut base = make_base();
        base.relationships = vec![make_rel(
            "Rel1",
            "Contact",
            "Invoice",
            vec![make_pred("id", "contact_id")],
        )];
        let mut updated = base.clone();
        updated.relationships = vec![make_rel(
            "Rel1",
            "Contact",
            "Invoice",
            vec![make_pred("id", "cid")],
        )];
        let result = diff_ddr(&base, &updated);
        let rel_items: Vec<_> = result
            .items
            .iter()
            .filter(|i| i.element_type == "relationship")
            .collect();
        assert_eq!(rel_items.len(), 1);
        assert_eq!(rel_items[0].kind, DiffKind::Modified);
    }

    #[test]
    fn diff_relationships_no_change_is_skipped() {
        let pred = make_pred("id", "contact_id");
        let mut base = make_base();
        base.relationships = vec![make_rel("Rel1", "Contact", "Invoice", vec![pred.clone()])];
        let mut updated = base.clone();
        updated.relationships = vec![make_rel("Rel1", "Contact", "Invoice", vec![pred])];
        let result = diff_ddr(&base, &updated);
        let rel_items: Vec<_> = result
            .items
            .iter()
            .filter(|i| i.element_type == "relationship")
            .collect();
        assert!(
            rel_items.is_empty(),
            "identical relationship should not appear in diff"
        );
    }
}
