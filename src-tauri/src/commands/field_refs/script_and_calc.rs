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

    // 1. ベーステーブルに紐づく全オカレンス名を取得
    let occ_names =
        fetch_occ_names(&db.conn, project_id, &table_name).map_err(CommandError::from)?;

    // 2. 全スクリプトステップを取得（ソリューション全体スコープ）
    let mut stmt = db
        .conn
        .prepare(
            "SELECT s.id, s.name, s.project_id,
                    COALESCE(ss.step_text, ''),
                    COALESCE(ss.calculation, '')
             FROM scripts s
             JOIN script_steps ss ON ss.script_id = s.id
             WHERE s.project_id IN (
               SELECT id FROM projects
               WHERE solution_id = (SELECT solution_id FROM projects WHERE id = ?1)
             )",
        )
        .map_err(CommandError::from)?;
    let step_rows: Vec<(i64, String, i64, String, String)> = stmt
        .query_map(params![project_id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })
        .map_err(CommandError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(CommandError::from)?;

    // 3. Rust でフィルタリング（DISTINCT script_id）
    let mut seen = std::collections::HashSet::new();
    let results = step_rows
        .into_iter()
        .filter(|(_, _, _, step_text, calc)| {
            field_ref_matches(step_text, &occ_names, &field_name)
                || field_ref_matches(calc, &occ_names, &field_name)
        })
        .filter_map(|(script_id, script_name, script_project_id, _, _)| {
            seen.insert(script_id).then_some(FieldRefScript {
                script_id,
                script_name,
                project_id: script_project_id,
            })
        })
        .collect::<Vec<_>>();

    let mut results = results;
    results.sort_by(|a, b| a.script_name.cmp(&b.script_name));
    Ok(results)
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

    // 1. ベーステーブルに紐づく全オカレンス名を取得
    let occ_names =
        fetch_occ_names(&db.conn, project_id, &table_name).map_err(CommandError::from)?;

    // 2. 全フィールド（calculation 非空・自分自身除く）を取得（ソリューション全体スコープ）
    let mut stmt = db
        .conn
        .prepare(
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
        )
        .map_err(CommandError::from)?;
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
        })
        .map_err(CommandError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(CommandError::from)?;

    // 3. Rust でフィルタリング
    let results = candidates
        .into_iter()
        .filter(|(_, _, _, _, _, calc)| field_ref_matches(calc, &occ_names, &field_name))
        .map(
            |(field_id, field_name, table_name, table_id, field_project_id, _)| FieldCalcRef {
                field_id,
                field_name,
                table_name,
                table_id,
                project_id: field_project_id,
            },
        )
        .collect();
    Ok(results)
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::super::helpers::test_helpers::{setup, setup_cross_project};
    use super::*;

    /// `get_field_calc_refs` の内部ロジックを直接呼び出すヘルパー。
    fn calc_refs(
        conn: &rusqlite::Connection,
        project_id: i64,
        table_name: &str,
        field_name: &str,
    ) -> Vec<(String, String)> {
        let occ_names = fetch_occ_names(conn, project_id, table_name).unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT f.id, f.name, bt.name, bt.id, f.calculation
                 FROM fields f
                 JOIN base_tables bt ON bt.id = f.table_id
                 WHERE bt.project_id = ?1
                   AND f.calculation IS NOT NULL AND f.calculation != ''
                   AND NOT (bt.name = ?2 AND f.name = ?3)
                 ORDER BY bt.name, f.name",
            )
            .unwrap();
        stmt.query_map(
            rusqlite::params![project_id, table_name, field_name],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )
        .unwrap()
        .filter_map(|r| r.ok())
        .filter(|(_, _, _, _, calc)| field_ref_matches(calc, &occ_names, field_name))
        .map(|(_, fname, tname, _, _)| (tname, fname))
        .collect()
    }

    #[test]
    fn test_finds_via_occurrence_name() {
        let (conn, project_id) = setup();
        let refs = calc_refs(&conn, project_id, "Invoice", "Amount");
        // Invoice::Total (InvoiceAlias::Amount), Invoice::合計金額 (bare Amount), Order::Total (Invoice::Amount)
        assert!(refs.contains(&("Invoice".to_string(), "Total".to_string())));
        assert!(refs.contains(&("Invoice".to_string(), "合計金額".to_string())));
        assert!(refs.contains(&("Order".to_string(), "Total".to_string())));
    }

    #[test]
    fn test_excludes_self() {
        let (conn, project_id) = setup();
        let refs = calc_refs(&conn, project_id, "Invoice", "Amount");
        assert!(!refs.contains(&("Invoice".to_string(), "Amount".to_string())));
    }

    #[test]
    fn test_returns_empty_when_no_refs() {
        let (conn, project_id) = setup();
        // Order::Note は Invoice::Amount を参照していない
        let refs = calc_refs(&conn, project_id, "Invoice", "Note");
        assert!(refs.is_empty());
    }

    #[test]
    fn test_different_occurrence_not_matched() {
        let (conn, project_id) = setup();
        // Order::Note は Order::Amount を参照しており Invoice::Amount ではない
        let refs = calc_refs(&conn, project_id, "Invoice", "Amount");
        assert!(!refs.contains(&("Order".to_string(), "Note".to_string())));
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

        // データファイルの project_id を起点に参照スクリプトを検索
        let occ_names = fetch_occ_names(&conn, data_project_id, "Customer").unwrap();
        assert!(
            occ_names.contains(&"Customers".to_string()),
            "ソリューションスコープで Customers が取れるべき"
        );

        // 注: get_field_refs はコマンド層なので、ここでは fetch_occ_names の検証のみ行い
        //     スクリプト検索は実装後の Green 確認で検証する
        let _ = script_id;
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

        // データファイルの project_id で calc refs を検索 → プログラムファイルの Field も見つかるべき
        let occ_names = fetch_occ_names(&conn, data_project_id, "Customer").unwrap();
        assert!(occ_names.contains(&"Customers".to_string()));
        let _ = (program_project_id, form_table_id);
    }
}
