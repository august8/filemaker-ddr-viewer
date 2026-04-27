//! DDR 一括挿入と内部ヘルパー関数。

use rusqlite::params;

use crate::db::{Database, DbError};
use crate::parser::models::{
    Account, CustomFunction, DataType, DdrFile, Field, FieldKind, JoinPredicate, Layout,
    LayoutFieldRef, LayoutObject, PrivilegeSet, Relationship, Script, ScriptStep, ScriptTrigger,
    Table, TableOccurrence, ValueList, ValueListFieldRef, ValueListSource,
};

// ---------------------------------------------------------------------------
// DDR 一括インサート
// ---------------------------------------------------------------------------

/// search_index 登録エントリ（DB ID 確定後に収集）
struct SearchEntry {
    element_type: &'static str,
    element_id: i64,
    name: String,
    content: String,
}

/// `DdrFile` をトランザクションで一括挿入し、プロジェクト ID を返す。
pub fn insert_ddr_file(
    db: &mut Database,
    ddr: &DdrFile,
    solution_id: i64,
    file_path: Option<&str>,
) -> Result<i64, DbError> {
    let tx = db.conn.transaction()?;

    // 1. project
    tx.execute(
        "INSERT INTO projects (solution_id, name, file_path, fm_version, imported_at) VALUES (?1, ?2, ?3, ?4, datetime('now', 'localtime'))",
        params![
            solution_id,
            ddr.file_name,
            file_path,
            ddr.fm_version.to_string()
        ],
    )?;
    let project_id = tx.last_insert_rowid();

    // DB ID を確定後に収集して search_index へ登録する
    let mut search_entries: Vec<SearchEntry> = Vec::new();

    // 2. tables + fields
    for table in &ddr.tables {
        let table_db_id = insert_table_inner(&tx, project_id, table)?;
        search_entries.push(SearchEntry {
            element_type: "table",
            element_id: table_db_id,
            name: table.name.clone(),
            content: String::new(),
        });
        for field in &table.fields {
            let field_db_id = insert_field_inner(&tx, project_id, table_db_id, field)?;
            let content = [
                field.comment.as_str(),
                field.calculation.as_deref().unwrap_or(""),
            ]
            .join(" ");
            search_entries.push(SearchEntry {
                element_type: "field",
                element_id: field_db_id,
                name: field.name.clone(),
                content: content.trim().to_string(),
            });
        }
    }

    // 3. scripts + steps
    for (pos, script) in ddr.scripts.iter().enumerate() {
        let script_db_id = insert_script_inner(&tx, project_id, script, pos as i64)?;
        // 全ステップテキストをコンテンツとしてインデックス（$変数・スクリプト内容の検索を可能にする）
        let content = script
            .steps
            .iter()
            .filter_map(|s| s.step_text.as_deref())
            .collect::<Vec<_>>()
            .join(" ");
        search_entries.push(SearchEntry {
            element_type: "script",
            element_id: script_db_id,
            name: script.name.clone(),
            content,
        });
        for (step_pos, step) in script.steps.iter().enumerate() {
            insert_step_inner(&tx, script_db_id, step, step_pos as i64)?;
        }
    }

    // 4. layouts + triggers + field refs + layout objects
    for (pos, layout) in ddr.layouts.iter().enumerate() {
        let layout_db_id = insert_layout_inner(&tx, project_id, layout, pos as i64)?;
        search_entries.push(SearchEntry {
            element_type: "layout",
            element_id: layout_db_id,
            name: layout.name.clone(),
            content: layout.table_occurrence_name.clone().unwrap_or_default(),
        });
        for trigger in &layout.script_triggers {
            insert_trigger_inner(&tx, layout_db_id, trigger)?;
        }
        for field_ref in &layout.field_refs {
            insert_layout_field_ref_inner(&tx, layout_db_id, field_ref)?;
        }
        for (obj_pos, obj) in layout.layout_objects.iter().enumerate() {
            let obj_db_id = insert_layout_object_inner(&tx, layout_db_id, obj, obj_pos as i64)?;
            for cf in &obj.conditional_formats {
                tx.execute(
                    "INSERT INTO layout_object_conditions (object_id, rule_order, calculation, format_css)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![obj_db_id, cf.rule_order, cf.calculation, cf.format_css],
                )?;
            }
        }
    }

    // 5. relationships + predicates + table occurrences
    for rel in &ddr.relationships {
        let rel_db_id = insert_relationship_inner(&tx, project_id, rel)?;
        let rel_content = rel
            .predicates
            .iter()
            .map(|p| {
                format!(
                    "{}::{} {} {}::{}",
                    rel.left_table, p.left_field, p.operator, rel.right_table, p.right_field
                )
            })
            .collect::<Vec<_>>()
            .join(" ");
        search_entries.push(SearchEntry {
            element_type: "relationship",
            element_id: rel_db_id,
            name: rel.name.clone(),
            content: rel_content,
        });
        for (pos, pred) in rel.predicates.iter().enumerate() {
            insert_predicate_inner(&tx, rel_db_id, pred, pos as i64)?;
        }
    }
    for occ in &ddr.table_occurrences {
        let occ_db_id = insert_table_occurrence_inner(&tx, project_id, occ)?;
        search_entries.push(SearchEntry {
            element_type: "table_occurrence",
            element_id: occ_db_id,
            name: occ.occurrence_name.clone(),
            content: occ.base_table_name.clone(),
        });
    }

    // 6. value lists + items + field refs
    for vl in &ddr.value_lists {
        let vl_db_id = insert_value_list_inner(&tx, project_id, vl)?;
        search_entries.push(SearchEntry {
            element_type: "value_list",
            element_id: vl_db_id,
            name: vl.name.clone(),
            content: vl.custom_values.join(" "),
        });
        for (pos, val) in vl.custom_values.iter().enumerate() {
            tx.execute(
                "INSERT INTO value_list_items (value_list_id, value, position) VALUES (?1, ?2, ?3)",
                params![vl_db_id, val, pos as i64],
            )?;
        }
        for field_ref in &vl.field_refs {
            insert_value_list_field_ref_inner(&tx, vl_db_id, field_ref)?;
        }
    }

    // 7. custom functions
    for cf in &ddr.custom_functions {
        let cf_db_id = insert_custom_function_inner(&tx, project_id, cf)?;
        let content = [
            cf.parameters.join("; ").as_str(),
            cf.calculation.as_deref().unwrap_or(""),
        ]
        .join(" ");
        search_entries.push(SearchEntry {
            element_type: "custom_function",
            element_id: cf_db_id,
            name: cf.name.clone(),
            content: content.trim().to_string(),
        });
    }

    // 8. accounts
    for account in &ddr.accounts {
        insert_account_inner(&tx, project_id, account)?;
    }

    // 9. privilege sets
    for ps in &ddr.privilege_sets {
        insert_privilege_set_inner(&tx, project_id, ps)?;
    }

    // 10. FTS5 インデックス（DB IDを使用）
    {
        let mut stmt = tx.prepare(
            "INSERT INTO search_index (project_id, element_type, element_id, name, content)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        for entry in &search_entries {
            stmt.execute(params![
                project_id,
                entry.element_type,
                entry.element_id,
                entry.name,
                entry.content
            ])?;
        }
    }

    tx.commit()?;
    Ok(project_id)
}

/// 条件付き書式ルールを1件 insert する。
pub fn insert_layout_object_condition(
    db: &Database,
    object_id: i64,
    rule_order: u32,
    calculation: &str,
    format_css: &str,
) -> Result<(), DbError> {
    db.conn.execute(
        "INSERT INTO layout_object_conditions (object_id, rule_order, calculation, format_css)
         VALUES (?1, ?2, ?3, ?4)",
        params![object_id, rule_order, calculation, format_css],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// 内部: 個別インサートヘルパー
// ---------------------------------------------------------------------------

fn insert_table_inner(
    tx: &rusqlite::Transaction<'_>,
    project_id: i64,
    table: &Table,
) -> Result<i64, DbError> {
    tx.execute(
        "INSERT INTO base_tables (project_id, fm_id, name) VALUES (?1, ?2, ?3)",
        params![project_id, table.id.0 as i64, table.name],
    )?;
    Ok(tx.last_insert_rowid())
}

fn insert_field_inner(
    tx: &rusqlite::Transaction<'_>,
    project_id: i64,
    table_db_id: i64,
    field: &Field,
) -> Result<i64, DbError> {
    tx.execute(
        "INSERT INTO fields
            (project_id, table_id, fm_id, name, data_type, field_type,
             comment, is_global, max_repeat, calculation,
             auto_enter_type, auto_enter_calc, auto_enter_allow_editing,
             val_not_empty, val_unique, val_existing, val_max_length,
             val_value_list, val_calc, val_range_from, val_range_to,
             val_always, val_error_message, index_type, container_storage)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,
                 ?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25)",
        params![
            project_id,
            table_db_id,
            field.id.0 as i64,
            field.name,
            data_type_str(&field.data_type),
            field_kind_str(&field.field_type),
            field.comment,
            field.is_global as i64,
            field.max_repeat as i64,
            field.calculation,
            field.auto_enter_type,
            field.auto_enter_calc,
            field.auto_enter_allow_editing as i64,
            field.val_not_empty as i64,
            field.val_unique as i64,
            field.val_existing as i64,
            field.val_max_length,
            field.val_value_list,
            field.val_calc,
            field.val_range_from,
            field.val_range_to,
            field.val_always as i64,
            field.val_error_message,
            field.index_type,
            field.container_storage,
        ],
    )?;
    Ok(tx.last_insert_rowid())
}

fn insert_script_inner(
    tx: &rusqlite::Transaction<'_>,
    project_id: i64,
    script: &Script,
    position: i64,
) -> Result<i64, DbError> {
    tx.execute(
        "INSERT INTO scripts (project_id, fm_id, name, run_with_full_access, position)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            project_id,
            script.id.0 as i64,
            script.name,
            script.run_with_full_access as i64,
            position,
        ],
    )?;
    Ok(tx.last_insert_rowid())
}

fn insert_step_inner(
    tx: &rusqlite::Transaction<'_>,
    script_db_id: i64,
    step: &ScriptStep,
    position: i64,
) -> Result<(), DbError> {
    let (ref_name, ref_file) = step
        .script_ref
        .as_ref()
        .map(|r| (Some(r.name.as_str()), Some(r.file_name.as_str())))
        .unwrap_or((None, None));

    tx.execute(
        "INSERT INTO script_steps
            (script_id, step_type_id, name, enabled,
             script_ref_name, script_ref_file, calculation, step_text, position)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            script_db_id,
            step.step_id as i64,
            step.name,
            step.enabled as i64,
            ref_name,
            ref_file,
            step.calculation,
            step.step_text,
            position,
        ],
    )?;
    Ok(())
}

fn insert_layout_inner(
    tx: &rusqlite::Transaction<'_>,
    project_id: i64,
    layout: &Layout,
    position: i64,
) -> Result<i64, DbError> {
    tx.execute(
        "INSERT INTO layouts (project_id, fm_id, name, table_occurrence_name, position)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            project_id,
            layout.id.0 as i64,
            layout.name,
            layout.table_occurrence_name,
            position,
        ],
    )?;
    Ok(tx.last_insert_rowid())
}

fn insert_trigger_inner(
    tx: &rusqlite::Transaction<'_>,
    layout_db_id: i64,
    trigger: &ScriptTrigger,
) -> Result<(), DbError> {
    tx.execute(
        "INSERT INTO script_triggers (layout_id, event, script_name, file_name)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            layout_db_id,
            trigger.event,
            trigger.script_name,
            trigger.file_name
        ],
    )?;
    Ok(())
}

fn insert_layout_field_ref_inner(
    tx: &rusqlite::Transaction<'_>,
    layout_db_id: i64,
    field_ref: &LayoutFieldRef,
) -> Result<(), DbError> {
    // 同一レイアウト内で同じオカレンス+フィールドが複数回現れる場合は無視する
    tx.execute(
        "INSERT OR IGNORE INTO layout_field_refs (layout_id, table_occurrence, field_name)
         VALUES (?1, ?2, ?3)",
        params![
            layout_db_id,
            field_ref.table_occurrence,
            field_ref.field_name
        ],
    )?;
    Ok(())
}

fn insert_layout_object_inner(
    tx: &rusqlite::Transaction<'_>,
    layout_db_id: i64,
    obj: &LayoutObject,
    position: i64,
) -> Result<i64, DbError> {
    let (bound_top, bound_left, bound_bottom, bound_right) = obj
        .bounds
        .as_ref()
        .map(|b| (Some(b.top), Some(b.left), Some(b.bottom), Some(b.right)))
        .unwrap_or((None, None, None, None));
    tx.execute(
        "INSERT INTO layout_objects
            (layout_id, object_type, object_key, object_name, button_label,
             field_table_occurrence, field_name, tooltip, hide_condition, position,
             bound_top, bound_left, bound_bottom, bound_right)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            layout_db_id,
            obj.object_type,
            obj.object_key as i64,
            obj.object_name,
            obj.button_label,
            obj.field_table_occurrence,
            obj.field_name,
            obj.tooltip,
            obj.hide_condition,
            position,
            bound_top,
            bound_left,
            bound_bottom,
            bound_right,
        ],
    )?;
    Ok(tx.last_insert_rowid())
}

fn insert_table_occurrence_inner(
    tx: &rusqlite::Transaction<'_>,
    project_id: i64,
    occ: &TableOccurrence,
) -> Result<i64, DbError> {
    // 同名オカレンスが重複する場合は無視する
    tx.execute(
        "INSERT OR IGNORE INTO table_occurrences (project_id, occurrence_name, base_table_name, source_file)
         VALUES (?1, ?2, ?3, ?4)",
        params![project_id, occ.occurrence_name, occ.base_table_name, occ.source_file.as_deref().unwrap_or("")],
    )?;
    // INSERT OR IGNORE の場合でも ID を返すためクエリで取得
    let id: i64 = tx.query_row(
        "SELECT id FROM table_occurrences WHERE project_id = ?1 AND occurrence_name = ?2",
        params![project_id, occ.occurrence_name],
        |r| r.get(0),
    )?;
    Ok(id)
}

fn insert_relationship_inner(
    tx: &rusqlite::Transaction<'_>,
    project_id: i64,
    rel: &Relationship,
) -> Result<i64, DbError> {
    tx.execute(
        "INSERT INTO relationships (project_id, fm_id, name, left_table, right_table)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            project_id,
            rel.id.0 as i64,
            rel.name,
            rel.left_table,
            rel.right_table,
        ],
    )?;
    Ok(tx.last_insert_rowid())
}

fn insert_predicate_inner(
    tx: &rusqlite::Transaction<'_>,
    rel_db_id: i64,
    pred: &JoinPredicate,
    position: i64,
) -> Result<(), DbError> {
    tx.execute(
        "INSERT INTO join_predicates
            (relationship_id, left_field, right_field, operator, position)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            rel_db_id,
            pred.left_field,
            pred.right_field,
            pred.operator,
            position
        ],
    )?;
    Ok(())
}

fn insert_value_list_inner(
    tx: &rusqlite::Transaction<'_>,
    project_id: i64,
    vl: &ValueList,
) -> Result<i64, DbError> {
    tx.execute(
        "INSERT INTO value_lists (project_id, fm_id, name, source) VALUES (?1, ?2, ?3, ?4)",
        params![
            project_id,
            vl.id.0 as i64,
            vl.name,
            value_list_source_str(&vl.source),
        ],
    )?;
    Ok(tx.last_insert_rowid())
}

fn insert_value_list_field_ref_inner(
    tx: &rusqlite::Transaction<'_>,
    value_list_id: i64,
    field_ref: &ValueListFieldRef,
) -> Result<(), DbError> {
    tx.execute(
        "INSERT INTO value_list_field_refs (value_list_id, table_occurrence, field_name)
         VALUES (?1, ?2, ?3)",
        params![
            value_list_id,
            field_ref.table_occurrence,
            field_ref.field_name
        ],
    )?;
    Ok(())
}

fn insert_custom_function_inner(
    tx: &rusqlite::Transaction<'_>,
    project_id: i64,
    cf: &CustomFunction,
) -> Result<i64, DbError> {
    tx.execute(
        "INSERT INTO custom_functions (project_id, fm_id, name, parameters, calculation)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            project_id,
            cf.id.0 as i64,
            cf.name,
            cf.parameters.join("; "),
            cf.calculation,
        ],
    )?;
    Ok(tx.last_insert_rowid())
}

fn insert_account_inner(
    tx: &rusqlite::Transaction<'_>,
    project_id: i64,
    account: &Account,
) -> Result<(), DbError> {
    tx.execute(
        "INSERT INTO accounts (project_id, fm_id, name, privilege_set, enabled)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            project_id,
            account.id.0 as i64,
            account.name,
            account.privilege_set,
            account.enabled as i64,
        ],
    )?;
    Ok(())
}

fn insert_privilege_set_inner(
    tx: &rusqlite::Transaction<'_>,
    project_id: i64,
    ps: &PrivilegeSet,
) -> Result<(), DbError> {
    tx.execute(
        "INSERT INTO privilege_sets (project_id, fm_id, name, comment)
         VALUES (?1, ?2, ?3, ?4)",
        params![project_id, ps.id.0 as i64, ps.name, ps.comment],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// 内部: enum → &str 変換
// ---------------------------------------------------------------------------

fn data_type_str(dt: &DataType) -> &str {
    match dt {
        DataType::Text => "Text",
        DataType::Number => "Number",
        DataType::Date => "Date",
        DataType::Time => "Time",
        DataType::Timestamp => "Timestamp",
        DataType::Container => "Container",
        DataType::Unknown(s) => s.as_str(),
    }
}

fn field_kind_str(fk: &FieldKind) -> &str {
    match fk {
        FieldKind::Normal => "Normal",
        FieldKind::Calculated => "Calculated",
        FieldKind::Summary => "Summary",
        FieldKind::Unknown(s) => s.as_str(),
    }
}

fn value_list_source_str(src: &ValueListSource) -> &str {
    match src {
        ValueListSource::Custom => "Custom",
        ValueListSource::Field => "Field",
        ValueListSource::Unknown(s) => s.as_str(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::repository::{
        delete_project, delete_solution, insert_solution, list_layout_object_conditions,
    };
    use crate::db::Database;
    use crate::parser::parse_ddr;

    const MINIMAL_XML: &str = include_str!("../../../../tests/fixtures/minimal.xml");

    fn db_with_minimal() -> (Database, i64, i64) {
        let mut db = Database::open_in_memory().unwrap();
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let sid = insert_solution(&mut db, "TestSolution", Some("/tmp/test.xml")).unwrap();
        let pid = insert_ddr_file(&mut db, &ddr, sid, Some("/tmp/test.xml")).unwrap();
        (db, sid, pid)
    }

    #[test]
    fn insert_creates_project() {
        let (db, _sid, pid) = db_with_minimal();
        let project = crate::db::repository::get_project(&db, pid).unwrap();
        assert_eq!(project.name, "TestDB");
        assert_eq!(project.fm_version, "21.0v1");
        assert_eq!(project.file_path.as_deref(), Some("/tmp/test.xml"));
    }

    #[test]
    fn insert_creates_tables_and_fields() {
        let (db, _sid, pid) = db_with_minimal();
        let n_tables: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM base_tables WHERE project_id=?1",
                params![pid],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n_tables, 1);

        let n_fields: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM fields WHERE project_id=?1",
                params![pid],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n_fields, 1);
    }

    #[test]
    fn insert_creates_scripts_and_steps() {
        let (db, _sid, pid) = db_with_minimal();
        let n_scripts: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM scripts WHERE project_id=?1",
                params![pid],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n_scripts, 1);

        let n_steps: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM script_steps ss
                   JOIN scripts s ON ss.script_id = s.id
                  WHERE s.project_id=?1",
                params![pid],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n_steps, 2);
    }

    #[test]
    fn insert_creates_layouts_and_triggers() {
        let (db, _sid, pid) = db_with_minimal();
        let n_layouts: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM layouts WHERE project_id=?1",
                params![pid],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n_layouts, 1);

        let n_triggers: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM script_triggers st
                   JOIN layouts l ON st.layout_id = l.id
                  WHERE l.project_id=?1",
                params![pid],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n_triggers, 1);
    }

    #[test]
    fn insert_creates_relationships_and_predicates() {
        let (db, _sid, pid) = db_with_minimal();
        let n_rels: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM relationships WHERE project_id=?1",
                params![pid],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n_rels, 1);

        let n_preds: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM join_predicates jp
                   JOIN relationships r ON jp.relationship_id = r.id
                  WHERE r.project_id=?1",
                params![pid],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n_preds, 1);
    }

    #[test]
    fn insert_creates_value_lists_and_items() {
        let (db, _sid, pid) = db_with_minimal();
        let n_vl: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM value_lists WHERE project_id=?1",
                params![pid],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n_vl, 1);

        let n_items: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM value_list_items vli
                   JOIN value_lists vl ON vli.value_list_id = vl.id
                  WHERE vl.project_id=?1",
                params![pid],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n_items, 2);
    }

    #[test]
    fn insert_creates_custom_functions() {
        let (db, _sid, pid) = db_with_minimal();
        let n: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM custom_functions WHERE project_id=?1",
                params![pid],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn insert_creates_accounts_and_privilege_sets() {
        let (db, _sid, pid) = db_with_minimal();
        let n_accts: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM accounts WHERE project_id=?1",
                params![pid],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n_accts, 1);

        let n_ps: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM privilege_sets WHERE project_id=?1",
                params![pid],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n_ps, 1);
    }

    #[test]
    fn delete_project_removes_search_index() {
        let (mut db, _sid, pid) = db_with_minimal();
        let before: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM search_index WHERE project_id=?1",
                params![pid],
                |r| r.get(0),
            )
            .unwrap();
        assert!(before > 0, "search_index に登録があること");

        delete_project(&mut db, pid).unwrap();

        let after: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM search_index WHERE project_id=?1",
                params![pid],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            after, 0,
            "delete_project 後は search_index からも削除される"
        );
    }

    #[test]
    fn delete_solution_removes_search_index() {
        let (mut db, sid, pid) = db_with_minimal();
        let before: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM search_index WHERE project_id=?1",
                params![pid],
                |r| r.get(0),
            )
            .unwrap();
        assert!(before > 0, "search_index に登録があること");

        delete_solution(&mut db, sid).unwrap();

        let after: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM search_index WHERE project_id=?1",
                params![pid],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            after, 0,
            "delete_solution 後は search_index からも削除される"
        );
    }

    #[test]
    fn insert_and_list_conditions() {
        let mut db = Database::open_in_memory().unwrap();
        let sid = insert_solution(&mut db, "S", None).unwrap();

        let tx = db.conn.transaction().unwrap();
        tx.execute(
            "INSERT INTO projects (solution_id, name, fm_version) VALUES (?1, ?2, ?3)",
            params![sid, "P", "21"],
        )
        .unwrap();
        let pid = tx.last_insert_rowid();
        tx.execute(
            "INSERT INTO layouts (project_id, fm_id, name, table_occurrence_name) VALUES (?1, 1, 'L', NULL)",
            params![pid],
        )
        .unwrap();
        let lid = tx.last_insert_rowid();
        tx.execute(
            "INSERT INTO layout_objects
               (layout_id, object_type, object_key, object_name, button_label,
                field_table_occurrence, field_name, tooltip, hide_condition, position,
                bound_top, bound_left, bound_bottom, bound_right)
             VALUES (?1, 'Text', 400, NULL, NULL, NULL, NULL, NULL, NULL, 0, 0, 0, 20, 100)",
            params![lid],
        )
        .unwrap();
        let oid = tx.last_insert_rowid();
        tx.commit().unwrap();

        insert_layout_object_condition(&db, oid, 0, "Table::Field = 0", "color: red;").unwrap();
        insert_layout_object_condition(&db, oid, 1, "Table::Field = 1", "color: blue;").unwrap();

        let conditions = list_layout_object_conditions(&db, oid, -1, 0).unwrap();
        assert_eq!(conditions.len(), 2);
        assert_eq!(conditions[0].rule_order, 0);
        assert_eq!(conditions[0].calculation, "Table::Field = 0");
        assert!(conditions[0].format_css.contains("red"));
        assert_eq!(conditions[1].rule_order, 1);
    }
}
