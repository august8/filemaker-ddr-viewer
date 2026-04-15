//! ナビゲーション用の list_* クエリ群。

use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::db::{Database, DbError};

// ---------------------------------------------------------------------------
// 公開データ型
// ---------------------------------------------------------------------------

/// DB から取得したテーブル行（field_count 付き）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    pub id: i64,
    pub fm_id: i64,
    pub name: String,
    pub field_count: i64,
}

/// DB から取得したフィールド行。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldRow {
    pub id: i64,
    pub fm_id: i64,
    pub name: String,
    pub data_type: String,
    pub field_type: String,
    pub comment: String,
    pub is_global: bool,
    pub max_repeat: i64,
    pub calculation: Option<String>,
    pub auto_enter_type: String,
    pub auto_enter_calc: Option<String>,
    pub auto_enter_allow_editing: bool,
    pub val_not_empty: bool,
    pub val_unique: bool,
    pub val_existing: bool,
    pub val_max_length: Option<i64>,
    pub val_value_list: Option<String>,
    pub val_calc: Option<String>,
    pub val_range_from: Option<String>,
    pub val_range_to: Option<String>,
    pub val_always: bool,
    pub val_error_message: Option<String>,
    pub index_type: String,
    pub container_storage: Option<String>,
}

/// DB から取得したスクリプト行（step_count 付き）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptRow {
    pub id: i64,
    pub fm_id: i64,
    pub name: String,
    pub run_with_full_access: bool,
    pub step_count: i64,
}

/// DB から取得したスクリプトステップ行。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptStepRow {
    pub id: i64,
    pub step_type_id: i64,
    pub name: String,
    pub enabled: bool,
    pub script_ref_name: Option<String>,
    pub script_ref_file: Option<String>,
    pub calculation: Option<String>,
    pub step_text: Option<String>,
    pub position: i64,
}

/// DB から取得したレイアウト行（trigger_count 付き）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutRow {
    pub id: i64,
    pub fm_id: i64,
    pub name: String,
    pub table_occurrence_name: Option<String>,
    pub trigger_count: i64,
}

/// DB から取得したレイアウトオブジェクト行。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutObjectRow {
    pub id: i64,
    pub object_type: String,
    pub object_key: i64,
    pub object_name: Option<String>,
    pub button_label: Option<String>,
    pub field_table_occurrence: Option<String>,
    pub field_name: Option<String>,
    pub tooltip: Option<String>,
    pub hide_condition: Option<String>,
    pub position: i64,
    pub bound_top: Option<f64>,
    pub bound_left: Option<f64>,
    pub bound_bottom: Option<f64>,
    pub bound_right: Option<f64>,
}

/// DB から取得した条件付き書式行。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionRow {
    pub id: i64,
    pub rule_order: i64,
    pub calculation: String,
    pub format_css: String,
}

/// DB から取得したトリガー行。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerRow {
    pub id: i64,
    pub event: String,
    pub script_name: String,
    pub file_name: String,
}

/// DB から取得したバリューリスト行（item_count 付き）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueListRow {
    pub id: i64,
    pub fm_id: i64,
    pub name: String,
    pub source: String,
    pub item_count: i64,
}

/// DB から取得したカスタム関数行。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFunctionRow {
    pub id: i64,
    pub fm_id: i64,
    pub name: String,
    pub parameters: String,
    pub calculation: Option<String>,
}

/// DB から取得したテーブルオカレンス行。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableOccurrenceRow {
    pub id: i64,
    pub occurrence_name: String,
    pub base_table_name: String,
    /// 外部ファイル参照元ファイル名。空文字列 = 自ファイルのテーブル
    pub source_file: String,
}

/// DB から取得した結合条件行。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredicateRow {
    pub id: i64,
    pub left_field: String,
    pub right_field: String,
    pub operator: String,
    pub position: i64,
}

/// DB から取得したリレーション行（結合条件を含む）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipRow {
    pub id: i64,
    pub fm_id: i64,
    pub name: String,
    pub left_table: String,
    pub right_table: String,
    pub predicates: Vec<PredicateRow>,
}

/// DB から取得したアカウント行。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountRow {
    pub id: i64,
    pub fm_id: i64,
    pub name: String,
    pub privilege_set: Option<String>,
    pub enabled: bool,
}

/// DB から取得した権限セット行。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivilegeSetRow {
    pub id: i64,
    pub fm_id: i64,
    pub name: String,
    pub comment: Option<String>,
}

// ---------------------------------------------------------------------------
// ナビゲーション用クエリ関数
// ---------------------------------------------------------------------------

/// プロジェクトのテーブル一覧を field_count 付きで返す（名前順）。
pub fn list_tables(db: &Database, project_id: i64) -> Result<Vec<TableRow>, DbError> {
    let mut stmt = db.conn.prepare(
        "SELECT t.id, t.fm_id, t.name, COUNT(f.id) as field_count
           FROM base_tables t
           LEFT JOIN fields f ON f.table_id = t.id
          WHERE t.project_id = ?1
          GROUP BY t.id
          ORDER BY t.name",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(TableRow {
            id: row.get(0)?,
            fm_id: row.get(1)?,
            name: row.get(2)?,
            field_count: row.get(3)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// テーブルに属するフィールド一覧を返す（名前順）。
pub fn list_table_fields(
    db: &Database,
    project_id: i64,
    table_db_id: i64,
) -> Result<Vec<FieldRow>, DbError> {
    let mut stmt = db.conn.prepare(
        "SELECT id, fm_id, name, data_type, field_type, comment,
                is_global, max_repeat, calculation,
                auto_enter_type, auto_enter_calc, auto_enter_allow_editing,
                val_not_empty, val_unique, val_existing, val_max_length,
                val_value_list, val_calc, val_range_from, val_range_to,
                val_always, val_error_message, index_type, container_storage
           FROM fields
          WHERE project_id = ?1
            AND table_id = ?2
          ORDER BY name",
    )?;
    let rows = stmt.query_map(params![project_id, table_db_id], |row| {
        Ok(FieldRow {
            id: row.get(0)?,
            fm_id: row.get(1)?,
            name: row.get(2)?,
            data_type: row.get(3)?,
            field_type: row.get(4)?,
            comment: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
            is_global: row.get::<_, i64>(6)? != 0,
            max_repeat: row.get(7)?,
            calculation: row.get(8)?,
            auto_enter_type: row.get::<_, Option<String>>(9)?.unwrap_or_default(),
            auto_enter_calc: row.get(10)?,
            auto_enter_allow_editing: row.get::<_, i64>(11)? != 0,
            val_not_empty: row.get::<_, i64>(12)? != 0,
            val_unique: row.get::<_, i64>(13)? != 0,
            val_existing: row.get::<_, i64>(14)? != 0,
            val_max_length: row.get(15)?,
            val_value_list: row.get(16)?,
            val_calc: row.get(17)?,
            val_range_from: row.get(18)?,
            val_range_to: row.get(19)?,
            val_always: row.get::<_, i64>(20)? != 0,
            val_error_message: row.get(21)?,
            index_type: row.get::<_, Option<String>>(22)?.unwrap_or_default(),
            container_storage: row.get(23)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// プロジェクトのスクリプト一覧を step_count 付きで返す（FileMaker順）。
pub fn list_scripts(db: &Database, project_id: i64) -> Result<Vec<ScriptRow>, DbError> {
    let mut stmt = db.conn.prepare(
        "SELECT s.id, s.fm_id, s.name, s.run_with_full_access,
                (SELECT COUNT(*) FROM script_steps WHERE script_id = s.id) as step_count
           FROM scripts s
          WHERE s.project_id = ?1
          ORDER BY s.position",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(ScriptRow {
            id: row.get(0)?,
            fm_id: row.get(1)?,
            name: row.get(2)?,
            run_with_full_access: row.get::<_, i64>(3)? != 0,
            step_count: row.get(4)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// スクリプトに属するステップ一覧を返す（position 順）。
pub fn list_script_steps(db: &Database, script_db_id: i64) -> Result<Vec<ScriptStepRow>, DbError> {
    let mut stmt = db.conn.prepare(
        "SELECT id, step_type_id, name, enabled,
                script_ref_name, script_ref_file, calculation, step_text, position
           FROM script_steps
          WHERE script_id = ?1
          ORDER BY position",
    )?;
    let rows = stmt.query_map(params![script_db_id], |row| {
        Ok(ScriptStepRow {
            id: row.get(0)?,
            step_type_id: row.get(1)?,
            name: row.get(2)?,
            enabled: row.get::<_, i64>(3)? != 0,
            script_ref_name: row.get(4)?,
            script_ref_file: row.get(5)?,
            calculation: row.get(6)?,
            step_text: row.get(7)?,
            position: row.get(8)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// プロジェクトのレイアウト一覧を trigger_count 付きで返す（FileMaker順）。
pub fn list_layouts(db: &Database, project_id: i64) -> Result<Vec<LayoutRow>, DbError> {
    let mut stmt = db.conn.prepare(
        "SELECT l.id, l.fm_id, l.name, l.table_occurrence_name,
                (SELECT COUNT(*) FROM script_triggers WHERE layout_id = l.id) as trigger_count
           FROM layouts l
          WHERE l.project_id = ?1
          ORDER BY l.position",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(LayoutRow {
            id: row.get(0)?,
            fm_id: row.get(1)?,
            name: row.get(2)?,
            table_occurrence_name: row.get(3)?,
            trigger_count: row.get(4)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// レイアウトに属するトリガー一覧を返す。
pub fn list_layout_triggers(db: &Database, layout_db_id: i64) -> Result<Vec<TriggerRow>, DbError> {
    let mut stmt = db.conn.prepare(
        "SELECT id, event, script_name, file_name
           FROM script_triggers
          WHERE layout_id = ?1
          ORDER BY event",
    )?;
    let rows = stmt.query_map(params![layout_db_id], |row| {
        Ok(TriggerRow {
            id: row.get(0)?,
            event: row.get(1)?,
            script_name: row.get(2)?,
            file_name: row.get(3)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// レイアウトに属するオブジェクト一覧を返す（position 順）。
pub fn list_layout_objects(
    db: &Database,
    layout_db_id: i64,
) -> Result<Vec<LayoutObjectRow>, DbError> {
    let mut stmt = db.conn.prepare(
        "SELECT id, object_type, object_key, object_name, button_label,
                field_table_occurrence, field_name, tooltip, hide_condition, position,
                bound_top, bound_left, bound_bottom, bound_right
           FROM layout_objects
          WHERE layout_id = ?1
          ORDER BY position",
    )?;
    let rows = stmt.query_map(params![layout_db_id], |row| {
        Ok(LayoutObjectRow {
            id: row.get(0)?,
            object_type: row.get(1)?,
            object_key: row.get(2)?,
            object_name: row.get(3)?,
            button_label: row.get(4)?,
            field_table_occurrence: row.get(5)?,
            field_name: row.get(6)?,
            tooltip: row.get(7)?,
            hide_condition: row.get(8)?,
            position: row.get(9)?,
            bound_top: row.get(10)?,
            bound_left: row.get(11)?,
            bound_bottom: row.get(12)?,
            bound_right: row.get(13)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// レイアウトオブジェクトの条件付き書式ルール一覧を rule_order 順で返す。
pub fn list_layout_object_conditions(
    db: &Database,
    object_id: i64,
) -> Result<Vec<ConditionRow>, DbError> {
    let mut stmt = db.conn.prepare(
        "SELECT id, rule_order, calculation, format_css
           FROM layout_object_conditions
          WHERE object_id = ?1
          ORDER BY rule_order",
    )?;
    let rows = stmt.query_map(params![object_id], |row| {
        Ok(ConditionRow {
            id: row.get(0)?,
            rule_order: row.get(1)?,
            calculation: row.get(2)?,
            format_css: row.get(3)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// プロジェクトのバリューリスト一覧を item_count 付きで返す（名前順）。
pub fn list_value_lists(db: &Database, project_id: i64) -> Result<Vec<ValueListRow>, DbError> {
    let mut stmt = db.conn.prepare(
        "SELECT v.id, v.fm_id, v.name, v.source,
                (SELECT COUNT(*) FROM value_list_items WHERE value_list_id = v.id) as item_count
           FROM value_lists v
          WHERE v.project_id = ?1
          ORDER BY v.name",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(ValueListRow {
            id: row.get(0)?,
            fm_id: row.get(1)?,
            name: row.get(2)?,
            source: row.get(3)?,
            item_count: row.get(4)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// バリューリストに属する値一覧を返す（position 順）。
pub fn list_value_list_items(db: &Database, value_list_db_id: i64) -> Result<Vec<String>, DbError> {
    let mut stmt = db.conn.prepare(
        "SELECT value
           FROM value_list_items
          WHERE value_list_id = ?1
          ORDER BY position",
    )?;
    let rows = stmt.query_map(params![value_list_db_id], |row| row.get::<_, String>(0))?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// プロジェクトのカスタム関数一覧を返す（名前順）。
pub fn list_custom_functions(
    db: &Database,
    project_id: i64,
) -> Result<Vec<CustomFunctionRow>, DbError> {
    let mut stmt = db.conn.prepare(
        "SELECT id, fm_id, name, parameters, calculation
           FROM custom_functions
          WHERE project_id = ?1
          ORDER BY name",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(CustomFunctionRow {
            id: row.get(0)?,
            fm_id: row.get(1)?,
            name: row.get(2)?,
            parameters: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
            calculation: row.get(4)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// プロジェクトのテーブルオカレンス一覧を返す（名前順）。
pub fn list_table_occurrences(
    db: &Database,
    project_id: i64,
) -> Result<Vec<TableOccurrenceRow>, DbError> {
    let mut stmt = db.conn.prepare(
        "SELECT id, occurrence_name, base_table_name, source_file
           FROM table_occurrences
          WHERE project_id = ?1
          ORDER BY occurrence_name",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(TableOccurrenceRow {
            id: row.get(0)?,
            occurrence_name: row.get(1)?,
            base_table_name: row.get(2)?,
            source_file: row.get(3)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// プロジェクトのリレーション一覧を predicates 込みで返す（名前順）。
pub fn list_relationships(db: &Database, project_id: i64) -> Result<Vec<RelationshipRow>, DbError> {
    let mut stmt = db.conn.prepare(
        "SELECT id, fm_id, name, left_table, right_table
           FROM relationships
          WHERE project_id = ?1
          ORDER BY name",
    )?;
    let mut rels: Vec<RelationshipRow> = stmt
        .query_map(params![project_id], |row| {
            Ok(RelationshipRow {
                id: row.get(0)?,
                fm_id: row.get(1)?,
                name: row.get(2)?,
                left_table: row.get(3)?,
                right_table: row.get(4)?,
                predicates: vec![],
            })
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(DbError::from)?;

    for rel in &mut rels {
        let mut pstmt = db.conn.prepare(
            "SELECT id, left_field, right_field, operator, position
               FROM join_predicates
              WHERE relationship_id = ?1
              ORDER BY position",
        )?;
        let preds: Vec<PredicateRow> = pstmt
            .query_map(params![rel.id], |row| {
                Ok(PredicateRow {
                    id: row.get(0)?,
                    left_field: row.get(1)?,
                    right_field: row.get(2)?,
                    operator: row.get(3)?,
                    position: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::from)?;
        rel.predicates = preds;
    }

    Ok(rels)
}

/// プロジェクトのアカウント一覧を返す（名前順）。
pub fn list_accounts(db: &Database, project_id: i64) -> Result<Vec<AccountRow>, DbError> {
    let mut stmt = db.conn.prepare(
        "SELECT id, fm_id, name, privilege_set, enabled
           FROM accounts
          WHERE project_id = ?1
          ORDER BY name",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(AccountRow {
            id: row.get(0)?,
            fm_id: row.get(1)?,
            name: row.get(2)?,
            privilege_set: row.get(3)?,
            enabled: row.get::<_, i64>(4)? != 0,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// プロジェクトの権限セット一覧を返す（名前順）。
pub fn list_privilege_sets(
    db: &Database,
    project_id: i64,
) -> Result<Vec<PrivilegeSetRow>, DbError> {
    let mut stmt = db.conn.prepare(
        "SELECT id, fm_id, name, comment
           FROM privilege_sets
          WHERE project_id = ?1
          ORDER BY name",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(PrivilegeSetRow {
            id: row.get(0)?,
            fm_id: row.get(1)?,
            name: row.get(2)?,
            comment: row.get(3)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

// ---------------------------------------------------------------------------
// アップグレードチェック
// ---------------------------------------------------------------------------

/// アップグレードチェックの1ヒット。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeHit {
    pub item_id: String,
    pub project_id: i64,
    pub project_name: String,
    pub custom_function_name: Option<String>,
    pub script_id: Option<i64>,
    pub script_name: Option<String>,
    pub step_id: Option<i64>,
    pub step_name: Option<String>,
    pub step_text: Option<String>,
    pub field_id: Option<i64>,
    pub field_name: Option<String>,
    pub table_id: Option<i64>,
    pub table_name: Option<String>,
}

/// フロントエンドから渡されるチェック項目設定。
/// フロントエンドは camelCase で送信するため rename_all = "camelCase" が必要。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckItemConfig {
    pub id: String,
    pub detection_type: String, // "step_type_id" | "step_external" | "text_match" | "field_attr"
    pub detection_value: String,
}

/// 有効なチェック項目リストに基づいてヒット一覧を返す（ソリューション単位）。
pub fn run_upgrade_check(
    db: &Database,
    solution_id: i64,
    items: &[CheckItemConfig],
) -> Result<Vec<UpgradeHit>, DbError> {
    let mut all_hits: Vec<UpgradeHit> = Vec::new();

    for item in items {
        let hits = match item.detection_type.as_str() {
            "step_type_id" => {
                let type_id: i64 = item.detection_value.parse().unwrap_or(-1);
                let mut stmt = db.conn.prepare(
                    "SELECT ss.id, ss.name, ss.step_text, s.id, s.name, p.id, p.name
                       FROM script_steps ss
                       JOIN scripts s ON s.id = ss.script_id
                       JOIN projects p ON p.id = s.project_id
                      WHERE p.solution_id = ?1
                        AND ss.step_type_id = ?2",
                )?;
                let rows = stmt.query_map(params![solution_id, type_id], |row| {
                    Ok(UpgradeHit {
                        item_id: item.id.clone(),
                        step_id: row.get(0)?,
                        step_name: row.get(1)?,
                        step_text: row.get(2)?,
                        script_id: row.get(3)?,
                        script_name: row.get(4)?,
                        project_id: row.get(5)?,
                        project_name: row.get(6)?,
                        custom_function_name: None,
                        field_id: None,
                        field_name: None,
                        table_id: None,
                        table_name: None,
                    })
                })?;
                rows.collect::<Result<Vec<_>, _>>()?
            }
            "step_external" => {
                let type_id: i64 = item.detection_value.parse().unwrap_or(-1);
                let mut stmt = db.conn.prepare(
                    "SELECT ss.id, ss.name, ss.step_text, s.id, s.name, p.id, p.name
                       FROM script_steps ss
                       JOIN scripts s ON s.id = ss.script_id
                       JOIN projects p ON p.id = s.project_id
                      WHERE p.solution_id = ?1
                        AND ss.step_type_id = ?2
                        AND ss.script_ref_file IS NOT NULL
                        AND ss.script_ref_file != ''",
                )?;
                let rows = stmt.query_map(params![solution_id, type_id], |row| {
                    Ok(UpgradeHit {
                        item_id: item.id.clone(),
                        step_id: row.get(0)?,
                        step_name: row.get(1)?,
                        step_text: row.get(2)?,
                        script_id: row.get(3)?,
                        script_name: row.get(4)?,
                        project_id: row.get(5)?,
                        project_name: row.get(6)?,
                        custom_function_name: None,
                        field_id: None,
                        field_name: None,
                        table_id: None,
                        table_name: None,
                    })
                })?;
                rows.collect::<Result<Vec<_>, _>>()?
            }
            "text_match" => {
                let like_val = format!("%{}%", item.detection_value);
                // スクリプトステップ（step_text / calculation）
                let mut step_hits: Vec<UpgradeHit> = {
                    let mut stmt = db.conn.prepare(
                        "SELECT ss.id, ss.name, ss.step_text, s.id, s.name, p.id, p.name
                           FROM script_steps ss
                           JOIN scripts s ON s.id = ss.script_id
                           JOIN projects p ON p.id = s.project_id
                          WHERE p.solution_id = ?1
                            AND (ss.step_text LIKE ?2 OR ss.calculation LIKE ?2)",
                    )?;
                    let rows = stmt.query_map(params![solution_id, like_val], |row| {
                        Ok(UpgradeHit {
                            item_id: item.id.clone(),
                            step_id: row.get(0)?,
                            step_name: row.get(1)?,
                            step_text: row.get(2)?,
                            script_id: row.get(3)?,
                            script_name: row.get(4)?,
                            project_id: row.get(5)?,
                            project_name: row.get(6)?,
                            custom_function_name: None,
                            field_id: None,
                            field_name: None,
                            table_id: None,
                            table_name: None,
                        })
                    })?;
                    rows.collect::<Result<Vec<_>, _>>()?
                };
                // フィールド計算式
                let mut field_hits: Vec<UpgradeHit> = {
                    let mut stmt = db.conn.prepare(
                        "SELECT f.id, f.name, bt.id, bt.name, p.id, p.name
                           FROM fields f
                           JOIN base_tables bt ON bt.id = f.table_id
                           JOIN projects p ON p.id = f.project_id
                          WHERE p.solution_id = ?1
                            AND f.calculation LIKE ?2",
                    )?;
                    let rows = stmt.query_map(params![solution_id, like_val], |row| {
                        Ok(UpgradeHit {
                            item_id: item.id.clone(),
                            custom_function_name: None,
                            script_id: None,
                            script_name: None,
                            step_id: None,
                            step_name: None,
                            step_text: None,
                            field_id: row.get(0)?,
                            field_name: row.get(1)?,
                            table_id: row.get(2)?,
                            table_name: row.get(3)?,
                            project_id: row.get(4)?,
                            project_name: row.get(5)?,
                        })
                    })?;
                    rows.collect::<Result<Vec<_>, _>>()?
                };
                step_hits.append(&mut field_hits);
                step_hits
            }
            "field_attr" => {
                let where_clause = match item.detection_value.as_str() {
                    "container" => "f.data_type = 'Container'",
                    "auto_enter_serial" => "f.auto_enter_type = 'Serial'",
                    _ => continue, // 未知の属性はスキップして次の項目へ
                };
                let sql = format!(
                    "SELECT f.id, f.name, bt.id, bt.name, p.id, p.name
                       FROM fields f
                       JOIN base_tables bt ON bt.id = f.table_id
                       JOIN projects p ON p.id = f.project_id
                      WHERE p.solution_id = ?1
                        AND {where_clause}"
                );
                let mut stmt = db.conn.prepare(&sql)?;
                let rows = stmt.query_map(params![solution_id], |row| {
                    Ok(UpgradeHit {
                        item_id: item.id.clone(),
                        custom_function_name: None,
                        script_id: None,
                        script_name: None,
                        step_id: None,
                        step_name: None,
                        step_text: None,
                        field_id: row.get(0)?,
                        field_name: row.get(1)?,
                        table_id: row.get(2)?,
                        table_name: row.get(3)?,
                        project_id: row.get(4)?,
                        project_name: row.get(5)?,
                    })
                })?;
                rows.collect::<Result<Vec<_>, _>>()?
            }
            // ソリューション内の全カスタム関数名を動的に取得して呼び出し箇所を検出
            "any_custom_function" => {
                let cf_names: Vec<String> = {
                    let mut stmt = db.conn.prepare(
                        "SELECT DISTINCT cf.name
                           FROM custom_functions cf
                           JOIN projects p ON p.id = cf.project_id
                          WHERE p.solution_id = ?1
                          ORDER BY cf.name",
                    )?;
                    let rows =
                        stmt.query_map(params![solution_id], |row| row.get::<_, String>(0))?;
                    rows.collect::<Result<_, _>>()?
                };

                let mut cf_hits: Vec<UpgradeHit> = Vec::new();
                for cf_name in &cf_names {
                    let like_val = format!("%{}%", cf_name);

                    // スクリプトステップ
                    let mut stmt = db.conn.prepare(
                        "SELECT ss.id, ss.name, ss.step_text, s.id, s.name, p.id, p.name
                           FROM script_steps ss
                           JOIN scripts s ON s.id = ss.script_id
                           JOIN projects p ON p.id = s.project_id
                          WHERE p.solution_id = ?1
                            AND (ss.step_text LIKE ?2 OR ss.calculation LIKE ?2)",
                    )?;
                    let rows = stmt.query_map(params![solution_id, &like_val], |row| {
                        Ok(UpgradeHit {
                            item_id: item.id.clone(),
                            custom_function_name: Some(cf_name.clone()),
                            step_id: row.get(0)?,
                            step_name: row.get(1)?,
                            step_text: row.get(2)?,
                            script_id: row.get(3)?,
                            script_name: row.get(4)?,
                            project_id: row.get(5)?,
                            project_name: row.get(6)?,
                            field_id: None,
                            field_name: None,
                            table_id: None,
                            table_name: None,
                        })
                    })?;
                    cf_hits.extend(rows.collect::<Result<Vec<_>, _>>()?);

                    // フィールド計算式
                    let mut stmt = db.conn.prepare(
                        "SELECT f.id, f.name, bt.id, bt.name, p.id, p.name
                           FROM fields f
                           JOIN base_tables bt ON bt.id = f.table_id
                           JOIN projects p ON p.id = f.project_id
                          WHERE p.solution_id = ?1
                            AND f.calculation LIKE ?2",
                    )?;
                    let rows = stmt.query_map(params![solution_id, &like_val], |row| {
                        Ok(UpgradeHit {
                            item_id: item.id.clone(),
                            custom_function_name: Some(cf_name.clone()),
                            script_id: None,
                            script_name: None,
                            step_id: None,
                            step_name: None,
                            step_text: None,
                            field_id: row.get(0)?,
                            field_name: row.get(1)?,
                            table_id: row.get(2)?,
                            table_name: row.get(3)?,
                            project_id: row.get(4)?,
                            project_name: row.get(5)?,
                        })
                    })?;
                    cf_hits.extend(rows.collect::<Result<Vec<_>, _>>()?);
                }
                cf_hits
            }
            _ => vec![], // 未知の detection_type はスキップ
        };
        all_hits.extend(hits);
    }

    Ok(all_hits)
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

    const MINIMAL_XML: &str = include_str!("../../../../tests/fixtures/minimal.xml");

    fn db_with_minimal() -> (Database, i64, i64) {
        let mut db = Database::open_in_memory().unwrap();
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let sid = insert_solution(&mut db, "TestSolution", Some("/tmp/test.xml")).unwrap();
        let pid = insert_ddr_file(&mut db, &ddr, sid, Some("/tmp/test.xml")).unwrap();
        (db, sid, pid)
    }

    #[test]
    fn list_tables_returns_with_field_count() {
        let (db, _sid, pid) = db_with_minimal();
        let tables = list_tables(&db, pid).unwrap();
        assert_eq!(tables.len(), 1, "minimal.xml に 1 テーブル");
        assert_eq!(tables[0].field_count, 1, "field_count=1");
        assert_eq!(tables[0].name, "Contact");
    }

    #[test]
    fn list_table_fields_returns_fields() {
        let (db, _sid, pid) = db_with_minimal();
        let tables = list_tables(&db, pid).unwrap();
        let table_db_id = tables[0].id;
        let fields = list_table_fields(&db, pid, table_db_id).unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "FirstName");
        assert_eq!(fields[0].data_type, "Text");
    }

    #[test]
    fn list_scripts_returns_with_step_count() {
        let (db, _sid, pid) = db_with_minimal();
        let scripts = list_scripts(&db, pid).unwrap();
        assert_eq!(scripts.len(), 1, "minimal.xml に 1 スクリプト");
        assert_eq!(scripts[0].step_count, 2, "step_count=2");
    }

    #[test]
    fn list_script_steps_returns_steps() {
        let (db, _sid, pid) = db_with_minimal();
        let scripts = list_scripts(&db, pid).unwrap();
        let script_db_id = scripts[0].id;
        let steps = list_script_steps(&db, script_db_id).unwrap();
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].name, "Perform Script");
        assert!(steps[0].enabled);
    }

    #[test]
    fn list_layouts_returns_with_trigger_count() {
        let (db, _sid, pid) = db_with_minimal();
        let layouts = list_layouts(&db, pid).unwrap();
        assert_eq!(layouts.len(), 1, "minimal.xml に 1 レイアウト");
        assert_eq!(layouts[0].trigger_count, 1, "trigger_count=1");
    }

    #[test]
    fn list_layout_triggers_returns_triggers() {
        let (db, _sid, pid) = db_with_minimal();
        let layouts = list_layouts(&db, pid).unwrap();
        let layout_db_id = layouts[0].id;
        let triggers = list_layout_triggers(&db, layout_db_id).unwrap();
        assert_eq!(triggers.len(), 1);
        assert_eq!(triggers[0].event, "OnRecordLoad");
    }

    #[test]
    fn list_value_lists_returns_with_item_count() {
        let (db, _sid, pid) = db_with_minimal();
        let vls = list_value_lists(&db, pid).unwrap();
        assert_eq!(vls.len(), 1, "minimal.xml に 1 バリューリスト");
        assert_eq!(vls[0].item_count, 2, "item_count=2");
    }

    #[test]
    fn list_value_list_items_returns_items() {
        let (db, _sid, pid) = db_with_minimal();
        let vls = list_value_lists(&db, pid).unwrap();
        let vl_db_id = vls[0].id;
        let items = list_value_list_items(&db, vl_db_id).unwrap();
        assert_eq!(items, vec!["Active", "Inactive"]);
    }

    #[test]
    fn list_custom_functions_returns_functions() {
        let (db, _sid, pid) = db_with_minimal();
        let cfs = list_custom_functions(&db, pid).unwrap();
        assert_eq!(cfs.len(), 1);
        assert_eq!(cfs[0].name, "MyFunc");
    }

    #[test]
    fn list_accounts_returns_admin() {
        let (db, _sid, pid) = db_with_minimal();
        let accounts = list_accounts(&db, pid).unwrap();
        assert_eq!(accounts.len(), 1, "minimal.xml に 1 アカウント");
        assert_eq!(accounts[0].name, "Admin");
        assert_eq!(accounts[0].privilege_set, Some("[Full Access]".to_string()));
        assert!(accounts[0].enabled);
    }

    #[test]
    fn list_privilege_sets_returns_full_access() {
        let (db, _sid, pid) = db_with_minimal();
        let psets = list_privilege_sets(&db, pid).unwrap();
        assert_eq!(psets.len(), 1, "minimal.xml に 1 権限セット");
        assert_eq!(psets[0].name, "[Full Access]");
        assert_eq!(psets[0].comment, Some("Full access".to_string()));
    }

    // ADR-025: run_upgrade_check はソリューション単位で横断検索する
    #[test]
    fn run_upgrade_check_solution_scope() {
        let mut db = Database::open_in_memory().unwrap();
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        // 同一ソリューションに 2 プロジェクトをインポート
        let sid = insert_solution(&mut db, "Sol", None).unwrap();
        let _pid1 = insert_ddr_file(&mut db, &ddr, sid, None).unwrap();
        let _pid2 = insert_ddr_file(&mut db, &ddr, sid, None).unwrap();

        // minimal.xml の "Perform Script" ステップ (step_type_id = 89) を検出
        let items = vec![CheckItemConfig {
            id: "perform_script".to_string(),
            detection_type: "step_type_id".to_string(),
            detection_value: "89".to_string(),
        }];

        let hits = run_upgrade_check(&db, sid, &items).unwrap();
        // 2 プロジェクトそれぞれに 1 ヒット → 合計 2 件
        assert_eq!(
            hits.len(),
            2,
            "solution 配下の全プロジェクトからヒットを返すこと"
        );
        // 各ヒットに project_id / project_name が設定されていること
        assert!(hits.iter().all(|h| !h.project_name.is_empty()));
    }
}
