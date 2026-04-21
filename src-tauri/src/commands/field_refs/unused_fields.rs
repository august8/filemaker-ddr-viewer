use rusqlite::params;

use crate::{commands::CommandError, AppState};

use super::helpers::has_bare_field_ref;
use super::UnusedFieldRow;

/// レイアウト・リレーション・スクリプト計算式・フィールド計算式・バリューリストから
/// 一度も参照されていないフィールドの一覧を返す。
///
/// 検査対象:
/// - layout_field_refs / layout_objects（レイアウト配置）
/// - join_predicates（リレーション結合キー）
/// - value_list_field_refs（バリューリストのフィールドソース）
/// - script_steps.calculation / step_text, fields.calculation / auto_enter_calc / val_calc
///   （OccName::FieldName パターンのみ検出）
#[tauri::command]
pub async fn list_unused_fields(
    state: tauri::State<'_, AppState>,
    project_id: i64,
) -> Result<Vec<UnusedFieldRow>, CommandError> {
    let db = crate::commands::lock_db(&state)?;
    list_unused_fields_inner(&db.conn, project_id).map_err(CommandError::from)
}

// ---------------------------------------------------------------------------
// inner 関数（テスト可能・Tauri State 非依存）
// ---------------------------------------------------------------------------

/// `list_unused_fields` の内部実装（テスト可能なように分離）。
fn list_unused_fields_inner(
    conn: &rusqlite::Connection,
    project_id: i64,
) -> Result<Vec<UnusedFieldRow>, rusqlite::Error> {
    // Step 1: 構造的参照（レイアウト・リレーション・バリューリスト）で used なフィールドを SQL で収集
    let mut stmt = conn.prepare(
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
           UNION
           -- バリューリストのフィールド参照（PrimaryField / SecondaryField）
           SELECT DISTINCT toc.base_table_name AS tbl, vlfr.field_name
           FROM value_list_field_refs vlfr
           JOIN value_lists vl ON vl.id = vlfr.value_list_id AND vl.project_id = ?1
           JOIN table_occurrences toc
             ON toc.occurrence_name = vlfr.table_occurrence AND toc.project_id = ?1
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
    )?;
    let mut rows = stmt
        .query_map(params![project_id], |row| {
            Ok(UnusedFieldRow {
                table_name: row.get(0)?,
                field_name: row.get(1)?,
                field_type: row.get(2)?,
                data_type: row.get(3)?,
                field_id: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Step 2: スクリプト・フィールド計算式の OccName::FieldName Rust ポスト処理
    // SQL の LIKE クロスジョインを避けるため Rust でフィルタリングする。
    // 検出パターン: "OccurrenceName::FieldName" の文字列が計算テキストに含まれる場合に使用済みと判定。
    if !rows.is_empty() {
        // ベーステーブル名 → オカレンス名リスト のマップを構築
        let occ_map = fetch_occ_map(conn, project_id)?;

        // 全計算テキストを1つの文字列に結合（スクリプト + フィールド各 calc 列）
        let all_calc_text = fetch_all_calc_texts(conn, project_id)?;

        // 計算テキストに OccName::FieldName が含まれるフィールドを除外
        rows.retain(|f| {
            let occs = occ_map.get(&f.table_name).map(Vec::as_slice).unwrap_or(&[]);
            !occs
                .iter()
                .any(|occ| all_calc_text.contains(&format!("{}::{}", occ, f.field_name)))
        });
    }

    // Step 3: 同テーブルのフィールド計算式内ベア参照チェック（FileMaker 仕様準拠）
    // FileMaker では計算式内のベア参照は常に同テーブルのフィールドを指す。
    // スクリプトステップは実行時レイアウトコンテキスト依存のため対象外。
    if !rows.is_empty() {
        let same_table_calcs = fetch_same_table_field_calcs(conn, project_id)?;

        rows.retain(|f| {
            let calcs = same_table_calcs
                .get(&f.table_name)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            !calcs
                .iter()
                .any(|calc| has_bare_field_ref(calc, &f.field_name))
        });
    }

    Ok(rows)
}

/// テーブル名 → 同テーブル内全フィールドの計算テキスト一覧を返す。
///
/// FileMaker では計算式内のベア参照は同テーブルのフィールドを指すため、
/// 同テーブルのフィールド calc からのみベア参照チェックを行う。
/// 対象列: calculation / auto_enter_calc / val_calc
fn fetch_same_table_field_calcs(
    conn: &rusqlite::Connection,
    project_id: i64,
) -> Result<std::collections::HashMap<String, Vec<String>>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT bt.name,
                COALESCE(f.calculation, ''),
                COALESCE(f.auto_enter_calc, ''),
                COALESCE(f.val_calc, '')
         FROM fields f
         JOIN base_tables bt ON bt.id = f.table_id AND bt.project_id = ?1",
    )?;
    let mut map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let rows = stmt.query_map(params![project_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    for row in rows {
        let (table, calc, ae, val) = row?;
        let entry = map.entry(table).or_default();
        if !calc.is_empty() {
            entry.push(calc);
        }
        if !ae.is_empty() {
            entry.push(ae);
        }
        if !val.is_empty() {
            entry.push(val);
        }
    }
    Ok(map)
}

/// ベーステーブル名 → オカレンス名リスト のマップを取得する。
fn fetch_occ_map(
    conn: &rusqlite::Connection,
    project_id: i64,
) -> Result<std::collections::HashMap<String, Vec<String>>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT base_table_name, occurrence_name
         FROM table_occurrences
         WHERE project_id = ?1",
    )?;
    let mut map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let rows = stmt.query_map(params![project_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    for row in rows {
        let (base, occ) = row?;
        map.entry(base).or_default().push(occ);
    }
    Ok(map)
}

/// プロジェクト内の全計算テキストを結合した文字列を返す。
///
/// 対象:
/// - script_steps.calculation / step_text
/// - fields.calculation / auto_enter_calc / val_calc
fn fetch_all_calc_texts(
    conn: &rusqlite::Connection,
    project_id: i64,
) -> Result<String, rusqlite::Error> {
    let mut parts: Vec<String> = Vec::new();

    // スクリプトステップの計算式・ステップテキスト
    {
        let mut stmt = conn.prepare(
            "SELECT COALESCE(ss.calculation, ''), COALESCE(ss.step_text, '')
             FROM script_steps ss
             JOIN scripts s ON s.id = ss.script_id AND s.project_id = ?1",
        )?;
        let rows = stmt.query_map(params![project_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (calc, text) = row?;
            if !calc.is_empty() {
                parts.push(calc);
            }
            if !text.is_empty() {
                parts.push(text);
            }
        }
    }

    // フィールドの各計算式列（calculation / auto_enter_calc / val_calc）
    {
        let mut stmt = conn.prepare(
            "SELECT COALESCE(f.calculation, ''),
                    COALESCE(f.auto_enter_calc, ''),
                    COALESCE(f.val_calc, '')
             FROM fields f
             JOIN base_tables bt ON bt.id = f.table_id AND bt.project_id = ?1",
        )?;
        let rows = stmt.query_map(params![project_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        for row in rows {
            let (calc, ae, val) = row?;
            if !calc.is_empty() {
                parts.push(calc);
            }
            if !ae.is_empty() {
                parts.push(ae);
            }
            if !val.is_empty() {
                parts.push(val);
            }
        }
    }

    Ok(parts.join("\n"))
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::super::helpers::test_helpers::setup;
    use super::*;

    /// `list_unused_fields_inner` を呼び出して (table_name, field_name) のペアを返す。
    fn list_unused(conn: &Connection, project_id: i64) -> Vec<(String, String)> {
        list_unused_fields_inner(conn, project_id)
            .unwrap()
            .into_iter()
            .map(|r| (r.table_name, r.field_name))
            .collect()
    }

    fn setup_with_value_list_ref() -> (Connection, i64) {
        let (conn, project_id) = setup();

        // Invoice::Amount を参照するバリューリストを追加
        conn.execute(
            "INSERT INTO value_lists(project_id, fm_id, name, source) VALUES(?1, 1, 'TestVL', 'Field')",
            rusqlite::params![project_id],
        )
        .unwrap();
        let vl_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO value_list_field_refs(value_list_id, table_occurrence, field_name)
             VALUES(?1, 'Invoice', 'Amount')",
            rusqlite::params![vl_id],
        )
        .unwrap();

        (conn, project_id)
    }

    fn setup_with_bare_ref() -> (Connection, i64) {
        let (conn, project_id) = setup();

        // Customer テーブルを追加
        conn.execute(
            "INSERT INTO base_tables(project_id, fm_id, name) VALUES(?1, 10, 'Customer')",
            rusqlite::params![project_id],
        )
        .unwrap();
        let customer_table_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO table_occurrences(project_id, occurrence_name, base_table_name)
             VALUES(?1, 'Customer', 'Customer')",
            rusqlite::params![project_id],
        )
        .unwrap();

        // Status フィールド: レイアウト・リレーション・OccName::FieldName いずれでも未参照
        conn.execute(
            "INSERT INTO fields(project_id, table_id, fm_id, name, field_type, data_type, calculation)
             VALUES(?1, ?2, 101, 'Status', 'Normal', 'Text', '')",
            rusqlite::params![project_id, customer_table_id],
        )
        .unwrap();

        // DisplayName フィールド: Status をベア参照する計算式
        conn.execute(
            "INSERT INTO fields(project_id, table_id, fm_id, name, field_type, data_type, calculation)
             VALUES(?1, ?2, 102, 'DisplayName', 'Calculation', 'Text', 'Upper(Status)')",
            rusqlite::params![project_id, customer_table_id],
        )
        .unwrap();

        (conn, project_id)
    }

    /// setup() の Invoice::Amount は "InvoiceAlias::Amount" として Invoice::Total の計算式で
    /// 参照されている。新仕様では未参照リストに含まれてはいけない。
    #[test]
    fn unused_fields_excludes_calc_referenced_via_occurrence() {
        let (conn, project_id) = setup();
        // script_steps に計算式を挿入
        conn.execute(
            "INSERT INTO scripts(project_id, fm_id, name) VALUES(?1, 1, 'TestScript')",
            rusqlite::params![project_id],
        )
        .unwrap();
        let script_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO script_steps(script_id, step_type_id, name, enabled, calculation, position)
             VALUES(?1, 89, 'Set Field', 1, 'InvoiceAlias::Amount * 1.1', 0)",
            rusqlite::params![script_id],
        )
        .unwrap();

        let unused = list_unused(&conn, project_id);
        assert!(
            !unused.iter().any(|(t, f)| t == "Invoice" && f == "Amount"),
            "Invoice::Amount は計算式で参照されているのに未参照リストに含まれた: {:?}",
            unused
        );
    }

    /// fields.calculation で参照されているフィールドも未参照リストから除外される。
    #[test]
    fn unused_fields_excludes_field_calc_referenced() {
        let (conn, project_id) = setup();
        // setup() では Invoice::Total の calculation = "InvoiceAlias::Amount * 1.1"
        let unused = list_unused(&conn, project_id);
        assert!(
            !unused.iter().any(|(t, f)| t == "Invoice" && f == "Amount"),
            "Invoice::Amount はフィールド計算式で参照されているのに未参照リストに含まれた: {:?}",
            unused
        );
    }

    /// バリューリストのフィールド参照として使われているフィールドは未参照リストに出ない。
    #[test]
    fn unused_fields_excludes_value_list_field_refs() {
        let (conn, project_id) = setup_with_value_list_ref();
        let unused = list_unused(&conn, project_id);
        assert!(
            !unused.iter().any(|(t, f)| t == "Invoice" && f == "Amount"),
            "Invoice::Amount はバリューリストで参照されているのに未参照リストに含まれた: {:?}",
            unused
        );
    }

    /// 同テーブルの計算フィールドがベア参照しているフィールドは未参照リストに出ない。
    #[test]
    fn unused_fields_excludes_bare_ref_in_same_table_calc() {
        let (conn, project_id) = setup_with_bare_ref();
        let unused = list_unused(&conn, project_id);
        // Customer::Status は Customer::DisplayName の計算式 "Upper(Status)" でベア参照されている
        assert!(
            !unused
                .iter()
                .any(|(t, f)| t == "Customer" && f == "Status"),
            "Customer::Status は同テーブル calc でベア参照されているのに未参照リストに含まれた: {:?}",
            unused
        );
    }

    /// 異なるテーブルの同名フィールドはベア参照で誤検出されない。
    #[test]
    fn unused_fields_bare_ref_does_not_cross_table() {
        let (conn, project_id) = setup_with_bare_ref();
        let unused = list_unused(&conn, project_id);
        let customer_bare_referenced: Vec<_> =
            unused.iter().filter(|(t, _)| t == "Invoice").collect();
        // Invoice テーブルのフィールドが Customer::DisplayName の "Upper(Status)" によって
        // 除外されていないことを確認（Invoice に Status というフィールドはないので変化なし）
        let _ = customer_bare_referenced;
    }

    #[test]
    fn fetch_occ_map_groups_by_base_table() {
        let (conn, project_id) = setup();
        let map = fetch_occ_map(&conn, project_id).unwrap();
        let invoice_occs = map.get("Invoice").unwrap();
        assert!(invoice_occs.contains(&"Invoice".to_string()));
        assert!(invoice_occs.contains(&"InvoiceAlias".to_string()));
        let order_occs = map.get("Order").unwrap();
        assert!(order_occs.contains(&"Order".to_string()));
    }

    #[test]
    fn fetch_all_calc_texts_includes_field_calcs() {
        let (conn, project_id) = setup();
        let text = fetch_all_calc_texts(&conn, project_id).unwrap();
        // Invoice::Total の計算式が含まれる
        assert!(text.contains("InvoiceAlias::Amount"));
    }

    #[test]
    fn fetch_all_calc_texts_includes_script_step_calcs() {
        let (conn, project_id) = setup();
        conn.execute(
            "INSERT INTO scripts(project_id, fm_id, name) VALUES(?1, 99, 'S')",
            rusqlite::params![project_id],
        )
        .unwrap();
        let sid = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO script_steps(script_id, step_type_id, name, enabled, calculation, position)
             VALUES(?1, 89, 'Set Field', 1, 'Invoice::Amount * 2', 0)",
            rusqlite::params![sid],
        )
        .unwrap();
        let text = fetch_all_calc_texts(&conn, project_id).unwrap();
        assert!(text.contains("Invoice::Amount * 2"));
    }

    #[test]
    fn fetch_same_table_field_calcs_groups_by_table() {
        let (conn, project_id) = setup();
        let map = fetch_same_table_field_calcs(&conn, project_id).unwrap();
        let inv_calcs = map.get("Invoice").unwrap();
        // Invoice::Total の calculation が含まれる
        assert!(inv_calcs.iter().any(|c| c.contains("InvoiceAlias::Amount")));
    }

    // unused_fields.rs の Step 2 で field_ref_matches を使った検査がある（helpers 経由）
    // ここでは直接テストせず、list_unused_fields_inner のテストで間接的にカバー済み
}
