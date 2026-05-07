use rusqlite::params;

use crate::{commands::CommandError, AppState};

use super::helpers::{fetch_occ_names, field_ref_matches};
use super::{FieldCalcRef, FieldRefScript};

/// フィールドを参照しているスクリプトの一覧を返す。
///
/// オカレンス名経由の検索 + 識別子境界チェックにより誤検知・漏れを防ぐ。
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
    let db = crate::commands::lock_db(&state)?;
    get_field_refs_inner(&db.conn, project_id, &table_name, &field_name).map_err(CommandError::from)
}

/// このフィールドを計算式（calculation）で参照している他フィールドの一覧を返す。
///
/// オカレンス名経由の検索 + 識別子境界チェックにより誤検知・漏れを防ぐ。
/// 自分自身は除外する。
#[tauri::command]
pub async fn get_field_calc_refs(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    table_name: String,
    field_name: String,
) -> Result<Vec<FieldCalcRef>, CommandError> {
    if table_name.is_empty() || field_name.is_empty() {
        return Ok(vec![]);
    }
    let db = crate::commands::lock_db(&state)?;
    get_field_calc_refs_inner(&db.conn, project_id, &table_name, &field_name)
        .map_err(CommandError::from)
}

// ---------------------------------------------------------------------------
// inner 関数（テスト可能・Tauri State 非依存）
// ---------------------------------------------------------------------------

/// `get_field_refs` の内部実装（ソリューション全体スコープ）。
fn get_field_refs_inner(
    conn: &rusqlite::Connection,
    project_id: i64,
    table_name: &str,
    field_name: &str,
) -> Result<Vec<FieldRefScript>, rusqlite::Error> {
    // 1. ベーステーブルに紐づく全オカレンス名を取得（ソリューションスコープ）
    let occ_names = fetch_occ_names(conn, project_id, table_name)?;

    // 2. 全スクリプトステップを取得（ソリューション全体スコープ）
    let mut stmt = conn.prepare(
        "SELECT s.id, s.name, s.project_id,
                COALESCE(ss.step_text, ''),
                COALESCE(ss.calculation, '')
         FROM scripts s
         JOIN script_steps ss ON ss.script_id = s.id
         WHERE s.project_id IN (
           SELECT id FROM projects
           WHERE solution_id = (SELECT solution_id FROM projects WHERE id = ?1)
         )",
    )?;
    let step_rows: Vec<(i64, String, i64, String, String)> = stmt
        .query_map(params![project_id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // 3. Rust でフィルタリング（DISTINCT script_id）
    let mut seen = std::collections::HashSet::new();
    let mut results: Vec<FieldRefScript> = step_rows
        .into_iter()
        .filter(|(_, _, _, step_text, calc)| {
            field_ref_matches(step_text, &occ_names, field_name)
                || field_ref_matches(calc, &occ_names, field_name)
        })
        .filter_map(|(script_id, script_name, script_project_id, _, _)| {
            seen.insert(script_id).then_some(FieldRefScript {
                script_id,
                script_name,
                project_id: script_project_id,
            })
        })
        .collect();
    results.sort_by(|a, b| a.script_name.cmp(&b.script_name));
    Ok(results)
}

/// `get_field_calc_refs` の内部実装（ソリューション全体スコープ）。
fn get_field_calc_refs_inner(
    conn: &rusqlite::Connection,
    project_id: i64,
    table_name: &str,
    field_name: &str,
) -> Result<Vec<FieldCalcRef>, rusqlite::Error> {
    // 1. ベーステーブルに紐づく全オカレンス名を取得（ソリューションスコープ）
    let occ_names = fetch_occ_names(conn, project_id, table_name)?;

    // 2. 全フィールド（calculation 非空・自分自身除く）を取得（ソリューション全体スコープ）
    let mut stmt = conn.prepare(
        "SELECT f.id, f.name, bt.name, bt.id, bt.project_id, f.calculation
         FROM fields f
         JOIN base_tables bt ON bt.id = f.table_id
         WHERE bt.project_id IN (
           SELECT id FROM projects
           WHERE solution_id = (SELECT solution_id FROM projects WHERE id = ?1)
         )
           AND f.calculation IS NOT NULL
           AND f.calculation != ''
           AND NOT (bt.name = ?2 AND f.name = ?3)
         ORDER BY bt.name, f.name",
    )?;
    let candidates: Vec<(i64, String, String, i64, i64, String)> = stmt
        .query_map(params![project_id, table_name, field_name], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // 3. Rust でフィルタリング
    Ok(candidates
        .into_iter()
        .filter(|(_, _, _, _, _, calc)| field_ref_matches(calc, &occ_names, field_name))
        .map(
            |(field_id, field_name, table_name, table_id, field_project_id, _)| FieldCalcRef {
                field_id,
                field_name,
                table_name,
                table_id,
                project_id: field_project_id,
            },
        )
        .collect())
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::super::helpers::test_helpers::{setup, setup_cross_project};
    use super::*;

    #[test]
    fn test_finds_via_occurrence_name() {
        let (conn, project_id) = setup();
        let refs = get_field_calc_refs_inner(&conn, project_id, "Invoice", "Amount").unwrap();
        let pairs: Vec<(&str, &str)> = refs
            .iter()
            .map(|r| (r.table_name.as_str(), r.field_name.as_str()))
            .collect();
        // Invoice::Total (InvoiceAlias::Amount), Invoice::合計金額 (bare Amount), Order::Total (Invoice::Amount)
        assert!(pairs.contains(&("Invoice", "Total")));
        assert!(pairs.contains(&("Invoice", "合計金額")));
        assert!(pairs.contains(&("Order", "Total")));
    }

    #[test]
    fn test_excludes_self() {
        let (conn, project_id) = setup();
        let refs = get_field_calc_refs_inner(&conn, project_id, "Invoice", "Amount").unwrap();
        assert!(!refs
            .iter()
            .any(|r| r.table_name == "Invoice" && r.field_name == "Amount"));
    }

    #[test]
    fn test_returns_empty_when_no_refs() {
        let (conn, project_id) = setup();
        // Order::Note は Invoice::Amount を参照していない
        let refs = get_field_calc_refs_inner(&conn, project_id, "Invoice", "Note").unwrap();
        assert!(refs.is_empty());
    }

    #[test]
    fn test_different_occurrence_not_matched() {
        let (conn, project_id) = setup();
        // Order::Note は Order::Amount を参照しており Invoice::Amount ではない
        let refs = get_field_calc_refs_inner(&conn, project_id, "Invoice", "Amount").unwrap();
        assert!(!refs
            .iter()
            .any(|r| r.table_name == "Order" && r.field_name == "Note"));
    }

    /// 分離モデル: データファイルの project_id で検索したとき、
    /// プログラムファイル側のスクリプトが見つかること。
    #[test]
    fn get_field_refs_finds_scripts_in_other_project_of_same_solution() {
        let (conn, program_project_id, data_project_id) = setup_cross_project();

        // プログラムファイルにスクリプトを追加（"Customers::FirstName" を参照）
        conn.execute(
            "INSERT INTO scripts(project_id, fm_id, name, run_with_full_access)
             VALUES(?1, 1, 'UpdateCustomer', 0)",
            [program_project_id],
        )
        .unwrap();
        let script_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO script_steps(script_id, step_type_id, name, enabled, step_text)
             VALUES(?1, 76, 'Set Field', 1, 'Set Field [ Customers::FirstName ; \"Test\" ]')",
            [script_id],
        )
        .unwrap();

        // データファイルの project_id を起点にソリューション全体を検索
        let refs = get_field_refs_inner(&conn, data_project_id, "Customer", "FirstName").unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].script_name, "UpdateCustomer");
        assert_eq!(refs[0].project_id, program_project_id);
    }

    /// 分離モデル: calc refs がソリューション全体を検索すること。
    #[test]
    fn get_field_calc_refs_finds_calc_fields_in_other_project() {
        let (conn, program_project_id, data_project_id) = setup_cross_project();

        // プログラムファイル側にも計算フィールドを追加（Customer::FirstName を参照）
        conn.execute(
            "INSERT INTO base_tables(project_id, fm_id, name) VALUES(?1, 10, 'Form')",
            [program_project_id],
        )
        .unwrap();
        let form_table_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO fields(project_id, table_id, fm_id, name, field_type, data_type, calculation)
             VALUES(?1, ?2, 10, 'Display', 'Calculation', 'Text', 'Customers::FirstName')",
            rusqlite::params![program_project_id, form_table_id],
        )
        .unwrap();

        // データファイルの project_id で検索 → プログラムファイルの計算フィールドがヒットする
        let refs =
            get_field_calc_refs_inner(&conn, data_project_id, "Customer", "FirstName").unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].field_name, "Display");
        assert_eq!(refs[0].project_id, program_project_id);
    }
}
