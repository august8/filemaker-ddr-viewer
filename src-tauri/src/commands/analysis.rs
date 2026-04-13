//! プロジェクト管理・サマリー取得コマンド。

use rusqlite::{params, OptionalExtension as _};
use serde::{Deserialize, Serialize};

use crate::{
    analyzer::{
        broken_refs::{find_broken_refs, BrokenRef},
        report_card::{generate_report_card, ReportCard},
    },
    commands::CommandError,
    db::{
        repository::{
            delete_project as db_delete_project, delete_solution as db_delete_solution,
            get_project, list_projects as db_list_projects, list_solutions as db_list_solutions,
            run_upgrade_check, CheckItemConfig, ProjectRow, SolutionRow, UpgradeHit,
        },
        Database,
    },
    AppState,
};

use super::callchain::get_ddr;

// ---------------------------------------------------------------------------
// 公開型
// ---------------------------------------------------------------------------

/// プロジェクトの統計サマリー。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub project: ProjectRow,
    pub table_count: i64,
    pub field_count: i64,
    pub script_count: i64,
    pub layout_count: i64,
    pub table_occurrence_count: i64,
    pub relationship_count: i64,
    pub value_list_count: i64,
    pub custom_function_count: i64,
    pub account_count: i64,
    pub privilege_set_count: i64,
}

/// 要素名から DB の id を解決する結果型（差分クリックナビゲーション用）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementRef {
    pub id: i64,
    pub name: String,
}

// ---------------------------------------------------------------------------
// ビジネスロジック
// ---------------------------------------------------------------------------

/// プロジェクトの統計サマリーを構築する。
pub(crate) fn build_project_summary(
    db: &Database,
    project_id: i64,
) -> Result<ProjectSummary, CommandError> {
    let project = get_project(db, project_id).map_err(CommandError::from)?;

    let count = |table: &str| -> Result<i64, CommandError> {
        db.conn
            .query_row(
                &format!("SELECT COUNT(*) FROM {table} WHERE project_id = ?1"),
                params![project_id],
                |r| r.get(0),
            )
            .map_err(CommandError::from)
    };

    Ok(ProjectSummary {
        project,
        table_count: count("base_tables")?,
        field_count: count("fields")?,
        script_count: count("scripts")?,
        layout_count: count("layouts")?,
        table_occurrence_count: count("table_occurrences")?,
        relationship_count: count("relationships")?,
        value_list_count: count("value_lists")?,
        custom_function_count: count("custom_functions")?,
        account_count: count("accounts")?,
        privilege_set_count: count("privilege_sets")?,
    })
}

// ---------------------------------------------------------------------------
// Tauri コマンド
// ---------------------------------------------------------------------------

/// インポート済みソリューション一覧を返す（新しい順）。
#[tauri::command]
pub async fn list_solutions(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<SolutionRow>, CommandError> {
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    db_list_solutions(&db).map_err(CommandError::from)
}

/// ソリューションに属するプロジェクト一覧を返す。
#[tauri::command]
pub async fn get_solution_projects(
    state: tauri::State<'_, AppState>,
    solution_id: i64,
) -> Result<Vec<ProjectRow>, CommandError> {
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    crate::db::repository::get_solution_projects(&db, solution_id).map_err(CommandError::from)
}

/// ソリューションを削除する（関連 project も CASCADE 削除）。
#[tauri::command]
pub async fn delete_solution(
    state: tauri::State<'_, AppState>,
    solution_id: i64,
) -> Result<(), CommandError> {
    let mut db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    db_delete_solution(&mut db, solution_id).map_err(CommandError::from)
}

/// インポート済みプロジェクト一覧を返す（新しい順）。
#[tauri::command]
pub async fn list_projects(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ProjectRow>, CommandError> {
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    db_list_projects(&db).map_err(CommandError::from)
}

/// プロジェクトを削除する（関連データも CASCADE 削除）。
#[tauri::command]
pub async fn delete_project(
    state: tauri::State<'_, AppState>,
    project_id: i64,
) -> Result<(), CommandError> {
    let mut db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    db_delete_project(&mut db, project_id).map_err(CommandError::from)
}

/// プロジェクトの統計サマリーを返す。
#[tauri::command]
pub async fn get_project_summary(
    state: tauri::State<'_, AppState>,
    project_id: i64,
) -> Result<ProjectSummary, CommandError> {
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    build_project_summary(&db, project_id)
}

/// プロジェクト内の壊れた参照を返す。
#[tauri::command]
pub async fn get_broken_refs(
    state: tauri::State<'_, AppState>,
    project_id: i64,
) -> Result<Vec<BrokenRef>, CommandError> {
    let ddr = get_ddr(&state, project_id)?;
    Ok(find_broken_refs(&ddr))
}

/// プロジェクトの健全性レポートを返す。
#[tauri::command]
pub async fn get_report_card(
    state: tauri::State<'_, AppState>,
    project_id: i64,
) -> Result<ReportCard, CommandError> {
    let ddr = get_ddr(&state, project_id)?;
    Ok(generate_report_card(&ddr))
}

/// 要素名から DB の id を解決する（差分クリックナビゲーション用）。
#[tauri::command]
pub async fn resolve_element_by_name(
    state: tauri::State<'_, crate::AppState>,
    project_id: i64,
    element_type: String,
    name: String,
) -> Result<Option<ElementRef>, CommandError> {
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let table = match element_type.as_str() {
        "script" => "scripts",
        "layout" => "layouts",
        "table" => "base_tables",
        "value_list" => "value_lists",
        "custom_function" => "custom_functions",
        _ => return Ok(None),
    };
    let sql = format!("SELECT id, name FROM {table} WHERE project_id=?1 AND name=?2");
    let result = db
        .conn
        .query_row(&sql, rusqlite::params![project_id, name], |r| {
            Ok(ElementRef {
                id: r.get(0)?,
                name: r.get(1)?,
            })
        })
        .optional()
        .map_err(CommandError::from)?;
    Ok(result)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// アップグレードチェック: ソリューション配下の全プロジェクトを対象にヒット一覧を返す。
#[tauri::command]
pub async fn get_upgrade_check(
    state: tauri::State<'_, AppState>,
    solution_id: i64,
    check_items: Vec<CheckItemConfig>,
) -> Result<Vec<UpgradeHit>, CommandError> {
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    run_upgrade_check(&db, solution_id, &check_items).map_err(CommandError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::repository::{
        delete_project as db_delete_project, delete_solution as db_delete_solution,
        insert_ddr_file, insert_solution, list_solutions,
    };
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
    fn summary_counts_match_minimal_fixture() {
        let (db, pid) = setup();
        let s = build_project_summary(&db, pid).unwrap();
        assert_eq!(s.table_count, 1);
        assert_eq!(s.field_count, 1);
        assert_eq!(s.script_count, 1);
        assert_eq!(s.layout_count, 1);
        assert_eq!(s.relationship_count, 1);
        assert_eq!(s.value_list_count, 1);
        assert_eq!(s.custom_function_count, 1);
        assert_eq!(s.account_count, 1);
        assert_eq!(s.privilege_set_count, 1);
    }

    #[test]
    fn summary_project_name_matches() {
        let (db, pid) = setup();
        let s = build_project_summary(&db, pid).unwrap();
        assert_eq!(s.project.name, "TestDB");
    }

    #[test]
    fn summary_invalid_project_returns_error() {
        let db = Database::open_in_memory().unwrap();
        assert!(build_project_summary(&db, 9999).is_err());
    }

    #[test]
    fn list_empty_db_returns_empty() {
        let db = Database::open_in_memory().unwrap();
        let projects = db_list_projects(&db).unwrap();
        assert!(projects.is_empty());
    }

    #[test]
    fn list_returns_inserted_projects() {
        let (db, _) = setup();
        let projects = db_list_projects(&db).unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "TestDB");
    }

    #[test]
    fn delete_removes_project() {
        let (mut db, pid) = setup();
        db_delete_project(&mut db, pid).unwrap();
        assert!(db_list_projects(&db).unwrap().is_empty());
    }

    #[test]
    fn delete_nonexistent_returns_error() {
        let mut db = Database::open_in_memory().unwrap();
        assert!(db_delete_project(&mut db, 9999).is_err());
    }

    // ---- solution コマンドのテスト ----

    #[test]
    fn list_empty_db_returns_empty_solutions() {
        let db = Database::open_in_memory().unwrap();
        let solutions = list_solutions(&db).unwrap();
        assert!(solutions.is_empty());
    }

    #[test]
    fn list_returns_inserted_solutions() {
        let mut db = Database::open_in_memory().unwrap();
        insert_solution(&mut db, "Sol1", None).unwrap();
        insert_solution(&mut db, "Sol2", None).unwrap();
        let solutions = list_solutions(&db).unwrap();
        assert_eq!(solutions.len(), 2);
    }

    #[test]
    fn delete_solution_removes_solution() {
        let mut db = Database::open_in_memory().unwrap();
        let sid = insert_solution(&mut db, "Sol", None).unwrap();
        db_delete_solution(&mut db, sid).unwrap();
        let solutions = list_solutions(&db).unwrap();
        assert_eq!(solutions.len(), 0);
    }

    #[test]
    fn delete_nonexistent_solution_returns_error() {
        let mut db = Database::open_in_memory().unwrap();
        assert!(db_delete_solution(&mut db, 9999).is_err());
    }
}
