use rusqlite::params;

use crate::{commands::CommandError, AppState};

use super::LayoutRefDebugInfo;

/// レイアウトフィールド参照のデバッグ情報を返す。
#[tauri::command]
pub async fn get_layout_ref_debug_info(
    state: tauri::State<'_, AppState>,
    project_id: i64,
) -> Result<LayoutRefDebugInfo, CommandError> {
    let db = crate::commands::lock_db(&state)?;
    get_layout_ref_debug_info_inner(&db.conn, project_id).map_err(CommandError::from)
}

// ---------------------------------------------------------------------------
// inner 関数（テスト可能・Tauri State 非依存）
// ---------------------------------------------------------------------------

/// `get_layout_ref_debug_info` の内部実装。
fn get_layout_ref_debug_info_inner(
    conn: &rusqlite::Connection,
    project_id: i64,
) -> Result<LayoutRefDebugInfo, rusqlite::Error> {
    let occurrence_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM table_occurrences WHERE project_id = ?1",
            params![project_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let layout_field_ref_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM layout_field_refs lfr
             JOIN layouts l ON l.id = lfr.layout_id
             WHERE l.project_id = ?1",
            params![project_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let mut stmt = conn.prepare(
        "SELECT occurrence_name || ' -> ' || base_table_name
         FROM table_occurrences WHERE project_id = ?1 LIMIT 10",
    )?;
    let sample_occurrences = stmt
        .query_map(params![project_id], |r| r.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();

    let mut stmt2 = conn.prepare(
        "SELECT l.name || ' | ' || lfr.table_occurrence || '::' || lfr.field_name
         FROM layout_field_refs lfr
         JOIN layouts l ON l.id = lfr.layout_id
         WHERE l.project_id = ?1 LIMIT 10",
    )?;
    let sample_field_refs = stmt2
        .query_map(params![project_id], |r| r.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(LayoutRefDebugInfo {
        occurrence_count,
        layout_field_ref_count,
        sample_occurrences,
        sample_field_refs,
    })
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::super::helpers::test_helpers::setup_with_layout_refs;
    use super::*;

    #[test]
    fn debug_info_counts_occurrences_and_refs() {
        let (conn, project_id) = setup_with_layout_refs();
        let info = get_layout_ref_debug_info_inner(&conn, project_id).unwrap();
        // setup() で 3 オカレンス（Invoice, InvoiceAlias, Order）
        assert_eq!(info.occurrence_count, 3);
        // setup_with_layout_refs() で 2 件の layout_field_refs
        assert_eq!(info.layout_field_ref_count, 2);
        assert!(!info.sample_occurrences.is_empty());
        assert!(!info.sample_field_refs.is_empty());
    }

    #[test]
    fn debug_info_returns_zero_for_empty_project() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::schema::initialize(&conn).unwrap();
        conn.execute("INSERT INTO solutions(name) VALUES('s')", [])
            .unwrap();
        let sid = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO projects(solution_id, name, fm_version) VALUES(?1, 'p', '19')",
            [sid],
        )
        .unwrap();
        let project_id = conn.last_insert_rowid();

        let info = get_layout_ref_debug_info_inner(&conn, project_id).unwrap();
        assert_eq!(info.occurrence_count, 0);
        assert_eq!(info.layout_field_ref_count, 0);
        assert!(info.sample_occurrences.is_empty());
        assert!(info.sample_field_refs.is_empty());
    }
}
