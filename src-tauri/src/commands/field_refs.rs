//! フィールド参照解析コマンド。

use rusqlite::{params, OptionalExtension as _};
use serde::{Deserialize, Serialize};

use crate::{commands::CommandError, AppState};

// ---------------------------------------------------------------------------
// 公開型
// ---------------------------------------------------------------------------

/// フィールドがリレーションキーとして使用されているリレーションの情報。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldRelKeyRef {
    pub relationship_id: i64,
    pub relationship_name: String,
    pub left_table: String,
    pub right_table: String,
    pub operator: String,
    /// "left" or "right" — このフィールドがキーとして使われている側
    pub side: String,
}

/// 未使用フィールドの情報。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnusedFieldRow {
    pub table_name: String,
    pub field_name: String,
    pub field_type: String,
    pub data_type: String,
    pub field_id: i64,
}

/// フィールドを参照しているスクリプトの一覧。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldRefScript {
    pub script_id: i64,
    pub script_name: String,
}

/// フィールドのテーブルを使用しているレイアウトの一覧。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldRefLayout {
    pub layout_id: i64,
    pub layout_name: String,
}

/// テーブルオカレンス名とフィールド名から解決したフィールドの位置情報。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldLocation {
    pub table_id: i64,
    pub field_id: i64,
    pub table_name: String,
}

/// レイアウトフィールド参照のデバッグ情報。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutRefDebugInfo {
    pub occurrence_count: i64,
    pub layout_field_ref_count: i64,
    pub sample_occurrences: Vec<String>,
    pub sample_field_refs: Vec<String>,
}

// ---------------------------------------------------------------------------
// Tauri コマンド
// ---------------------------------------------------------------------------

/// フィールドを参照しているスクリプトの一覧を返す。
#[tauri::command]
pub async fn get_field_refs(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    table_name: String,
    field_name: String,
) -> Result<Vec<FieldRefScript>, CommandError> {
    if table_name.is_empty() || field_name.is_empty() {
        return Ok(vec![]);
    }
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let pattern = format!("%{}::{}%", table_name, field_name);
    let mut stmt = db
        .conn
        .prepare(
            "SELECT DISTINCT s.id, s.name
             FROM scripts s
             JOIN script_steps ss ON ss.script_id = s.id
             WHERE s.project_id = ?1
               AND (
                 ss.step_text LIKE ?2
                 OR ss.calculation LIKE ?2
               )
             ORDER BY s.name",
        )
        .map_err(CommandError::from)?;
    let rows = stmt
        .query_map(params![project_id, pattern], |row| {
            Ok(FieldRefScript {
                script_id: row.get(0)?,
                script_name: row.get(1)?,
            })
        })
        .map_err(CommandError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(CommandError::from)?;
    Ok(rows)
}

/// フィールドが配置されているレイアウトの一覧を返す。
///
/// 検索手順：
/// 1. `table_name`（ベーステーブル名）に対応するオカレンス名一覧を取得
/// 2. メインテーブル（`table_occurrence_name`）がそのオカレンスの1つであるレイアウトを絞り込む
/// 3. さらに、そのレイアウト上に `field_name` が配置されているかを確認
#[tauri::command]
pub async fn get_field_layout_refs(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    table_name: String,
    field_name: String,
) -> Result<Vec<FieldRefLayout>, CommandError> {
    if table_name.is_empty() || field_name.is_empty() {
        return Ok(vec![]);
    }
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let mut stmt = db
        .conn
        .prepare(
            // layout の main TO が対象ベーステーブルのオカレンスであり、
            // かつそのレイアウト上に対象フィールドが配置されていること（フラット JOIN）
            "SELECT DISTINCT l.id, l.name
             FROM layouts l
             JOIN table_occurrences toc_main
               ON toc_main.project_id = l.project_id
              AND toc_main.occurrence_name = l.table_occurrence_name
              AND toc_main.base_table_name = ?2
             JOIN layout_field_refs lfr
               ON lfr.layout_id = l.id
              AND lfr.field_name = ?3
             JOIN table_occurrences toc_field
               ON toc_field.project_id = ?1
              AND toc_field.occurrence_name = lfr.table_occurrence
              AND toc_field.base_table_name = ?2
             WHERE l.project_id = ?1
             ORDER BY l.position, l.name",
        )
        .map_err(CommandError::from)?;
    let rows = stmt
        .query_map(params![project_id, table_name, field_name], |row| {
            Ok(FieldRefLayout {
                layout_id: row.get(0)?,
                layout_name: row.get(1)?,
            })
        })
        .map_err(CommandError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(CommandError::from)?;
    Ok(rows)
}

/// テーブルオカレンス名とフィールド名からフィールドの DB ID・テーブル DB ID・ベーステーブル名を解決する。
#[tauri::command]
pub async fn resolve_layout_field(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    occurrence_name: String,
    field_name: String,
) -> Result<Option<FieldLocation>, CommandError> {
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let result = db
        .conn
        .query_row(
            "SELECT bt.id, f.id, bt.name
               FROM fields f
               JOIN base_tables bt ON bt.id = f.table_id
               JOIN table_occurrences toc
                 ON toc.base_table_name = bt.name
                AND toc.project_id = bt.project_id
              WHERE bt.project_id = ?1
                AND toc.occurrence_name = ?2
                AND f.name = ?3
              LIMIT 1",
            params![project_id, occurrence_name, field_name],
            |row| {
                Ok(FieldLocation {
                    table_id: row.get(0)?,
                    field_id: row.get(1)?,
                    table_name: row.get(2)?,
                })
            },
        )
        .optional()
        .map_err(CommandError::from)?;
    Ok(result)
}

/// フィールドがリレーションキーとして使用されているリレーション一覧を返す。
///
/// `table_name` はベーステーブル名。
/// `join_predicates` を `relationships` / `table_occurrences` と結合して検索する。
#[tauri::command]
pub async fn get_field_relationship_keys(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    table_name: String,
    field_name: String,
) -> Result<Vec<FieldRelKeyRef>, CommandError> {
    if table_name.is_empty() || field_name.is_empty() {
        return Ok(vec![]);
    }
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let mut stmt = db
        .conn
        .prepare(
            "SELECT r.id, r.name, r.left_table, r.right_table, jp.operator, 'left'
             FROM relationships r
             JOIN join_predicates jp ON jp.relationship_id = r.id
             JOIN table_occurrences toc
               ON toc.occurrence_name = r.left_table AND toc.project_id = r.project_id
             WHERE r.project_id = ?1
               AND jp.left_field = ?3
               AND toc.base_table_name = ?2
             UNION
             SELECT r.id, r.name, r.left_table, r.right_table, jp.operator, 'right'
             FROM relationships r
             JOIN join_predicates jp ON jp.relationship_id = r.id
             JOIN table_occurrences toc
               ON toc.occurrence_name = r.right_table AND toc.project_id = r.project_id
             WHERE r.project_id = ?1
               AND jp.right_field = ?3
               AND toc.base_table_name = ?2
             ORDER BY r.name",
        )
        .map_err(CommandError::from)?;
    let rows = stmt
        .query_map(params![project_id, table_name, field_name], |row| {
            Ok(FieldRelKeyRef {
                relationship_id: row.get(0)?,
                relationship_name: row.get(1)?,
                left_table: row.get(2)?,
                right_table: row.get(3)?,
                operator: row.get(4)?,
                side: row.get(5)?,
            })
        })
        .map_err(CommandError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(CommandError::from)?;
    Ok(rows)
}

/// レイアウト・リレーションから一度も参照されていないフィールドの一覧を返す。
///
/// 検査対象: layout_field_refs / layout_objects / join_predicates
/// ※ スクリプトステップ内のテキスト参照は対象外。
#[tauri::command]
pub async fn list_unused_fields(
    state: tauri::State<'_, AppState>,
    project_id: i64,
) -> Result<Vec<UnusedFieldRow>, CommandError> {
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let mut stmt = db
        .conn
        .prepare(
            "WITH used_fields AS (
               -- レイアウト上に直接配置されたフィールド
               SELECT DISTINCT toc.base_table_name AS tbl, lfr.field_name
               FROM layout_field_refs lfr
               JOIN layouts l ON l.id = lfr.layout_id AND l.project_id = ?1
               JOIN table_occurrences toc
                 ON toc.occurrence_name = lfr.table_occurrence AND toc.project_id = ?1
               UNION
               -- レイアウトオブジェクトのフィールド参照
               SELECT DISTINCT toc.base_table_name AS tbl, lo.field_name
               FROM layout_objects lo
               JOIN layouts l ON l.id = lo.layout_id AND l.project_id = ?1
               JOIN table_occurrences toc
                 ON toc.occurrence_name = lo.field_table_occurrence AND toc.project_id = ?1
               WHERE lo.field_name IS NOT NULL AND lo.field_name != ''
               UNION
               -- リレーション結合キー（左側）
               SELECT DISTINCT toc.base_table_name AS tbl, jp.left_field
               FROM join_predicates jp
               JOIN relationships r ON r.id = jp.relationship_id AND r.project_id = ?1
               JOIN table_occurrences toc
                 ON toc.occurrence_name = r.left_table AND toc.project_id = ?1
               WHERE jp.left_field != ''
               UNION
               -- リレーション結合キー（右側）
               SELECT DISTINCT toc.base_table_name AS tbl, jp.right_field
               FROM join_predicates jp
               JOIN relationships r ON r.id = jp.relationship_id AND r.project_id = ?1
               JOIN table_occurrences toc
                 ON toc.occurrence_name = r.right_table AND toc.project_id = ?1
               WHERE jp.right_field != ''
             )
             SELECT bt.name, f.name, f.field_type, f.data_type, f.id
             FROM fields f
             JOIN base_tables bt ON bt.id = f.table_id AND bt.project_id = ?1
             WHERE f.field_type NOT IN ('Summary')
               AND NOT EXISTS (
                 SELECT 1 FROM used_fields uf
                 WHERE uf.tbl = bt.name AND uf.field_name = f.name
               )
             ORDER BY bt.name, f.name",
        )
        .map_err(CommandError::from)?;
    let rows = stmt
        .query_map(params![project_id], |row| {
            Ok(UnusedFieldRow {
                table_name: row.get(0)?,
                field_name: row.get(1)?,
                field_type: row.get(2)?,
                data_type: row.get(3)?,
                field_id: row.get(4)?,
            })
        })
        .map_err(CommandError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(CommandError::from)?;
    Ok(rows)
}

/// レイアウトフィールド参照のデバッグ情報を返す。
#[tauri::command]
pub async fn get_layout_ref_debug_info(
    state: tauri::State<'_, AppState>,
    project_id: i64,
) -> Result<LayoutRefDebugInfo, CommandError> {
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;

    let occurrence_count: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM table_occurrences WHERE project_id = ?1",
            params![project_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let layout_field_ref_count: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM layout_field_refs lfr
             JOIN layouts l ON l.id = lfr.layout_id
             WHERE l.project_id = ?1",
            params![project_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let mut stmt = db
        .conn
        .prepare(
            "SELECT occurrence_name || ' -> ' || base_table_name
             FROM table_occurrences WHERE project_id = ?1 LIMIT 10",
        )
        .map_err(CommandError::from)?;
    let sample_occurrences = stmt
        .query_map(params![project_id], |r| r.get::<_, String>(0))
        .map_err(CommandError::from)?
        .filter_map(|r| r.ok())
        .collect();

    let mut stmt2 = db
        .conn
        .prepare(
            "SELECT l.name || ' | ' || lfr.table_occurrence || '::' || lfr.field_name
             FROM layout_field_refs lfr
             JOIN layouts l ON l.id = lfr.layout_id
             WHERE l.project_id = ?1 LIMIT 10",
        )
        .map_err(CommandError::from)?;
    let sample_field_refs = stmt2
        .query_map(params![project_id], |r| r.get::<_, String>(0))
        .map_err(CommandError::from)?
        .filter_map(|r| r.ok())
        .collect();

    Ok(LayoutRefDebugInfo {
        occurrence_count,
        layout_field_ref_count,
        sample_occurrences,
        sample_field_refs,
    })
}
