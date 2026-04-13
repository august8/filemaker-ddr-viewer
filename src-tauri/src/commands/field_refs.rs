//! フィールド参照解析コマンド。

use rusqlite::{params, OptionalExtension as _};
use serde::{Deserialize, Serialize};

use crate::{commands::CommandError, AppState};

// ---------------------------------------------------------------------------
// 公開型
// ---------------------------------------------------------------------------

/// フィールドがリレーションキーとして使用されているリレーションの情報。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldRelKeyRef {
    pub relationship_id: i64,
    pub relationship_name: String,
    pub left_table: String,
    pub right_table: String,
    pub operator: String,
    /// "left" or "right" — このフィールドがキーとして使われている側
    pub side: String,
}

/// 未使用フィールドの情報。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnusedFieldRow {
    pub table_name: String,
    pub field_name: String,
    pub field_type: String,
    pub data_type: String,
    pub field_id: i64,
}

/// このフィールドを計算式で参照している他フィールドの情報。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldCalcRef {
    pub field_id: i64,
    pub field_name: String,
    pub table_name: String,
    pub table_id: i64,
}

/// フィールドを参照しているスクリプトの一覧。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldRefScript {
    pub script_id: i64,
    pub script_name: String,
}

/// フィールドのテーブルを使用しているレイアウトの一覧。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldRefLayout {
    pub layout_id: i64,
    pub layout_name: String,
}

/// テーブルオカレンス名とフィールド名から解決したフィールドの位置情報。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldLocation {
    pub table_id: i64,
    pub field_id: i64,
    pub table_name: String,
}

/// レイアウトフィールド参照のデバッグ情報。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutRefDebugInfo {
    pub occurrence_count: i64,
    pub layout_field_ref_count: i64,
    pub sample_occurrences: Vec<String>,
    pub sample_field_refs: Vec<String>,
}

// ---------------------------------------------------------------------------
// Tauri コマンド
// ---------------------------------------------------------------------------

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
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;

    // 1. ベーステーブルに紐づく全オカレンス名を取得
    let occ_names =
        fetch_occ_names(&db.conn, project_id, &table_name).map_err(CommandError::from)?;

    // 2. 全スクリプトステップを取得
    let mut stmt = db
        .conn
        .prepare(
            "SELECT s.id, s.name,
                    COALESCE(ss.step_text, ''),
                    COALESCE(ss.calculation, '')
             FROM scripts s
             JOIN script_steps ss ON ss.script_id = s.id
             WHERE s.project_id = ?1",
        )
        .map_err(CommandError::from)?;
    let step_rows: Vec<(i64, String, String, String)> = stmt
        .query_map(params![project_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(CommandError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(CommandError::from)?;

    // 3. Rust でフィルタリング（DISTINCT script_id）
    let mut seen = std::collections::HashSet::new();
    let results = step_rows
        .into_iter()
        .filter(|(_, _, step_text, calc)| {
            field_ref_matches(step_text, &occ_names, &field_name)
                || field_ref_matches(calc, &occ_names, &field_name)
        })
        .filter_map(|(script_id, script_name, _, _)| {
            seen.insert(script_id).then_some(FieldRefScript {
                script_id,
                script_name,
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
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;

    // 1. ベーステーブルに紐づく全オカレンス名を取得
    let occ_names =
        fetch_occ_names(&db.conn, project_id, &table_name).map_err(CommandError::from)?;

    // 2. 全フィールド（calculation 非空・自分自身除く）を取得
    let mut stmt = db
        .conn
        .prepare(
            "SELECT f.id, f.name, bt.name, bt.id, f.calculation
             FROM fields f
             JOIN base_tables bt ON bt.id = f.table_id
             WHERE bt.project_id = ?1
               AND f.calculation IS NOT NULL
               AND f.calculation != ''
               AND NOT (bt.name = ?2 AND f.name = ?3)
             ORDER BY bt.name, f.name",
        )
        .map_err(CommandError::from)?;
    let candidates: Vec<(i64, String, String, i64, String)> = stmt
        .query_map(params![project_id, table_name, field_name], |row| {
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

    // 3. Rust でフィルタリング
    let results = candidates
        .into_iter()
        .filter(|(_, _, _, _, calc)| field_ref_matches(calc, &occ_names, &field_name))
        .map(
            |(field_id, field_name, table_name, table_id, _)| FieldCalcRef {
                field_id,
                field_name,
                table_name,
                table_id,
            },
        )
        .collect();
    Ok(results)
}

/// フィールドが配置されているレイアウトの一覧を返す。
///
/// 検索手順：
/// 1. `table_name`（ベーステーブル名）に対応するオカレンス名一覧を取得
/// 2. メインテーブル（`table_occurrence_name`）がそのオカレンスの1つであるレイアウトを絞り込む
/// 3. さらに、そのレイアウト上に `field_name` が配置されているかを確認
#[tauri::command]
pub async fn get_field_layout_refs(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    table_name: String,
    field_name: String,
) -> Result<Vec<FieldRefLayout>, CommandError> {
    if table_name.is_empty() || field_name.is_empty() {
        return Ok(vec![]);
    }
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let mut stmt = db
        .conn
        .prepare(
            // layout の main TO が対象ベーステーブルのオカレンスであり、
            // かつそのレイアウト上に対象フィールドが配置されていること（フラット JOIN）
            "SELECT DISTINCT l.id, l.name
             FROM layouts l
             JOIN table_occurrences toc_main
               ON toc_main.project_id = l.project_id
              AND toc_main.occurrence_name = l.table_occurrence_name
              AND toc_main.base_table_name = ?2
             JOIN layout_field_refs lfr
               ON lfr.layout_id = l.id
              AND lfr.field_name = ?3
             JOIN table_occurrences toc_field
               ON toc_field.project_id = ?1
              AND toc_field.occurrence_name = lfr.table_occurrence
              AND toc_field.base_table_name = ?2
             WHERE l.project_id = ?1
             ORDER BY l.position, l.name",
        )
        .map_err(CommandError::from)?;
    let rows = stmt
        .query_map(params![project_id, table_name, field_name], |row| {
            Ok(FieldRefLayout {
                layout_id: row.get(0)?,
                layout_name: row.get(1)?,
            })
        })
        .map_err(CommandError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(CommandError::from)?;
    Ok(rows)
}

/// テーブルオカレンス名とフィールド名からフィールドの DB ID・テーブル DB ID・ベーステーブル名を解決する。
#[tauri::command]
pub async fn resolve_layout_field(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    occurrence_name: String,
    field_name: String,
) -> Result<Option<FieldLocation>, CommandError> {
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let result = db
        .conn
        .query_row(
            "SELECT bt.id, f.id, bt.name
               FROM fields f
               JOIN base_tables bt ON bt.id = f.table_id
               JOIN table_occurrences toc
                 ON toc.base_table_name = bt.name
                AND toc.project_id = bt.project_id
              WHERE bt.project_id = ?1
                AND toc.occurrence_name = ?2
                AND f.name = ?3
              LIMIT 1",
            params![project_id, occurrence_name, field_name],
            |row| {
                Ok(FieldLocation {
                    table_id: row.get(0)?,
                    field_id: row.get(1)?,
                    table_name: row.get(2)?,
                })
            },
        )
        .optional()
        .map_err(CommandError::from)?;
    Ok(result)
}

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
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let mut stmt = db
        .conn
        .prepare(
            "SELECT r.id, r.name, r.left_table, r.right_table, jp.operator, 'left'
             FROM relationships r
             JOIN join_predicates jp ON jp.relationship_id = r.id
             JOIN table_occurrences toc
               ON toc.occurrence_name = r.left_table AND toc.project_id = r.project_id
             WHERE r.project_id = ?1
               AND jp.left_field = ?3
               AND toc.base_table_name = ?2
             UNION
             SELECT r.id, r.name, r.left_table, r.right_table, jp.operator, 'right'
             FROM relationships r
             JOIN join_predicates jp ON jp.relationship_id = r.id
             JOIN table_occurrences toc
               ON toc.occurrence_name = r.right_table AND toc.project_id = r.project_id
             WHERE r.project_id = ?1
               AND jp.right_field = ?3
               AND toc.base_table_name = ?2
             ORDER BY r.name",
        )
        .map_err(CommandError::from)?;
    let rows = stmt
        .query_map(params![project_id, table_name, field_name], |row| {
            Ok(FieldRelKeyRef {
                relationship_id: row.get(0)?,
                relationship_name: row.get(1)?,
                left_table: row.get(2)?,
                right_table: row.get(3)?,
                operator: row.get(4)?,
                side: row.get(5)?,
            })
        })
        .map_err(CommandError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(CommandError::from)?;
    Ok(rows)
}

/// レイアウト・リレーションから一度も参照されていないフィールドの一覧を返す。
///
/// 検査対象: layout_field_refs / layout_objects / join_predicates
/// ※ スクリプトステップ内のテキスト参照は対象外。
#[tauri::command]
pub async fn list_unused_fields(
    state: tauri::State<'_, AppState>,
    project_id: i64,
) -> Result<Vec<UnusedFieldRow>, CommandError> {
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let mut stmt = db
        .conn
        .prepare(
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
        )
        .map_err(CommandError::from)?;
    let rows = stmt
        .query_map(params![project_id], |row| {
            Ok(UnusedFieldRow {
                table_name: row.get(0)?,
                field_name: row.get(1)?,
                field_type: row.get(2)?,
                data_type: row.get(3)?,
                field_id: row.get(4)?,
            })
        })
        .map_err(CommandError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(CommandError::from)?;
    Ok(rows)
}

/// レイアウトフィールド参照のデバッグ情報を返す。
#[tauri::command]
pub async fn get_layout_ref_debug_info(
    state: tauri::State<'_, AppState>,
    project_id: i64,
) -> Result<LayoutRefDebugInfo, CommandError> {
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;

    let occurrence_count: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM table_occurrences WHERE project_id = ?1",
            params![project_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let layout_field_ref_count: i64 = db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM layout_field_refs lfr
             JOIN layouts l ON l.id = lfr.layout_id
             WHERE l.project_id = ?1",
            params![project_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let mut stmt = db
        .conn
        .prepare(
            "SELECT occurrence_name || ' -> ' || base_table_name
             FROM table_occurrences WHERE project_id = ?1 LIMIT 10",
        )
        .map_err(CommandError::from)?;
    let sample_occurrences = stmt
        .query_map(params![project_id], |r| r.get::<_, String>(0))
        .map_err(CommandError::from)?
        .filter_map(|r| r.ok())
        .collect();

    let mut stmt2 = db
        .conn
        .prepare(
            "SELECT l.name || ' | ' || lfr.table_occurrence || '::' || lfr.field_name
             FROM layout_field_refs lfr
             JOIN layouts l ON l.id = lfr.layout_id
             WHERE l.project_id = ?1 LIMIT 10",
        )
        .map_err(CommandError::from)?;
    let sample_field_refs = stmt2
        .query_map(params![project_id], |r| r.get::<_, String>(0))
        .map_err(CommandError::from)?
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
// 内部ヘルパー
// ---------------------------------------------------------------------------

/// ベーステーブルに紐づく全オカレンス名を返す。
fn fetch_occ_names(
    conn: &rusqlite::Connection,
    project_id: i64,
    base_table_name: &str,
) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT occurrence_name FROM table_occurrences
         WHERE project_id = ?1 AND base_table_name = ?2",
    )?;
    let rows = stmt
        .query_map(params![project_id, base_table_name], |r| r.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    Ok(rows)
}

/// FileMaker 計算式・スクリプトテキスト中でフィールドが参照されているか判定。
///
/// - `OccName::field_name` 形式（任意のオカレンス名）
/// - 識別子境界を考慮した bare `field_name` 形式
///
/// のいずれかにマッチすれば `true` を返す。
fn field_ref_matches(text: &str, occ_names: &[String], field_name: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    // OccName::FieldName パターン
    if occ_names
        .iter()
        .any(|occ| text.contains(&format!("{}::{}", occ, field_name)))
    {
        return true;
    }
    // bare FieldName パターン（識別子境界チェック）
    has_bare_field_ref(text, field_name)
}

/// `text` 中に `field_name` が識別子として単独で現れるか判定。
///
/// 直前が識別子文字または `:` でなく、直後が識別子文字または `(` でない位置に
/// `field_name` が存在する場合に `true` を返す。
fn has_bare_field_ref(text: &str, field_name: &str) -> bool {
    let mut from = 0;
    while let Some(pos) = text[from..].find(field_name) {
        let abs = from + pos;
        let end = abs + field_name.len();
        let before_ok = text[..abs]
            .chars()
            .last()
            .is_none_or(|c| !is_fm_ident_char(c) && c != ':');
        let after_ok = text[end..]
            .chars()
            .next()
            .is_none_or(|c| !is_fm_ident_char(c) && c != '(');
        if before_ok && after_ok {
            return true;
        }
        // field_name.len() バイト分進める（UTF-8 境界が保証される）
        from = abs + field_name.len();
    }
    false
}

/// FileMaker 識別子文字（フィールド名・TO名に使える文字）の判定。
fn is_fm_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '＿'
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    use crate::db::schema::initialize;

    // -----------------------------------------------------------------------
    // ヘルパー: has_bare_field_ref / field_ref_matches の単体テスト
    // -----------------------------------------------------------------------

    #[test]
    fn bare_ref_detects_standalone() {
        // フィールド名が単独で現れる
        assert!(has_bare_field_ref("Amount * 1.1", "Amount"));
        assert!(has_bare_field_ref("Amount", "Amount"));
        assert!(has_bare_field_ref("(Amount + Tax)", "Amount"));
    }

    #[test]
    fn bare_ref_ignores_qualified() {
        // OccName::FieldName は bare ref ではない（`:` が直前にある）
        assert!(!has_bare_field_ref("Invoice::Amount * 1.1", "Amount"));
        assert!(!has_bare_field_ref("Inv::Amount", "Amount"));
    }

    #[test]
    fn bare_ref_ignores_substring() {
        // 別フィールド名の一部にマッチしない
        assert!(!has_bare_field_ref("TotalAmount + 1", "Amount"));
        assert!(!has_bare_field_ref("合計金額 + 1", "金額"));
    }

    #[test]
    fn bare_ref_ignores_function_call() {
        // 関数呼び出し（直後が `(`）は除外
        assert!(!has_bare_field_ref("Amount(x)", "Amount"));
    }

    #[test]
    fn field_ref_matches_via_occ_name() {
        // OccName が base_table_name と異なる場合も検出できる
        let occ_names = vec!["InvoiceAlias".to_string()];
        assert!(field_ref_matches(
            "InvoiceAlias::Amount * 1.1",
            &occ_names,
            "Amount"
        ));
        // base_table_name（Invoice）では検出されない
        let empty: Vec<String> = vec![];
        assert!(!field_ref_matches("InvoiceAlias::Amount", &empty, "Amount"));
    }

    // -----------------------------------------------------------------------
    // ヘルパー: DB セットアップ
    // -----------------------------------------------------------------------

    /// インメモリ DB を作り、プロジェクト・テーブル・オカレンス・フィールドを挿入する。
    ///
    /// テーブル構成:
    /// - Invoice (occurrence: "Invoice", "InvoiceAlias")
    /// - Order   (occurrence: "Order")
    ///
    /// フィールド:
    /// - Invoice::Amount          計算式なし
    /// - Invoice::Total           計算式 "InvoiceAlias::Amount * 1.1"  (オカレンス名経由)
    /// - Invoice::合計金額         計算式 "Amount + Tax"                (bare ref + 部分一致の罠)
    /// - Order::Total             計算式 "Invoice::Amount + 100"        (base table 名と一致)
    /// - Order::Note              計算式 "Order::Amount"                (別フィールド)
    fn setup() -> (Connection, i64) {
        let conn = Connection::open_in_memory().unwrap();
        initialize(&conn).unwrap();

        conn.execute("INSERT INTO solutions(name) VALUES('sol')", [])
            .unwrap();
        let solution_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO projects(solution_id, name, fm_version) VALUES(?1, 'test.fmp12', '19')",
            [solution_id],
        )
        .unwrap();
        let project_id = conn.last_insert_rowid();

        // base_tables
        conn.execute(
            "INSERT INTO base_tables(project_id, fm_id, name) VALUES(?1, 1, 'Invoice')",
            [project_id],
        )
        .unwrap();
        let invoice_table_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO base_tables(project_id, fm_id, name) VALUES(?1, 2, 'Order')",
            [project_id],
        )
        .unwrap();
        let order_table_id = conn.last_insert_rowid();

        // table_occurrences（Invoice に 2 つのオカレンス）
        conn.execute(
            "INSERT INTO table_occurrences(project_id, occurrence_name, base_table_name)
             VALUES(?1, 'Invoice', 'Invoice')",
            [project_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO table_occurrences(project_id, occurrence_name, base_table_name)
             VALUES(?1, 'InvoiceAlias', 'Invoice')",
            [project_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO table_occurrences(project_id, occurrence_name, base_table_name)
             VALUES(?1, 'Order', 'Order')",
            [project_id],
        )
        .unwrap();

        // fields
        for (fm_id, table_id, name, calc) in [
            (1, invoice_table_id, "Amount", ""),
            (2, invoice_table_id, "Total", "InvoiceAlias::Amount * 1.1"),
            (3, invoice_table_id, "合計金額", "Amount + Tax"), // bare ref to Amount
            (4, order_table_id, "Total", "Invoice::Amount + 100"),
            (5, order_table_id, "Note", "Order::Amount"),
        ] {
            conn.execute(
                "INSERT INTO fields(project_id, table_id, fm_id, name, field_type, data_type, calculation)
                 VALUES(?1, ?2, ?3, ?4, 'Calculation', 'Number', ?5)",
                rusqlite::params![project_id, table_id, fm_id, name, calc],
            )
            .unwrap();
        }

        (conn, project_id)
    }

    /// `get_field_calc_refs` の内部ロジックを直接呼び出すヘルパー。
    fn calc_refs(
        conn: &Connection,
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

    // -----------------------------------------------------------------------
    // get_field_calc_refs 相当のテスト
    // -----------------------------------------------------------------------

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
}
