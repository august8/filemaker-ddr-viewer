//! エンティティ一覧取得コマンド。

use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::{
    commands::CommandError,
    db::{
        repository::{
            list_accounts as db_list_accounts, list_custom_functions as db_list_custom_functions,
            list_external_data_sources as db_list_external_data_sources,
            list_layout_object_conditions as db_list_layout_object_conditions,
            list_layout_objects as db_list_layout_objects,
            list_layout_triggers as db_list_layout_triggers, list_layouts as db_list_layouts,
            list_privilege_sets as db_list_privilege_sets,
            list_relationships as db_list_relationships, list_script_steps as db_list_script_steps,
            list_scripts as db_list_scripts, list_table_fields as db_list_table_fields,
            list_table_occurrences as db_list_table_occurrences, list_tables as db_list_tables,
            list_value_list_items as db_list_value_list_items,
            list_value_lists as db_list_value_lists, AccountRow, ConditionRow, CustomFunctionRow,
            ExternalDataSourceRow, FieldRow, LayoutObjectRow, LayoutRow, PrivilegeSetRow,
            RelationshipRow, ScriptRow, ScriptStepRow, TableOccurrenceRow, TableRow, TriggerRow,
            ValueListRow,
        },
        Database,
    },
    AppState,
};

// ---------------------------------------------------------------------------
// 公開型
// ---------------------------------------------------------------------------

/// プロジェクト内の全フィールドをテーブル横断で返す（テーブル名順、フィールド名順）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllFieldRow {
    pub id: i64,
    pub fm_id: i64,
    pub name: String,
    pub data_type: String,
    pub field_type: String,
    pub comment: String,
    pub is_global: bool,
    pub max_repeat: i64,
    pub calculation: Option<String>,
    pub table_id: i64,
    pub table_name: String,
}

// ---------------------------------------------------------------------------
// Tauri コマンド
// ---------------------------------------------------------------------------

/// プロジェクトのテーブル一覧を返す。
///
/// `limit=None` で全件取得（後方互換）。
#[tauri::command]
pub async fn list_tables(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<TableRow>, CommandError> {
    let db = super::lock_db(&state)?;
    db_list_tables(&db, project_id, limit.unwrap_or(-1), offset.unwrap_or(0))
        .map_err(CommandError::from)
}

/// プロジェクト内の全フィールドをテーブル横断で返す。
///
/// `limit=None` で全件取得（後方互換）。
#[tauri::command]
pub async fn list_all_fields(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<AllFieldRow>, CommandError> {
    let db = super::lock_db(&state)?;
    list_all_fields_inner(&db, project_id, limit.unwrap_or(-1), offset.unwrap_or(0))
        .map_err(CommandError::from)
}

fn list_all_fields_inner(
    db: &Database,
    project_id: i64,
    limit: i64,
    offset: i64,
) -> Result<Vec<AllFieldRow>, rusqlite::Error> {
    let mut stmt = db.conn.prepare(
        "SELECT f.id, f.fm_id, f.name, f.data_type, f.field_type,
                f.comment, f.is_global, f.max_repeat, f.calculation,
                bt.id as table_id, bt.name as table_name
           FROM fields f
           JOIN base_tables bt ON bt.id = f.table_id
          WHERE f.project_id = ?1
          ORDER BY bt.name, f.name
          LIMIT ?2 OFFSET ?3",
    )?;
    let rows = stmt
        .query_map(params![project_id, limit, offset], |row| {
            Ok(AllFieldRow {
                id: row.get(0)?,
                fm_id: row.get(1)?,
                name: row.get(2)?,
                data_type: row.get(3)?,
                field_type: row.get(4)?,
                comment: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                is_global: row.get::<_, i64>(6)? != 0,
                max_repeat: row.get(7)?,
                calculation: row.get(8)?,
                table_id: row.get(9)?,
                table_name: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// テーブルのフィールド一覧を返す。
#[tauri::command]
pub async fn list_table_fields(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    table_id: i64,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<FieldRow>, CommandError> {
    let db = super::lock_db(&state)?;
    db_list_table_fields(
        &db,
        project_id,
        table_id,
        limit.unwrap_or(-1),
        offset.unwrap_or(0),
    )
    .map_err(CommandError::from)
}

/// プロジェクトのスクリプト一覧を返す。
#[tauri::command]
pub async fn list_scripts(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ScriptRow>, CommandError> {
    let db = super::lock_db(&state)?;
    db_list_scripts(&db, project_id, limit.unwrap_or(-1), offset.unwrap_or(0))
        .map_err(CommandError::from)
}

/// スクリプトのステップ一覧を返す。
#[tauri::command]
pub async fn list_script_steps(
    state: tauri::State<'_, AppState>,
    script_id: i64,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ScriptStepRow>, CommandError> {
    let db = super::lock_db(&state)?;
    db_list_script_steps(&db, script_id, limit.unwrap_or(-1), offset.unwrap_or(0))
        .map_err(CommandError::from)
}

/// プロジェクトのレイアウト一覧を返す。
#[tauri::command]
pub async fn list_layouts(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<LayoutRow>, CommandError> {
    let db = super::lock_db(&state)?;
    db_list_layouts(&db, project_id, limit.unwrap_or(-1), offset.unwrap_or(0))
        .map_err(CommandError::from)
}

/// レイアウトのトリガー一覧を返す。
#[tauri::command]
pub async fn list_layout_triggers(
    state: tauri::State<'_, AppState>,
    layout_id: i64,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<TriggerRow>, CommandError> {
    let db = super::lock_db(&state)?;
    db_list_layout_triggers(&db, layout_id, limit.unwrap_or(-1), offset.unwrap_or(0))
        .map_err(CommandError::from)
}

/// レイアウトのオブジェクト一覧を返す。
#[tauri::command]
pub async fn list_layout_objects(
    state: tauri::State<'_, AppState>,
    layout_id: i64,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<LayoutObjectRow>, CommandError> {
    let db = super::lock_db(&state)?;
    db_list_layout_objects(&db, layout_id, limit.unwrap_or(-1), offset.unwrap_or(0))
        .map_err(CommandError::from)
}

/// レイアウトオブジェクトの条件付き書式ルール一覧を返す。
#[tauri::command]
pub async fn list_layout_object_conditions(
    state: tauri::State<'_, AppState>,
    object_id: i64,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ConditionRow>, CommandError> {
    let db = super::lock_db(&state)?;
    db_list_layout_object_conditions(&db, object_id, limit.unwrap_or(-1), offset.unwrap_or(0))
        .map_err(CommandError::from)
}

/// プロジェクトのバリューリスト一覧を返す。
#[tauri::command]
pub async fn list_value_lists(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ValueListRow>, CommandError> {
    let db = super::lock_db(&state)?;
    db_list_value_lists(&db, project_id, limit.unwrap_or(-1), offset.unwrap_or(0))
        .map_err(CommandError::from)
}

/// バリューリストの値一覧を返す。
#[tauri::command]
pub async fn list_value_list_items(
    state: tauri::State<'_, AppState>,
    value_list_id: i64,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<String>, CommandError> {
    let db = super::lock_db(&state)?;
    db_list_value_list_items(&db, value_list_id, limit.unwrap_or(-1), offset.unwrap_or(0))
        .map_err(CommandError::from)
}

/// プロジェクトのカスタム関数一覧を返す。
#[tauri::command]
pub async fn list_custom_functions(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<CustomFunctionRow>, CommandError> {
    let db = super::lock_db(&state)?;
    db_list_custom_functions(&db, project_id, limit.unwrap_or(-1), offset.unwrap_or(0))
        .map_err(CommandError::from)
}

/// プロジェクトのテーブルオカレンス一覧を返す（名前順）。
#[tauri::command]
pub async fn list_table_occurrences(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<TableOccurrenceRow>, CommandError> {
    let db = super::lock_db(&state)?;
    db_list_table_occurrences(&db, project_id, limit.unwrap_or(-1), offset.unwrap_or(0))
        .map_err(CommandError::from)
}

/// プロジェクトのリレーション一覧を predicates 込みで返す（名前順）。
#[tauri::command]
pub async fn list_relationships(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<RelationshipRow>, CommandError> {
    let db = super::lock_db(&state)?;
    db_list_relationships(&db, project_id, limit.unwrap_or(-1), offset.unwrap_or(0))
        .map_err(CommandError::from)
}

/// プロジェクトのアカウント一覧を返す（名前順）。
#[tauri::command]
pub async fn list_accounts(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<AccountRow>, CommandError> {
    let db = super::lock_db(&state)?;
    db_list_accounts(&db, project_id, limit.unwrap_or(-1), offset.unwrap_or(0))
        .map_err(CommandError::from)
}

/// プロジェクトの権限セット一覧を返す（名前順）。
#[tauri::command]
pub async fn list_privilege_sets(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<PrivilegeSetRow>, CommandError> {
    let db = super::lock_db(&state)?;
    db_list_privilege_sets(&db, project_id, limit.unwrap_or(-1), offset.unwrap_or(0))
        .map_err(CommandError::from)
}

/// プロジェクトの外部データソース一覧を返す（名前順）。
#[tauri::command]
pub async fn list_external_data_sources(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ExternalDataSourceRow>, CommandError> {
    let db = super::lock_db(&state)?;
    db_list_external_data_sources(&db, project_id, limit.unwrap_or(-1), offset.unwrap_or(0))
        .map_err(CommandError::from)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::repository::{insert_ddr_file, insert_solution};
    use crate::db::Database;
    use crate::parser::parse_ddr;

    const MINIMAL_XML: &str = include_str!("../../../tests/fixtures/minimal.xml");

    fn setup() -> (Database, i64) {
        let mut db = Database::open_in_memory().unwrap();
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let sid = insert_solution(&mut db, &ddr.file_name, None).unwrap();
        let pid = insert_ddr_file(&mut db, &ddr, sid, None).unwrap();
        (db, pid)
    }

    #[test]
    fn list_tables_command_returns_tables() {
        let (db, pid) = setup();
        let tables = db_list_tables(&db, pid, -1, 0).unwrap();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].name, "Contact");
        assert_eq!(tables[0].field_count, 1);
    }

    #[test]
    fn list_table_fields_command_returns_fields() {
        let (db, pid) = setup();
        let tables = db_list_tables(&db, pid, -1, 0).unwrap();
        let table_id = tables[0].id;
        let fields = db_list_table_fields(&db, pid, table_id, -1, 0).unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "FirstName");
        assert_eq!(fields[0].data_type, "Text");
    }

    #[test]
    fn list_all_fields_returns_all_fields_with_table_name() {
        let (db, pid) = setup();
        let fields = list_all_fields_inner(&db, pid, -1, 0).unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "FirstName");
        assert_eq!(fields[0].table_name, "Contact");
    }

    #[test]
    fn list_scripts_command_returns_scripts() {
        let (db, pid) = setup();
        let scripts = db_list_scripts(&db, pid, -1, 0).unwrap();
        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0].step_count, 2);
    }

    #[test]
    fn list_script_steps_command_returns_steps() {
        let (db, pid) = setup();
        let scripts = db_list_scripts(&db, pid, -1, 0).unwrap();
        let script_id = scripts[0].id;
        let steps = db_list_script_steps(&db, script_id, -1, 0).unwrap();
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].name, "Perform Script");
        assert!(steps[0].enabled);
    }

    #[test]
    fn list_layouts_command_returns_layouts() {
        let (db, pid) = setup();
        let layouts = db_list_layouts(&db, pid, -1, 0).unwrap();
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].trigger_count, 1);
    }

    #[test]
    fn list_layout_triggers_command_returns_triggers() {
        let (db, pid) = setup();
        let layouts = db_list_layouts(&db, pid, -1, 0).unwrap();
        let layout_id = layouts[0].id;
        let triggers = db_list_layout_triggers(&db, layout_id, -1, 0).unwrap();
        assert_eq!(triggers.len(), 1);
        assert_eq!(triggers[0].event, "OnRecordLoad");
    }

    #[test]
    fn list_value_lists_command_returns_value_lists() {
        let (db, pid) = setup();
        let vls = db_list_value_lists(&db, pid, -1, 0).unwrap();
        assert_eq!(vls.len(), 1);
        assert_eq!(vls[0].item_count, 2);
    }

    #[test]
    fn list_value_list_items_command_returns_items() {
        let (db, pid) = setup();
        let vls = db_list_value_lists(&db, pid, -1, 0).unwrap();
        let vl_id = vls[0].id;
        let items = db_list_value_list_items(&db, vl_id, -1, 0).unwrap();
        assert_eq!(items, vec!["Active", "Inactive"]);
    }

    #[test]
    fn list_custom_functions_command_returns_functions() {
        let (db, pid) = setup();
        let cfs = db_list_custom_functions(&db, pid, -1, 0).unwrap();
        assert_eq!(cfs.len(), 1);
        assert_eq!(cfs[0].name, "MyFunc");
    }

    #[test]
    fn list_table_occurrences_command_returns_occurrences() {
        let (db, pid) = setup();
        let tos = db_list_table_occurrences(&db, pid, -1, 0).unwrap();
        assert_eq!(tos.len(), 1);
        assert_eq!(tos[0].occurrence_name, "Contact");
        assert_eq!(tos[0].base_table_name, "Contact");
    }

    #[test]
    fn list_relationships_command_returns_relationships() {
        let (db, pid) = setup();
        let rels = db_list_relationships(&db, pid, -1, 0).unwrap();
        assert_eq!(rels.len(), 1);
        assert!(rels[0].name.contains("ContactID"));
        assert_eq!(rels[0].predicates.len(), 1);
    }

    #[test]
    fn list_accounts_command_returns_accounts() {
        let (db, pid) = setup();
        let accounts = db_list_accounts(&db, pid, -1, 0).unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].name, "Admin");
    }

    #[test]
    fn list_privilege_sets_command_returns_privilege_sets() {
        let (db, pid) = setup();
        let ps = db_list_privilege_sets(&db, pid, -1, 0).unwrap();
        assert_eq!(ps.len(), 1);
        assert_eq!(ps[0].name, "[Full Access]");
    }

    #[test]
    fn list_layout_objects_command_returns_empty_for_minimal() {
        let (db, pid) = setup();
        let layouts = db_list_layouts(&db, pid, -1, 0).unwrap();
        let layout_id = layouts[0].id;
        let objects = db_list_layout_objects(&db, layout_id, -1, 0).unwrap();
        assert!(objects.is_empty());
    }

    #[test]
    fn list_layout_object_conditions_returns_empty() {
        let (db, pid) = setup();
        let layouts = db_list_layouts(&db, pid, -1, 0).unwrap();
        let layout_id = layouts[0].id;
        let objects = db_list_layout_objects(&db, layout_id, -1, 0).unwrap();
        assert!(objects.is_empty());
        let conditions = db_list_layout_object_conditions(&db, 9999, -1, 0).unwrap();
        assert!(conditions.is_empty());
    }

    #[test]
    fn list_all_fields_returns_empty_for_empty_project() {
        let db = Database::open_in_memory().unwrap();
        let fields = list_all_fields_inner(&db, 9999, -1, 0).unwrap();
        assert!(fields.is_empty());
    }

    #[test]
    fn list_table_occurrences_returns_empty_for_empty_project() {
        let db = Database::open_in_memory().unwrap();
        let tos = db_list_table_occurrences(&db, 9999, -1, 0).unwrap();
        assert!(tos.is_empty());
    }

    #[test]
    fn list_relationships_returns_empty_for_empty_project() {
        let db = Database::open_in_memory().unwrap();
        let rels = db_list_relationships(&db, 9999, -1, 0).unwrap();
        assert!(rels.is_empty());
    }

    // Pagination tests
    #[test]
    fn list_tables_command_offset_skips_row() {
        let (db, pid) = setup();
        let empty = db_list_tables(&db, pid, -1, 1).unwrap();
        assert!(empty.is_empty(), "offset=1 で唯一のテーブルをスキップ");
    }

    #[test]
    fn list_all_fields_inner_offset_skips_row() {
        let (db, pid) = setup();
        let empty = list_all_fields_inner(&db, pid, -1, 1).unwrap();
        assert!(empty.is_empty(), "offset=1 で唯一のフィールドをスキップ");
    }

    #[test]
    fn list_all_fields_inner_neg_limit_returns_all() {
        let (db, pid) = setup();
        let all = list_all_fields_inner(&db, pid, -1, 0).unwrap();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn list_scripts_command_limit_zero_returns_empty() {
        let (db, pid) = setup();
        let empty = db_list_scripts(&db, pid, 0, 0).unwrap();
        assert!(empty.is_empty(), "limit=0 は0件");
    }
}
