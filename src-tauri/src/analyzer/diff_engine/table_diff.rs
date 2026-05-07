use std::collections::HashMap;

use crate::parser::models::{Field, Table};

use super::{DiffItem, DiffKind};

/// テーブル専用の差分検出。フィールド単位の detail を生成する。
pub(super) fn diff_tables(old_tables: &[Table], new_tables: &[Table], out: &mut Vec<DiffItem>) {
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
        parts.push(super::format_diff_section("追加", &added));
    }
    if !removed.is_empty() {
        parts.push(super::format_diff_section("削除", &removed));
    }
    if !changed.is_empty() {
        parts.push(super::format_diff_section("変更", &changed));
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::super::diff_ddr;
    use crate::parser::models::*;
    use crate::parser::version::FmVersion;

    fn make_field(name: &str, data_type: DataType) -> Field {
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
            container_storage: None,
        }
    }

    fn make_table(name: &str, fields: Vec<Field>) -> Table {
        Table {
            id: TableId(1),
            name: name.into(),
            fields,
        }
    }

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
            external_data_sources: vec![],
        }
    }

    #[test]
    fn diff_table_shows_added_field() {
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
}
