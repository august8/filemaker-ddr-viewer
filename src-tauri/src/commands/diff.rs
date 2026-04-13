//! DDR 差分比較コマンド。

use crate::{
    analyzer::diff_engine::{diff_ddr, DiffItem, DiffKind, DiffResult},
    commands::CommandError,
    db::repository::get_solution_projects,
    AppState,
};
use serde::{Deserialize, Serialize};

use super::callchain::get_ddr;

/// プロジェクト選択ドロップダウン用の型。
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectWithSolution {
    pub project_id: i64,
    pub project_name: String,
    pub solution_id: i64,
    pub solution_name: String,
    pub solution_imported_at: String,
}

/// 全プロジェクトを solution 情報付きで返す。
#[tauri::command]
pub async fn list_all_projects(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ProjectWithSolution>, CommandError> {
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let mut stmt = db
        .conn
        .prepare(
            "SELECT p.id, p.name, s.id, s.name, s.imported_at
               FROM projects p
               JOIN solutions s ON s.id = p.solution_id
              ORDER BY s.imported_at DESC, p.id ASC",
        )
        .map_err(CommandError::from)?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ProjectWithSolution {
                project_id: row.get(0)?,
                project_name: row.get(1)?,
                solution_id: row.get(2)?,
                solution_name: row.get(3)?,
                solution_imported_at: row.get(4)?,
            })
        })
        .map_err(CommandError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(CommandError::from)?;
    Ok(rows)
}

/// 2 プロジェクト間の差分サマリーを返す。
#[tauri::command]
pub async fn compare_projects(
    state: tauri::State<'_, AppState>,
    project_id_a: i64,
    project_id_b: i64,
) -> Result<DiffResult, CommandError> {
    let ddr_a = get_ddr(&state, project_id_a)?;
    let ddr_b = get_ddr(&state, project_id_b)?;
    Ok(diff_ddr(&ddr_a, &ddr_b))
}

/// 2 ソリューション間の差分サマリーを返す。
///
/// 各ソリューション内のプロジェクト（DDR ファイル）を名前でマッチングし、
/// 同名プロジェクト同士を比較して差分を統合する。
/// どちらか一方にしか存在しないプロジェクトは Added/Removed として報告する。
#[tauri::command]
pub async fn compare_solutions(
    state: tauri::State<'_, AppState>,
    solution_id_a: i64,
    solution_id_b: i64,
) -> Result<DiffResult, CommandError> {
    let projects_a = {
        let db = state
            .db
            .lock()
            .map_err(|e| CommandError::Internal(e.to_string()))?;
        get_solution_projects(&db, solution_id_a).map_err(CommandError::from)?
    };
    let projects_b = {
        let db = state
            .db
            .lock()
            .map_err(|e| CommandError::Internal(e.to_string()))?;
        get_solution_projects(&db, solution_id_b).map_err(CommandError::from)?
    };

    let mut all_items: Vec<DiffItem> = Vec::new();

    // ソリューション A の各プロジェクトを B の同名プロジェクトと比較
    for proj_a in &projects_a {
        match projects_b.iter().find(|p| p.name == proj_a.name) {
            Some(proj_b) => {
                let ddr_a = get_ddr(&state, proj_a.id)?;
                let ddr_b = get_ddr(&state, proj_b.id)?;
                let result = diff_ddr(&ddr_a, &ddr_b);
                // Added/Modified → Target に遷移・Primary を比較元
                // Removed → Primary に遷移・Target を比較元
                for mut item in result.items {
                    match item.kind {
                        DiffKind::Added | DiffKind::Modified => {
                            item.project_id = Some(proj_b.id);
                            item.compare_project_id = Some(proj_a.id);
                        }
                        DiffKind::Removed => {
                            item.project_id = Some(proj_a.id);
                            item.compare_project_id = Some(proj_b.id);
                        }
                    }
                    all_items.push(item);
                }
            }
            None => {
                // ソリューション A にあって B にないプロジェクト（削除）
                all_items.push(DiffItem {
                    kind: DiffKind::Removed,
                    element_type: "project".into(),
                    name: proj_a.name.clone(),
                    detail: None,
                    project_id: None,
                    compare_project_id: None,
                });
            }
        }
    }

    // ソリューション B にあって A にないプロジェクト（追加）
    for proj_b in &projects_b {
        if !projects_a.iter().any(|p| p.name == proj_b.name) {
            all_items.push(DiffItem {
                kind: DiffKind::Added,
                element_type: "project".into(),
                name: proj_b.name.clone(),
                detail: None,
                project_id: None,
                compare_project_id: None,
            });
        }
    }

    Ok(DiffResult::new(all_items))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_ddr;

    const MINIMAL_XML: &str = include_str!("../../../tests/fixtures/minimal.xml");

    #[test]
    fn diff_same_project_is_empty() {
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let result = diff_ddr(&ddr, &ddr);
        assert_eq!(result.added_count, 0);
        assert_eq!(result.removed_count, 0);
        assert_eq!(result.modified_count, 0);
    }

    #[test]
    fn diff_result_counts_match() {
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let result = diff_ddr(&ddr, &ddr);
        assert_eq!(
            result.added_count + result.removed_count + result.modified_count,
            result.items.len()
        );
    }

    #[test]
    fn list_all_projects_returns_all() {
        use crate::db::{
            repository::{insert_ddr_file, insert_solution},
            Database,
        };
        let mut db = Database::open_in_memory().unwrap();
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        // ソリューション2件・各1プロジェクトを挿入
        let sol_a = insert_solution(&mut db, "SolA", Some("/a/summary.xml")).unwrap();
        let sol_b = insert_solution(&mut db, "SolB", Some("/b/summary.xml")).unwrap();
        insert_ddr_file(&mut db, &ddr, sol_a, Some("/a/a.xml")).unwrap();
        insert_ddr_file(&mut db, &ddr, sol_b, Some("/b/b.xml")).unwrap();

        let mut stmt = db
            .conn
            .prepare(
                "SELECT p.id, p.name, s.id, s.name, s.imported_at
                   FROM projects p
                   JOIN solutions s ON s.id = p.solution_id
                  ORDER BY s.imported_at DESC, p.id ASC",
            )
            .unwrap();
        let rows: Vec<ProjectWithSolution> = stmt
            .query_map([], |row| {
                Ok(ProjectWithSolution {
                    project_id: row.get(0)?,
                    project_name: row.get(1)?,
                    solution_id: row.get(2)?,
                    solution_name: row.get(3)?,
                    solution_imported_at: row.get(4)?,
                })
            })
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(rows.len(), 2);
        // solution_name が含まれていること
        assert!(rows.iter().any(|r| r.solution_name == "SolA"));
        assert!(rows.iter().any(|r| r.solution_name == "SolB"));
        // solution_imported_at が空でないこと
        assert!(rows.iter().all(|r| !r.solution_imported_at.is_empty()));
    }

    #[test]
    fn compare_solutions_projects_are_matched_by_name() {
        use crate::db::{
            repository::{insert_ddr_file, insert_solution},
            Database,
        };
        let mut db = Database::open_in_memory().unwrap();
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        // 同名プロジェクトを持つ2ソリューション
        let sol_a = insert_solution(&mut db, "SolA", Some("/a/summary.xml")).unwrap();
        let sol_b = insert_solution(&mut db, "SolB", Some("/b/summary.xml")).unwrap();
        insert_ddr_file(&mut db, &ddr, sol_a, Some("/a/test.xml")).unwrap();
        insert_ddr_file(&mut db, &ddr, sol_b, Some("/b/test.xml")).unwrap();

        let proj_a = get_solution_projects(&db, sol_a).unwrap();
        let proj_b = get_solution_projects(&db, sol_b).unwrap();

        // 同名プロジェクトが存在してマッチングできること
        assert_eq!(proj_a.len(), 1);
        assert_eq!(proj_b.len(), 1);
        assert_eq!(proj_a[0].name, proj_b[0].name);
    }

    #[test]
    fn compare_solutions_unmatched_project_detected() {
        use crate::db::{
            repository::{insert_ddr_file, insert_solution},
            Database,
        };
        let mut db = Database::open_in_memory().unwrap();
        let mut ddr_extra = parse_ddr(MINIMAL_XML).unwrap();
        ddr_extra.file_name = "ExtraFile".into();
        let ddr = parse_ddr(MINIMAL_XML).unwrap();

        let sol_a = insert_solution(&mut db, "SolA", Some("/a/summary.xml")).unwrap();
        let sol_b = insert_solution(&mut db, "SolB", Some("/b/summary.xml")).unwrap();
        // SolA には1プロジェクト、SolB には2プロジェクト（追加ファイルあり）
        insert_ddr_file(&mut db, &ddr, sol_a, Some("/a/base.xml")).unwrap();
        insert_ddr_file(&mut db, &ddr, sol_b, Some("/b/base.xml")).unwrap();
        insert_ddr_file(&mut db, &ddr_extra, sol_b, Some("/b/extra.xml")).unwrap();

        let proj_a = get_solution_projects(&db, sol_a).unwrap();
        let proj_b = get_solution_projects(&db, sol_b).unwrap();

        // SolB にあって SolA にないプロジェクトを検出できること
        let unmatched: Vec<_> = proj_b
            .iter()
            .filter(|p| !proj_a.iter().any(|pa| pa.name == p.name))
            .collect();
        assert_eq!(unmatched.len(), 1);
        assert_eq!(unmatched[0].name, "ExtraFile");
    }
}
