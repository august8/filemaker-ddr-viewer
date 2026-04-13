//! FTS5 全文検索と LIKE 部分一致検索。
//!
//! 検索スコープは以下の3段階:
//! - 全体 (All):        project_id=None, solution_id=None → DB 内すべてを検索
//! - ソリューション:     solution_id=Some → そのソリューション内全プロジェクトを横断
//! - プロジェクト:       project_id=Some, solution_id=None → そのプロジェクト内のみ

use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::db::{Database, DbError};

// ---------------------------------------------------------------------------
// 公開データ型
// ---------------------------------------------------------------------------

/// 検索の1件の結果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// 結果が属するプロジェクトの DB id（全体・ソリューション横断検索でも必ず設定される）
    pub project_id: i64,
    pub element_type: String,
    pub element_id: i64,
    pub name: String,
    pub snippet: String,
    pub rank: f64,
    /// field の場合のみ: 親テーブルの DB id
    pub parent_id: Option<i64>,
    /// field の場合のみ: 親テーブル名（表示用）
    pub parent_name: Option<String>,
}

// ---------------------------------------------------------------------------
// 内部: スコープ種別
// ---------------------------------------------------------------------------

enum ScopeFilter {
    All,
    Project(i64),
    Solution(i64),
}

impl ScopeFilter {
    fn from_opts(project_id: Option<i64>, solution_id: Option<i64>) -> Self {
        if let Some(sid) = solution_id {
            ScopeFilter::Solution(sid)
        } else if let Some(pid) = project_id {
            ScopeFilter::Project(pid)
        } else {
            ScopeFilter::All
        }
    }
}

// ---------------------------------------------------------------------------
// 内部: 行マッパー
// ---------------------------------------------------------------------------

fn map_search_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SearchResult> {
    Ok(SearchResult {
        project_id: row.get(0)?,
        element_type: row.get(1)?,
        element_id: row.get(2)?,
        name: row.get(3)?,
        snippet: row.get(4)?,
        rank: row.get(5)?,
        parent_id: row.get(6)?,
        parent_name: row.get(7)?,
    })
}

// ---------------------------------------------------------------------------
// FTS5 全文検索（前方一致 "word"*）
// ---------------------------------------------------------------------------

/// FTS5 で全文検索する。`query` はスペース区切りのキーワード列。
///
/// - `project_id=None, solution_id=None` → 全体検索
/// - `solution_id=Some(sid)` → ソリューション内横断
/// - `project_id=Some(pid), solution_id=None` → プロジェクト内のみ
pub fn search(
    db: &Database,
    project_id: Option<i64>,
    solution_id: Option<i64>,
    query: &str,
    limit: i64,
) -> Result<Vec<SearchResult>, DbError> {
    let fts_query = build_fts_query(query);
    if fts_query.is_empty() {
        return Ok(Vec::new());
    }

    let scope = ScopeFilter::from_opts(project_id, solution_id);

    // field の場合のみ LEFT JOIN で親テーブル情報を解決する
    const JOIN_CLAUSE: &str = "
        SELECT f.project_id, f.element_type, f.element_id, f.name, f.content, f.rank,
               fld.table_id, bt.name AS table_name
          FROM fts f
          LEFT JOIN fields fld
            ON f.element_type = 'field' AND fld.id = f.element_id
          LEFT JOIN base_tables bt ON bt.id = fld.table_id";

    match scope {
        ScopeFilter::All => {
            let sql = format!(
                "WITH fts AS (
                    SELECT project_id, element_type, element_id, name, content, rank
                      FROM search_index
                     WHERE search_index MATCH ?1
                     ORDER BY rank LIMIT ?2
                 ){JOIN_CLAUSE}"
            );
            let mut stmt = db.conn.prepare(&sql)?;
            let rows = stmt.query_map(params![fts_query, limit], map_search_row)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
        }
        ScopeFilter::Project(pid) => {
            let sql = format!(
                "WITH fts AS (
                    SELECT project_id, element_type, element_id, name, content, rank
                      FROM search_index
                     WHERE search_index MATCH ?1
                       AND project_id = ?2
                     ORDER BY rank LIMIT ?3
                 ){JOIN_CLAUSE}"
            );
            let mut stmt = db.conn.prepare(&sql)?;
            let rows = stmt.query_map(params![fts_query, pid, limit], map_search_row)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
        }
        ScopeFilter::Solution(sid) => {
            let sql = format!(
                "WITH fts AS (
                    SELECT project_id, element_type, element_id, name, content, rank
                      FROM search_index
                     WHERE search_index MATCH ?1
                       AND project_id IN (SELECT id FROM projects WHERE solution_id = ?2)
                     ORDER BY rank LIMIT ?3
                 ){JOIN_CLAUSE}"
            );
            let mut stmt = db.conn.prepare(&sql)?;
            let rows = stmt.query_map(params![fts_query, sid, limit], map_search_row)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
        }
    }
}

// ---------------------------------------------------------------------------
// LIKE 部分一致検索（名前のみ対象）
// ---------------------------------------------------------------------------

/// LIKE `'%word%'` による名前の部分一致検索。
///
/// FTS5 は infix 検索非対応のため、名前フィールドに対して LIKE を使う。
/// スコープ指定は `search()` と同じ。
pub fn search_contains(
    db: &Database,
    project_id: Option<i64>,
    solution_id: Option<i64>,
    query: &str,
    limit: i64,
) -> Result<Vec<SearchResult>, DbError> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    // LIKE パターン: \, %, _ をエスケープ
    let like_pattern = format!(
        "%{}%",
        query
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    );

    let scope = ScopeFilter::from_opts(project_id, solution_id);

    // UNION ALL の各クエリに埋め込むフィルター句（動的生成、値は別途パラメータ）
    // fields は base_tables 経由なので bt.project_id を使う
    let (pf, btpf) = match &scope {
        ScopeFilter::All => ("1=1".to_owned(), "1=1".to_owned()),
        ScopeFilter::Project(_) => (
            "project_id = ?2".to_owned(),
            "bt.project_id = ?2".to_owned(),
        ),
        ScopeFilter::Solution(_) => (
            "project_id IN (SELECT id FROM projects WHERE solution_id = ?2)".to_owned(),
            "bt.project_id IN (SELECT id FROM projects WHERE solution_id = ?2)".to_owned(),
        ),
    };

    let sql = format!(
        "SELECT project_id, 'table' AS element_type, id AS element_id, name, '' AS content,
                0.0 AS rank, NULL AS parent_id, NULL AS parent_name
           FROM base_tables WHERE {pf} AND name LIKE ?1 ESCAPE '\\'
         UNION ALL
         SELECT bt.project_id, 'field', f.id, f.name, '', 0.0, f.table_id, bt.name
           FROM fields f JOIN base_tables bt ON bt.id = f.table_id
          WHERE {btpf} AND f.name LIKE ?1 ESCAPE '\\'
         UNION ALL
         SELECT project_id, 'script', id, name, '', 0.0, NULL, NULL
           FROM scripts WHERE {pf} AND name LIKE ?1 ESCAPE '\\'
         UNION ALL
         SELECT project_id, 'layout', id, name, '', 0.0, NULL, NULL
           FROM layouts WHERE {pf} AND name LIKE ?1 ESCAPE '\\'
         UNION ALL
         SELECT project_id, 'value_list', id, name, '', 0.0, NULL, NULL
           FROM value_lists WHERE {pf} AND name LIKE ?1 ESCAPE '\\'
         UNION ALL
         SELECT project_id, 'custom_function', id, name, '', 0.0, NULL, NULL
           FROM custom_functions WHERE {pf} AND name LIKE ?1 ESCAPE '\\'
         ORDER BY name LIMIT ?{limit_pos}",
        pf = pf,
        btpf = btpf,
        limit_pos = if matches!(scope, ScopeFilter::All) { 2 } else { 3 },
    );

    let mut stmt = db.conn.prepare(&sql)?;

    let rows = match scope {
        ScopeFilter::All => {
            stmt.query_map(params![like_pattern, limit], map_search_row)?
        }
        ScopeFilter::Project(pid) => {
            stmt.query_map(params![like_pattern, pid, limit], map_search_row)?
        }
        ScopeFilter::Solution(sid) => {
            stmt.query_map(params![like_pattern, sid, limit], map_search_row)?
        }
    };

    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

// ---------------------------------------------------------------------------
// 内部: FTS クエリ構築
// ---------------------------------------------------------------------------

/// スペース区切りのキーワードを FTS5 プレフィックス検索クエリに変換する。
///
/// 例: `"hello world"` → `"hello"* "world"*`
fn build_fts_query(query: &str) -> String {
    // name + content 両方を検索する（カラム指定なし = FTS5 全カラム）。
    // スクリプトステップ内容・フィールド計算式・カスタム関数使用箇所などを検索可能にする。
    query
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .map(|w| {
            // FTS5 の特殊文字をエスケープ
            let escaped = w.replace('"', "\"\"");
            format!("\"{escaped}\"*")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::repository::{insert_ddr_file, insert_solution};
    use crate::db::Database;
    use crate::parser::parse_ddr;
    use rstest::rstest;

    const MINIMAL_XML: &str = include_str!("../../../../tests/fixtures/minimal.xml");

    fn db_with_minimal() -> (Database, i64, i64) {
        let mut db = Database::open_in_memory().unwrap();
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let sid = insert_solution(&mut db, "TestSolution", Some("/tmp/test.xml")).unwrap();
        let pid = insert_ddr_file(&mut db, &ddr, sid, Some("/tmp/test.xml")).unwrap();
        (db, sid, pid)
    }

    // ---- FTS5 基本 ----

    #[test]
    fn search_finds_table_by_name() {
        let (db, _sid, pid) = db_with_minimal();
        let results = search(&db, Some(pid), None, "Contact", 10).unwrap();
        assert!(
            results
                .iter()
                .any(|r| r.element_type == "table" && r.name == "Contact"),
            "should find 'Contact' table"
        );
    }

    #[test]
    fn search_finds_script_by_name() {
        let (db, _sid, pid) = db_with_minimal();
        let results = search(&db, Some(pid), None, "Hello", 10).unwrap();
        assert!(
            results.iter().any(|r| r.element_type == "script"),
            "should find script"
        );
    }

    #[test]
    fn search_empty_query_returns_empty() {
        let (db, _sid, pid) = db_with_minimal();
        assert!(search(&db, Some(pid), None, "", 10).unwrap().is_empty());
        assert!(search(&db, Some(pid), None, "   ", 10).unwrap().is_empty());
    }

    #[test]
    fn search_nonexistent_term_returns_empty() {
        let (db, _sid, pid) = db_with_minimal();
        let results =
            search(&db, Some(pid), None, "xyzzy_nonexistent_term_999", 10).unwrap();
        assert!(results.is_empty());
    }

    #[rstest]
    #[case("Contact", "table")]
    #[case("Hello", "script")]
    #[case("MyFunc", "custom_function")]
    fn search_finds_element_type(#[case] term: &str, #[case] expected_type: &str) {
        let (db, _sid, pid) = db_with_minimal();
        let results = search(&db, Some(pid), None, term, 10).unwrap();
        assert!(
            results.iter().any(|r| r.element_type == expected_type),
            "search '{term}' should find element_type '{expected_type}'"
        );
    }

    #[test]
    fn search_hits_table_occurrence() {
        let (db, _sid, pid) = db_with_minimal();
        let results = search(&db, Some(pid), None, "Contact", 50).unwrap();
        assert!(
            results.iter().any(|r| r.element_type == "table_occurrence"),
            "table_occurrence が検索結果に含まれること"
        );
    }

    #[test]
    fn search_hits_relationship() {
        let (db, _sid, pid) = db_with_minimal();
        let results = search(&db, Some(pid), None, "Contact", 50).unwrap();
        assert!(
            results.iter().any(|r| r.element_type == "relationship"),
            "relationship が検索結果に含まれること"
        );
    }

    #[test]
    fn search_hits_field_content() {
        let (db, _sid, pid) = db_with_minimal();
        let results = search(&db, Some(pid), None, "param1", 50).unwrap();
        assert!(
            results.iter().any(|r| r.element_type == "custom_function"),
            "カスタム関数の計算式内容でヒットすること"
        );
    }

    // ---- 全体検索（project_id=None, solution_id=None）----

    #[test]
    fn search_all_scope_finds_data() {
        let (db, _sid, _pid) = db_with_minimal();
        let results = search(&db, None, None, "Contact", 50).unwrap();
        assert!(
            results
                .iter()
                .any(|r| r.element_type == "table" && r.name == "Contact"),
            "全体スコープで Contact テーブルがヒットすること"
        );
    }

    #[test]
    fn search_all_scope_finds_across_projects() {
        let mut db = Database::open_in_memory().unwrap();
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let sid = insert_solution(&mut db, "Sol", Some("/tmp/sol.xml")).unwrap();
        let pid1 = insert_ddr_file(&mut db, &ddr, sid, Some("/tmp/a.xml")).unwrap();
        let pid2 = insert_ddr_file(&mut db, &ddr, sid, Some("/tmp/b.xml")).unwrap();

        let r1 = search(&db, Some(pid1), None, "Contact", 50).unwrap();
        let r2 = search(&db, Some(pid2), None, "Contact", 50).unwrap();
        let all = search(&db, None, None, "Contact", 200).unwrap();

        assert!(
            all.len() >= r1.len() + r2.len(),
            "全体スコープは全プロジェクットの結果の合計以上: all={}, r1={}, r2={}",
            all.len(),
            r1.len(),
            r2.len()
        );
    }

    // ---- ソリューション横断検索 ----

    #[test]
    fn solution_search_finds_across_projects() {
        let mut db = Database::open_in_memory().unwrap();
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let sid = insert_solution(&mut db, "Sol", Some("/tmp/sol.xml")).unwrap();
        let pid1 = insert_ddr_file(&mut db, &ddr, sid, Some("/tmp/a.xml")).unwrap();
        let pid2 = insert_ddr_file(&mut db, &ddr, sid, Some("/tmp/b.xml")).unwrap();

        let r1 = search(&db, Some(pid1), None, "Contact", 50).unwrap();
        let r2 = search(&db, Some(pid2), None, "Contact", 50).unwrap();
        let all = search(&db, Some(pid1), Some(sid), "Contact", 200).unwrap();
        assert!(
            all.len() >= r1.len() + r2.len(),
            "solution 横断検索: all={}, r1={}, r2={}",
            all.len(),
            r1.len(),
            r2.len()
        );
    }

    // ---- build_fts_query ----

    #[test]
    fn fts_query_single_word() {
        assert_eq!(build_fts_query("hello"), r#""hello"*"#);
    }

    #[test]
    fn fts_query_multiple_words() {
        assert_eq!(build_fts_query("hello world"), r#""hello"* "world"*"#);
    }

    #[test]
    fn fts_query_empty_string() {
        assert_eq!(build_fts_query(""), "");
        assert_eq!(build_fts_query("   "), "");
    }

    #[test]
    fn fts_query_escapes_double_quotes() {
        assert_eq!(build_fts_query(r#"say "hi""#), r#""say"* """hi"""*"#);
    }

    #[test]
    fn search_does_not_return_fields_matching_only_table_name_in_content() {
        let (db, _sid, pid) = db_with_minimal();
        let results = search(&db, Some(pid), None, "Contact", 50).unwrap();
        assert!(
            results.iter().all(|r| r.element_type != "field"),
            "フィールドは name に Contact を含まないのでヒットしてはいけない: {:?}",
            results
                .iter()
                .filter(|r| r.element_type == "field")
                .collect::<Vec<_>>()
        );
        assert!(
            results
                .iter()
                .any(|r| r.element_type == "table" && r.name == "Contact"),
            "Contact テーブル自体はヒットする"
        );
    }

    // ---- search_contains ----

    #[test]
    fn contains_search_finds_infix_match() {
        let (db, _sid, pid) = db_with_minimal();
        let results = search_contains(&db, Some(pid), None, "ntac", 50).unwrap();
        assert!(
            results
                .iter()
                .any(|r| r.element_type == "table" && r.name == "Contact"),
            "部分一致で 'Contact' テーブルがヒットすること"
        );
    }

    #[test]
    fn contains_search_all_scope_finds_infix() {
        let (db, _sid, _pid) = db_with_minimal();
        let results = search_contains(&db, None, None, "ntac", 50).unwrap();
        assert!(
            results
                .iter()
                .any(|r| r.element_type == "table" && r.name == "Contact"),
            "全体スコープの部分一致で 'Contact' テーブルがヒットすること"
        );
    }

    #[test]
    fn contains_search_no_match_returns_empty() {
        let (db, _sid, pid) = db_with_minimal();
        let results =
            search_contains(&db, Some(pid), None, "xyzzy_nonexistent_999", 50).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn prefix_search_does_not_find_infix() {
        let (db, _sid, pid) = db_with_minimal();
        let results = search(&db, Some(pid), None, "ntac", 50).unwrap();
        assert!(
            results
                .iter()
                .all(|r| !(r.element_type == "table" && r.name == "Contact")),
            "FTS5 前方一致では 'Contact' テーブルはヒットしないこと"
        );
    }

    #[test]
    fn contains_search_empty_returns_empty() {
        let (db, _sid, pid) = db_with_minimal();
        assert!(search_contains(&db, Some(pid), None, "", 50)
            .unwrap()
            .is_empty());
        assert!(search_contains(&db, Some(pid), None, "   ", 50)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn contains_solution_search_works() {
        let mut db = Database::open_in_memory().unwrap();
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let sid = insert_solution(&mut db, "Sol", Some("/tmp/sol.xml")).unwrap();
        let pid1 = insert_ddr_file(&mut db, &ddr, sid, Some("/tmp/a.xml")).unwrap();
        let pid2 = insert_ddr_file(&mut db, &ddr, sid, Some("/tmp/b.xml")).unwrap();

        let single = search_contains(&db, Some(pid1), None, "ntac", 50).unwrap();
        let cross = search_contains(&db, Some(pid1), Some(sid), "ntac", 200).unwrap();
        assert!(!single.is_empty());
        assert!(
            cross.len() >= single.len() * 2,
            "横断部分一致は単体の2倍以上ヒットすること: cross={}, single={}",
            cross.len(),
            single.len()
        );
        let _ = pid2;
    }

    #[test]
    fn contains_all_scope_finds_across_projects() {
        let mut db = Database::open_in_memory().unwrap();
        let ddr = parse_ddr(MINIMAL_XML).unwrap();
        let sid = insert_solution(&mut db, "Sol", Some("/tmp/sol.xml")).unwrap();
        let pid1 = insert_ddr_file(&mut db, &ddr, sid, Some("/tmp/a.xml")).unwrap();
        let pid2 = insert_ddr_file(&mut db, &ddr, sid, Some("/tmp/b.xml")).unwrap();

        let single = search_contains(&db, Some(pid1), None, "ntac", 50).unwrap();
        let all = search_contains(&db, None, None, "ntac", 200).unwrap();
        assert!(
            all.len() >= single.len() * 2,
            "全体スコープ部分一致は単体の2倍以上: all={}, single={}",
            all.len(),
            single.len()
        );
        let _ = pid2;
    }
}
