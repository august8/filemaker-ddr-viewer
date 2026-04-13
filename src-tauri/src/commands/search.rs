//! FTS5 全文検索コマンド。

use crate::{
    commands::CommandError,
    db::repository::{search, search_contains, SearchResult},
    AppState,
};

// ---------------------------------------------------------------------------
// Tauri コマンド
// ---------------------------------------------------------------------------

/// プロジェクト内、ソリューション内、または全体を検索する。
///
/// スコープ:
/// - `project_id=Some, solution_id=None` → プロジェクト内のみ
/// - `solution_id=Some` → ソリューション内全プロジェクト横断
/// - 両方 None → DB 内すべてを検索（全体）
///
/// マッチモード:
/// - `contains=None/false` → FTS5 前方一致（name + content 全カラム）
/// - `contains=true` → LIKE 部分一致（名前のみ）
#[tauri::command]
pub async fn search_elements(
    state: tauri::State<'_, AppState>,
    project_id: Option<i64>,
    query: String,
    limit: Option<usize>,
    contains: Option<bool>,
    solution_id: Option<i64>,
) -> Result<Vec<SearchResult>, CommandError> {
    // None または 0 → -1（SQLite LIMIT -1 = 全件）
    let sql_limit: i64 = match limit {
        Some(0) | None => -1,
        Some(n) => n as i64,
    };
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;

    if contains == Some(true) {
        search_contains(&db, project_id, solution_id, &query, sql_limit).map_err(CommandError::from)
    } else {
        search(&db, project_id, solution_id, &query, sql_limit).map_err(CommandError::from)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::repository::{insert_ddr_file, insert_solution};
    use crate::{db::Database, parser::parse_ddr};

    const MINIMAL_XML: &str = include_str!("../../../tests/fixtures/minimal.xml");

    fn setup() -> (Database, i64) {
        let mut db = Database::open_in_memory().unwrap();
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let sid = insert_solution(&mut db, &ddr.file_name, None).unwrap();
        let pid = insert_ddr_file(&mut db, &ddr, sid, None).unwrap();
        (db, pid)
    }

    #[test]
    fn search_finds_contact_table() {
        let (db, pid) = setup();
        let results = search(&db, Some(pid), None, "Contact", 10).unwrap();
        assert!(results.iter().any(|r| r.element_type == "table"));
    }

    #[test]
    fn search_all_scope() {
        let (db, _pid) = setup();
        let results = search(&db, None, None, "Contact", 10).unwrap();
        assert!(results.iter().any(|r| r.element_type == "table"));
    }

    #[test]
    fn search_empty_query_returns_empty() {
        let (db, pid) = setup();
        assert!(search(&db, Some(pid), None, "", 10).unwrap().is_empty());
    }

    #[test]
    fn search_respects_limit() {
        let (db, pid) = setup();
        let results = search(&db, Some(pid), None, "a", 1).unwrap();
        assert!(results.len() <= 1);
    }

    #[test]
    fn search_wrong_project_id_returns_empty() {
        let (db, _) = setup();
        let results = search(&db, Some(9999), None, "Contact", 10).unwrap();
        assert!(results.is_empty());
    }
}
