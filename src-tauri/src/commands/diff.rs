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
    list_all_projects_inner(&db.conn).map_err(CommandError::from)
}

fn list_all_projects_inner(
    conn: &rusqlite::Connection,
) -> Result<Vec<ProjectWithSolution>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.name, s.id, s.name, s.imported_at
           FROM projects p
           JOIN solutions s ON s.id = p.solution_id
          ORDER BY s.imported_at DESC, p.id ASC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ProjectWithSolution {
                project_id: row.get(0)?,
                project_name: row.get(1)?,
                solution_id: row.get(2)?,
                solution_name: row.get(3)?,
                solution_imported_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
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

    let items = merge_solution_projects(&projects_a, &projects_b, |id_a, id_b| {
        let ddr_a = get_ddr(&state, id_a)?;
        let ddr_b = get_ddr(&state, id_b)?;
        Ok(diff_ddr(&ddr_a, &ddr_b))
    })?;

    Ok(DiffResult::new(items))
}

/// A と B のプロジェクトリストを名前でマッチングし、差分アイテムを統合する純粋関数。
///
/// `diff_fn` は (proj_a_id, proj_b_id) を受け取り `DiffResult` を返すクロージャ。
/// テスト時はモック `diff_fn` を渡すことで `get_ddr` 依存を排除できる。
fn merge_solution_projects<F>(
    projects_a: &[crate::db::repository::ProjectRow],
    projects_b: &[crate::db::repository::ProjectRow],
    mut diff_fn: F,
) -> Result<Vec<DiffItem>, CommandError>
where
    F: FnMut(i64, i64) -> Result<DiffResult, CommandError>,
{
    let mut all_items: Vec<DiffItem> = Vec::new();

    // A の各プロジェクトを B の同名プロジェクトと比較
    for proj_a in projects_a {
        match projects_b.iter().find(|p| p.name == proj_a.name) {
            Some(proj_b) => {
                let result = diff_fn(proj_a.id, proj_b.id)?;
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
                // A にあって B にないプロジェクト（削除）
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

    // B にあって A にないプロジェクト（追加）
    for proj_b in projects_b {
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

    Ok(all_items)
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

    fn make_project(id: i64, name: &str) -> crate::db::repository::ProjectRow {
        crate::db::repository::ProjectRow {
            id,
            name: name.to_string(),
            file_path: None,
            fm_version: "19".to_string(),
            imported_at: "2024-01-01".to_string(),
        }
    }

    fn make_diff_item(kind: DiffKind, name: &str) -> DiffItem {
        DiffItem {
            kind,
            element_type: "script".into(),
            name: name.to_string(),
            detail: None,
            project_id: None,
            compare_project_id: None,
        }
    }

    // -----------------------------------------------------------------------
    // list_all_projects_inner のテスト
    // -----------------------------------------------------------------------

    #[test]
    fn list_all_projects_inner_returns_projects_with_solution_info() {
        use crate::db::{
            repository::{insert_ddr_file, insert_solution},
            Database,
        };
        let mut db = Database::open_in_memory().unwrap();
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let sol_a = insert_solution(&mut db, "SolA", Some("/a/s.xml")).unwrap();
        insert_ddr_file(&mut db, &ddr, sol_a, None).unwrap();
        let rows = list_all_projects_inner(&db.conn).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].solution_name, "SolA");
        assert!(!rows[0].solution_imported_at.is_empty());
    }

    #[test]
    fn list_all_projects_inner_returns_empty_for_empty_db() {
        use crate::db::Database;
        let db = Database::open_in_memory().unwrap();
        let rows = list_all_projects_inner(&db.conn).unwrap();
        assert!(rows.is_empty());
    }

    // -----------------------------------------------------------------------
    // merge_solution_projects のテスト
    // -----------------------------------------------------------------------

    #[test]
    fn merge_added_item_gets_b_as_project_and_a_as_compare() {
        let a = [make_project(10, "DB")];
        let b = [make_project(20, "DB")];
        let items = merge_solution_projects(&a, &b, |_id_a, _id_b| {
            Ok(DiffResult::new(vec![make_diff_item(
                DiffKind::Added,
                "NewScript",
            )]))
        })
        .unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].project_id, Some(20));
        assert_eq!(items[0].compare_project_id, Some(10));
    }

    #[test]
    fn merge_removed_item_gets_a_as_project_and_b_as_compare() {
        let a = [make_project(10, "DB")];
        let b = [make_project(20, "DB")];
        let items = merge_solution_projects(&a, &b, |_id_a, _id_b| {
            Ok(DiffResult::new(vec![make_diff_item(
                DiffKind::Removed,
                "OldScript",
            )]))
        })
        .unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].project_id, Some(10));
        assert_eq!(items[0].compare_project_id, Some(20));
    }

    #[test]
    fn merge_modified_item_gets_b_as_project_and_a_as_compare() {
        let a = [make_project(10, "DB")];
        let b = [make_project(20, "DB")];
        let items = merge_solution_projects(&a, &b, |_id_a, _id_b| {
            Ok(DiffResult::new(vec![make_diff_item(
                DiffKind::Modified,
                "ChangedScript",
            )]))
        })
        .unwrap();
        assert_eq!(items[0].project_id, Some(20));
        assert_eq!(items[0].compare_project_id, Some(10));
    }

    #[test]
    fn merge_project_only_in_a_becomes_removed_project_item() {
        let a = [make_project(10, "OnlyInA")];
        let b: [crate::db::repository::ProjectRow; 0] = [];
        let items = merge_solution_projects(&a, &b, |_, _| unreachable!()).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, DiffKind::Removed);
        assert_eq!(items[0].element_type, "project");
        assert_eq!(items[0].name, "OnlyInA");
        assert!(items[0].project_id.is_none());
    }

    #[test]
    fn merge_project_only_in_b_becomes_added_project_item() {
        let a: [crate::db::repository::ProjectRow; 0] = [];
        let b = [make_project(20, "OnlyInB")];
        let items = merge_solution_projects(&a, &b, |_, _| unreachable!()).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, DiffKind::Added);
        assert_eq!(items[0].element_type, "project");
        assert_eq!(items[0].name, "OnlyInB");
        assert!(items[0].project_id.is_none());
    }

    #[test]
    fn merge_empty_solutions_returns_empty() {
        let a: [crate::db::repository::ProjectRow; 0] = [];
        let b: [crate::db::repository::ProjectRow; 0] = [];
        let items = merge_solution_projects(&a, &b, |_, _| unreachable!()).unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn merge_mixed_scenario() {
        // A: [Match, OnlyA] / B: [Match, OnlyB]
        let a = [make_project(1, "Match"), make_project(2, "OnlyA")];
        let b = [make_project(3, "Match"), make_project(4, "OnlyB")];
        let items =
            merge_solution_projects(&a, &b, |_id_a, _id_b| Ok(DiffResult::new(vec![]))).unwrap();
        // Match は diff が空 → 0 件、OnlyA → Removed、OnlyB → Added
        assert_eq!(items.len(), 2);
        assert!(items
            .iter()
            .any(|i| i.kind == DiffKind::Removed && i.name == "OnlyA"));
        assert!(items
            .iter()
            .any(|i| i.kind == DiffKind::Added && i.name == "OnlyB"));
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
