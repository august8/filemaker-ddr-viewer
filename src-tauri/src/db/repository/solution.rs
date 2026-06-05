//! ソリューション・プロジェクト CRUD 操作。

use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::db::{Database, DbError};

// ---------------------------------------------------------------------------
// 公開データ型
// ---------------------------------------------------------------------------

/// DB から取得したソリューション行。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionRow {
    pub id: i64,
    pub name: String,
    pub summary_path: Option<String>,
    pub imported_at: String,
}

/// ソリューションとそれに属するプロジェクト一覧。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionWithProjects {
    pub solution: SolutionRow,
    pub projects: Vec<ProjectRow>,
}

/// DB から取得したプロジェクト行。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRow {
    pub id: i64,
    pub name: String,
    pub file_path: Option<String>,
    pub fm_version: String,
    pub imported_at: String,
}

// ---------------------------------------------------------------------------
// ソリューション管理
// ---------------------------------------------------------------------------

/// 新しい solution を作成し、その ID を返す。
pub fn insert_solution(
    db: &mut Database,
    name: &str,
    summary_path: Option<&str>,
) -> Result<i64, DbError> {
    db.conn.execute(
        "INSERT INTO solutions (name, summary_path, imported_at) VALUES (?1, ?2, datetime('now', 'localtime'))",
        params![name, summary_path],
    )?;
    Ok(db.conn.last_insert_rowid())
}

/// ID で solution を1件取得する。
pub fn get_solution(db: &Database, solution_id: i64) -> Result<SolutionRow, DbError> {
    db.conn
        .query_row(
            "SELECT id, name, summary_path, imported_at
               FROM solutions WHERE id = ?1",
            params![solution_id],
            |row| {
                Ok(SolutionRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    summary_path: row.get(2)?,
                    imported_at: row.get(3)?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                DbError::NotFound(format!("solution:{solution_id}"))
            }
            other => DbError::Sqlite(other),
        })
}

/// solution 一覧を返す（新しい順）。
pub fn list_solutions(db: &Database) -> Result<Vec<SolutionRow>, DbError> {
    let mut stmt = db.conn.prepare(
        "SELECT id, name, summary_path, imported_at
           FROM solutions
          ORDER BY id DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(SolutionRow {
            id: row.get(0)?,
            name: row.get(1)?,
            summary_path: row.get(2)?,
            imported_at: row.get(3)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// solution を CASCADE 削除する（紐付く projects も全削除）。
pub fn delete_solution(db: &mut Database, solution_id: i64) -> Result<(), DbError> {
    // FTS5 仮想テーブルは CASCADE DELETE に対応しないため手動で削除する
    db.conn.execute(
        "DELETE FROM search_index WHERE project_id IN (SELECT id FROM projects WHERE solution_id = ?1)",
        params![solution_id],
    )?;
    let affected = db
        .conn
        .execute("DELETE FROM solutions WHERE id = ?1", params![solution_id])?;
    if affected == 0 {
        return Err(DbError::NotFound(format!("solution:{solution_id}")));
    }
    Ok(())
}

/// solution に属する project 一覧を返す（新しい順）。
pub fn get_solution_projects(db: &Database, solution_id: i64) -> Result<Vec<ProjectRow>, DbError> {
    let mut stmt = db.conn.prepare(
        "SELECT id, name, file_path, fm_version, imported_at
           FROM projects
          WHERE solution_id = ?1
          ORDER BY id ASC",
    )?;
    let rows = stmt.query_map(params![solution_id], |row| {
        Ok(ProjectRow {
            id: row.get(0)?,
            name: row.get(1)?,
            file_path: row.get(2)?,
            fm_version: row.get(3)?,
            imported_at: row.get(4)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

// ---------------------------------------------------------------------------
// プロジェクト管理
// ---------------------------------------------------------------------------

/// プロジェクト一覧を返す（新しい順）。
pub fn list_projects(db: &Database) -> Result<Vec<ProjectRow>, DbError> {
    let mut stmt = db.conn.prepare(
        "SELECT id, name, file_path, fm_version, imported_at
           FROM projects
          ORDER BY id DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ProjectRow {
            id: row.get(0)?,
            name: row.get(1)?,
            file_path: row.get(2)?,
            fm_version: row.get(3)?,
            imported_at: row.get(4)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

/// ID でプロジェクトを1件取得する。
pub fn get_project(db: &Database, project_id: i64) -> Result<ProjectRow, DbError> {
    db.conn
        .query_row(
            "SELECT id, name, file_path, fm_version, imported_at
               FROM projects WHERE id = ?1",
            params![project_id],
            |row| {
                Ok(ProjectRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    file_path: row.get(2)?,
                    fm_version: row.get(3)?,
                    imported_at: row.get(4)?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                DbError::NotFound(format!("project:{project_id}"))
            }
            other => DbError::Sqlite(other),
        })
}

/// プロジェクト名を更新する。
pub fn update_project_name(
    db: &mut Database,
    project_id: i64,
    new_name: &str,
) -> Result<(), DbError> {
    let affected = db.conn.execute(
        "UPDATE projects SET name = ?1 WHERE id = ?2",
        params![new_name, project_id],
    )?;
    if affected == 0 {
        return Err(DbError::NotFound(format!("project:{project_id}")));
    }
    Ok(())
}

/// プロジェクトを CASCADE 削除する（関連データも全削除）。
pub fn delete_project(db: &mut Database, project_id: i64) -> Result<(), DbError> {
    // FTS5 仮想テーブルは CASCADE DELETE に対応しないため手動で削除する
    db.conn.execute(
        "DELETE FROM search_index WHERE project_id = ?1",
        params![project_id],
    )?;
    let affected = db
        .conn
        .execute("DELETE FROM projects WHERE id = ?1", params![project_id])?;
    if affected == 0 {
        return Err(DbError::NotFound(format!("project:{project_id}")));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::repository::insert_ddr_file;
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

    // ---- solution CRUD ----

    #[test]
    fn insert_solution_creates_row() {
        let mut db = Database::open_in_memory().unwrap();
        let sid = insert_solution(&mut db, "MySolution", Some("/path/to/概要.xml")).unwrap();
        let row = get_solution(&db, sid).unwrap();
        assert_eq!(row.name, "MySolution");
        assert_eq!(row.summary_path.as_deref(), Some("/path/to/概要.xml"));
    }

    #[test]
    fn list_solutions_returns_all() {
        let mut db = Database::open_in_memory().unwrap();
        insert_solution(&mut db, "Sol1", None).unwrap();
        insert_solution(&mut db, "Sol2", None).unwrap();
        let solutions = list_solutions(&db).unwrap();
        assert_eq!(solutions.len(), 2);
    }

    #[test]
    fn get_solution_not_found() {
        let db = Database::open_in_memory().unwrap();
        assert!(get_solution(&db, 9999).is_err());
    }

    #[test]
    fn delete_solution_cascades_to_projects() {
        let (mut db, sid, pid) = db_with_minimal();
        delete_solution(&mut db, sid).unwrap();

        assert!(get_solution(&db, sid).is_err());
        assert!(get_project(&db, pid).is_err());
    }

    #[test]
    fn delete_solution_not_found_returns_err() {
        let mut db = Database::open_in_memory().unwrap();
        assert!(delete_solution(&mut db, 9999).is_err());
    }

    #[test]
    fn get_solution_projects_returns_correct_projects() {
        let mut db = Database::open_in_memory().unwrap();
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let sid = insert_solution(&mut db, "Sol", None).unwrap();
        insert_ddr_file(&mut db, &ddr, sid, None).unwrap();
        insert_ddr_file(&mut db, &ddr, sid, None).unwrap();
        let projects = get_solution_projects(&db, sid).unwrap();
        assert_eq!(projects.len(), 2);
    }

    // ---- project CRUD ----

    #[test]
    fn list_projects_returns_all() {
        let mut db = Database::open_in_memory().unwrap();
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let sid = insert_solution(&mut db, "S", None).unwrap();
        insert_ddr_file(&mut db, &ddr, sid, None).unwrap();
        insert_ddr_file(&mut db, &ddr, sid, None).unwrap();
        let projects = list_projects(&db).unwrap();
        assert_eq!(projects.len(), 2);
    }

    #[test]
    fn get_project_not_found() {
        let db = Database::open_in_memory().unwrap();
        assert!(get_project(&db, 9999).is_err());
    }

    #[test]
    fn delete_project_removes_cascaded_data() {
        let (mut db, _sid, pid) = db_with_minimal();
        delete_project(&mut db, pid).unwrap();

        let n: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM fields WHERE project_id=?1",
                params![pid],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 0, "cascade delete should remove fields");
    }

    #[test]
    fn delete_project_not_found_returns_err() {
        let mut db = Database::open_in_memory().unwrap();
        assert!(delete_project(&mut db, 9999).is_err());
    }

    // ---- update_project_name ----

    #[test]
    fn update_project_name_changes_name() {
        let (mut db, _sid, pid) = db_with_minimal();
        update_project_name(&mut db, pid, "新しい名前").unwrap();
        let row = get_project(&db, pid).unwrap();
        assert_eq!(row.name, "新しい名前");
    }

    #[test]
    fn update_project_name_not_found_returns_err() {
        let mut db = Database::open_in_memory().unwrap();
        assert!(update_project_name(&mut db, 9999, "X").is_err());
    }
}
