use rusqlite::params;

/// ベーステーブルに紐づく全オカレンス名を返す（ソリューション全体スコープ）。
///
/// 同一ソリューション内の全プロジェクトを対象にするため、分離モデルでも
/// プログラムファイル側のオカレンス名がデータファイルの project_id から取得できる。
pub(crate) fn fetch_occ_names(
    conn: &rusqlite::Connection,
    project_id: i64,
    base_table_name: &str,
) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT occurrence_name FROM table_occurrences
         WHERE project_id IN (
           SELECT id FROM projects
           WHERE solution_id = (SELECT solution_id FROM projects WHERE id = ?1)
         )
         AND base_table_name = ?2",
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
pub(crate) fn field_ref_matches(text: &str, occ_names: &[String], field_name: &str) -> bool {
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
pub(crate) fn has_bare_field_ref(text: &str, field_name: &str) -> bool {
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
pub(crate) fn is_fm_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '＿'
}

// ---------------------------------------------------------------------------
// テスト用セットアップヘルパー
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod test_helpers {
    use rusqlite::Connection;

    use crate::db::schema::initialize;

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
    /// 分離モデル（プログラムファイル + データファイル）用セットアップ。
    ///
    /// - program_project: table_occurrence "Customers" → base_table_name="Customer", source_file="DataFile"
    /// - data_project:    base_table "Customer", field "FirstName"
    ///
    /// 戻り値: (conn, program_project_id, data_project_id)
    pub(crate) fn setup_cross_project() -> (Connection, i64, i64) {
        let conn = Connection::open_in_memory().unwrap();
        initialize(&conn).unwrap();

        conn.execute("INSERT INTO solutions(name) VALUES('sol')", [])
            .unwrap();
        let solution_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO projects(solution_id, name, fm_version) VALUES(?1, 'ProgramFile', '21')",
            [solution_id],
        )
        .unwrap();
        let program_project_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO projects(solution_id, name, fm_version) VALUES(?1, 'DataFile', '21')",
            [solution_id],
        )
        .unwrap();
        let data_project_id = conn.last_insert_rowid();

        // data_project: base_table "Customer" + field "FirstName"
        conn.execute(
            "INSERT INTO base_tables(project_id, fm_id, name) VALUES(?1, 1, 'Customer')",
            [data_project_id],
        )
        .unwrap();
        let customer_table_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO fields(project_id, table_id, fm_id, name, field_type, data_type)
             VALUES(?1, ?2, 1, 'FirstName', 'Normal', 'Text')",
            rusqlite::params![data_project_id, customer_table_id],
        )
        .unwrap();

        // program_project: occurrence "Customers" → Customer in DataFile
        conn.execute(
            "INSERT INTO table_occurrences(project_id, occurrence_name, base_table_name, source_file)
             VALUES(?1, 'Customers', 'Customer', 'DataFile')",
            [program_project_id],
        )
        .unwrap();

        (conn, program_project_id, data_project_id)
    }

    pub(crate) fn setup() -> (Connection, i64) {
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

    /// レイアウト参照テスト用: Invoice レイアウトと layout_field_refs を追加する。
    pub(crate) fn setup_with_layout_refs() -> (Connection, i64) {
        let (conn, project_id) = setup();

        // レイアウト: Invoice をメインテーブルとして使用
        conn.execute(
            "INSERT INTO layouts(project_id, fm_id, name, table_occurrence_name, position)
             VALUES(?1, 1, 'InvoiceList', 'Invoice', 0)",
            rusqlite::params![project_id],
        )
        .unwrap();
        let layout_id = conn.last_insert_rowid();

        // layout_field_refs: Invoice::Amount を配置
        conn.execute(
            "INSERT INTO layout_field_refs(layout_id, table_occurrence, field_name)
             VALUES(?1, 'Invoice', 'Amount')",
            rusqlite::params![layout_id],
        )
        .unwrap();

        // layout_field_refs: InvoiceAlias::Total も配置（別オカレンス名経由）
        conn.execute(
            "INSERT INTO layout_field_refs(layout_id, table_occurrence, field_name)
             VALUES(?1, 'InvoiceAlias', 'Total')",
            rusqlite::params![layout_id],
        )
        .unwrap();

        (conn, project_id)
    }
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use test_helpers::{setup, setup_cross_project};

    #[test]
    fn bare_ref_detects_standalone() {
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

    #[test]
    fn fetch_occ_names_returns_names_for_table() {
        let (conn, project_id) = setup();
        let names = fetch_occ_names(&conn, project_id, "Invoice").unwrap();
        assert!(names.contains(&"Invoice".to_string()));
        assert!(names.contains(&"InvoiceAlias".to_string()));
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn fetch_occ_names_returns_empty_for_unknown_table() {
        let (conn, project_id) = setup();
        let names = fetch_occ_names(&conn, project_id, "NonExistent").unwrap();
        assert!(names.is_empty());
    }

    #[test]
    fn fetch_occ_names_solution_scope_finds_occ_in_other_project() {
        // データファイルの project_id を渡したとき、
        // プログラムファイル側のオカレンス名も返ること
        let (conn, _program_project_id, data_project_id) = setup_cross_project();
        let names = fetch_occ_names(&conn, data_project_id, "Customer").unwrap();
        assert!(
            names.contains(&"Customers".to_string()),
            "should find 'Customers' occurrence from program project: got {names:?}"
        );
    }
}
