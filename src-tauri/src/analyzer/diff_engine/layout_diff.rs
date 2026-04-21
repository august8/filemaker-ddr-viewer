use std::collections::HashMap;

use crate::parser::models::{Layout, LayoutObject};

use super::{DiffItem, DiffKind};

/// レイアウト専用の差分検出。人間が読める detail を生成する。
pub(super) fn diff_layouts(
    old_layouts: &[Layout],
    new_layouts: &[Layout],
    out: &mut Vec<DiffItem>,
) {
    let old_map: HashMap<&str, &Layout> =
        old_layouts.iter().map(|l| (l.name.as_str(), l)).collect();
    let new_map: HashMap<&str, &Layout> =
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
fn layout_diff_detail(old: &Layout, new_layout: &Layout) -> String {
    let mut parts: Vec<String> = Vec::new();

    // トリガー数の変化
    let old_trig = old.script_triggers.len();
    let new_trig = new_layout.script_triggers.len();
    if old_trig != new_trig {
        parts.push(format!("トリガー {}→{}", old_trig, new_trig));
    }

    // オブジェクト変化（object_key で比較）
    let old_obj_map: HashMap<u64, &LayoutObject> = old
        .layout_objects
        .iter()
        .map(|o| (o.object_key, o))
        .collect();
    let new_obj_map: HashMap<u64, &LayoutObject> = new_layout
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
        parts.push(super::format_diff_section("追加", &added_objs));
    }
    if !removed_objs.is_empty() {
        parts.push(super::format_diff_section("削除", &removed_objs));
    }
    if !changed_move.is_empty() {
        parts.push(super::format_diff_section("移動", &changed_move));
    }
    if !changed_calc.is_empty() {
        parts.push(super::format_diff_section("計算式変更", &changed_calc));
    }
    if !changed_attr.is_empty() {
        parts.push(super::format_diff_section("属性変更", &changed_attr));
    }

    if parts.is_empty() {
        "変更あり".into()
    } else {
        parts.join(" / ")
    }
}

/// オブジェクトの人間が読める名前を返す（優先順位順）。
fn object_display_name(obj: &LayoutObject) -> String {
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
fn object_signature(obj: &LayoutObject) -> String {
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
fn changed_what(old: &LayoutObject, new: &LayoutObject) -> String {
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
fn layout_signature(layout: &Layout) -> String {
    let obj_sig: String = layout
        .layout_objects
        .iter()
        .map(object_signature)
        .collect::<Vec<_>>()
        .join("|");
    format!("triggers={};{}", layout.script_triggers.len(), obj_sig)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::super::diff_ddr;
    use crate::parser::models::*;
    use crate::parser::version::FmVersion;

    fn make_base() -> DdrFile {
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

    fn make_layout_obj(key: u64, bounds: Option<Bounds>, tooltip: Option<&str>) -> LayoutObject {
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

    fn make_layout(name: &str, objects: Vec<LayoutObject>) -> Layout {
        Layout {
            id: LayoutId(1),
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
}
