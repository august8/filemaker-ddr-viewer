use rusqlite::params;

use crate::{commands::CommandError, AppState};

use super::FieldRelKeyRef;

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
    let db = crate::commands::lock_db(&state)?;
    get_field_relationship_keys_inner(&db.conn, project_id, &table_name, &field_name)
        .map_err(CommandError::from)
}

// ---------------------------------------------------------------------------
// inner 関数（テスト可能・Tauri State 非依存）
// ---------------------------------------------------------------------------

/// `get_field_relationship_keys` の内部実装。
fn get_field_relationship_keys_inner(
    conn: &rusqlite::Connection,
    project_id: i64,
    table_name: &str,
    field_name: &str,
) -> Result<Vec<FieldRelKeyRef>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT r.id, r.name, r.left_table, r.right_table, jp.operator, 'left', r.project_id
         FROM relationships r
         JOIN join_predicates jp ON jp.relationship_id = r.id
         JOIN table_occurrences toc
           ON toc.occurrence_name = r.left_table AND toc.project_id = r.project_id
         WHERE r.project_id IN (
           SELECT id FROM projects
           WHERE solution_id = (SELECT solution_id FROM projects WHERE id = ?1)
         )
           AND jp.left_field = ?3
           AND toc.base_table_name = ?2
         UNION
         SELECT r.id, r.name, r.left_table, r.right_table, jp.operator, 'right', r.project_id
         FROM relationships r
         JOIN join_predicates jp ON jp.relationship_id = r.id
         JOIN table_occurrences toc
           ON toc.occurrence_name = r.right_table AND toc.project_id = r.project_id
         WHERE r.project_id IN (
           SELECT id FROM projects
           WHERE solution_id = (SELECT solution_id FROM projects WHERE id = ?1)
         )
           AND jp.right_field = ?3
           AND toc.base_table_name = ?2
         ORDER BY r.name",
    )?;
    let rows = stmt
        .query_map(params![project_id, table_name, field_name], |row| {
            Ok(FieldRelKeyRef {
                relationship_id: row.get(0)?,
                relationship_name: row.get(1)?,
                left_table: row.get(2)?,
                right_table: row.get(3)?,
                operator: row.get(4)?,
                side: row.get(5)?,
                project_id: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::super::helpers::test_helpers::setup;
    use super::*;

    fn setup_with_relationships() -> (Connection, i64) {
        let (conn, project_id) = setup();

        // リレーション: Invoice::Amount = Order::Total
        conn.execute(
            "INSERT INTO relationships(project_id, fm_id, name, left_table, right_table)
             VALUES(?1, 1, 'Inv_Order', 'Invoice', 'Order')",
            rusqlite::params![project_id],
        )
        .unwrap();
        let rel_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO join_predicates(relationship_id, left_field, right_field, operator, position)
             VALUES(?1, 'Amount', 'Total', '=', 0)",
            rusqlite::params![rel_id],
        )
        .unwrap();

        (conn, project_id)
    }

    #[test]
    fn rel_keys_detects_left_side_key() {
        let (conn, project_id) = setup_with_relationships();
        let refs =
            get_field_relationship_keys_inner(&conn, project_id, "Invoice", "Amount").unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].relationship_name, "Inv_Order");
        assert_eq!(refs[0].side, "left");
    }

    #[test]
    fn rel_keys_detects_right_side_key() {
        let (conn, project_id) = setup_with_relationships();
        let refs = get_field_relationship_keys_inner(&conn, project_id, "Order", "Total").unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].relationship_name, "Inv_Order");
        assert_eq!(refs[0].side, "right");
    }

    #[test]
    fn rel_keys_returns_empty_when_no_match() {
        let (conn, project_id) = setup_with_relationships();
        let refs = get_field_relationship_keys_inner(&conn, project_id, "Invoice", "Note").unwrap();
        assert!(refs.is_empty());
    }
}
