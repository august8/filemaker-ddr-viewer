use rusqlite::{params, OptionalExtension as _};

use crate::{commands::CommandError, AppState};

use super::{FieldLocation, FieldRefLayout};

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
    let db = crate::commands::lock_db(&state)?;
    get_field_layout_refs_inner(&db.conn, project_id, &table_name, &field_name)
        .map_err(CommandError::from)
}

/// テーブルオカレンス名とフィールド名からフィールドの DB ID・テーブル DB ID・ベーステーブル名を解決する。
#[tauri::command]
pub async fn resolve_layout_field(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    occurrence_name: String,
    field_name: String,
) -> Result<Option<FieldLocation>, CommandError> {
    let db = crate::commands::lock_db(&state)?;
    resolve_layout_field_inner(&db.conn, project_id, &occurrence_name, &field_name)
        .map_err(CommandError::from)
}

// ---------------------------------------------------------------------------
// inner 関数（テスト可能・Tauri State 非依存）
// ---------------------------------------------------------------------------

/// `get_field_layout_refs` の内部実装（ソリューション全体スコープ）。
fn get_field_layout_refs_inner(
    conn: &rusqlite::Connection,
    project_id: i64,
    table_name: &str,
    field_name: &str,
) -> Result<Vec<FieldRefLayout>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT l.id, l.name, l.project_id
         FROM layouts l
         JOIN table_occurrences toc_main
           ON toc_main.project_id IN (
             SELECT id FROM projects
             WHERE solution_id = (SELECT solution_id FROM projects WHERE id = ?1)
           )
          AND toc_main.occurrence_name = l.table_occurrence_name
          AND toc_main.base_table_name = ?2
         JOIN layout_field_refs lfr
           ON lfr.layout_id = l.id
          AND lfr.field_name = ?3
         JOIN table_occurrences toc_field
           ON toc_field.project_id IN (
             SELECT id FROM projects
             WHERE solution_id = (SELECT solution_id FROM projects WHERE id = ?1)
           )
          AND toc_field.occurrence_name = lfr.table_occurrence
          AND toc_field.base_table_name = ?2
         WHERE l.project_id IN (
           SELECT id FROM projects
           WHERE solution_id = (SELECT solution_id FROM projects WHERE id = ?1)
         )
         ORDER BY l.position, l.name",
    )?;
    let rows = stmt
        .query_map(params![project_id, table_name, field_name], |row| {
            Ok(FieldRefLayout {
                layout_id: row.get(0)?,
                layout_name: row.get(1)?,
                project_id: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// `resolve_layout_field` の内部実装。
///
/// Step1: 同一プロジェクト内で検索。見つかれば `field_project_id = project_id` で返す。
/// Step2: 見つからない場合、`table_occurrences.source_file` から外部プロジェクトを特定して検索。
fn resolve_layout_field_inner(
    conn: &rusqlite::Connection,
    project_id: i64,
    occurrence_name: &str,
    field_name: &str,
) -> Result<Option<FieldLocation>, rusqlite::Error> {
    // Step1: 同一プロジェクト内検索
    let same_project = conn
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
                    field_project_id: project_id,
                })
            },
        )
        .optional()?;

    if same_project.is_some() {
        return Ok(same_project);
    }

    // Step2: source_file 経由で外部プロジェクトを検索
    let ext_info: Option<(String, String)> = conn
        .query_row(
            "SELECT source_file, base_table_name
           FROM table_occurrences
          WHERE project_id = ?1 AND occurrence_name = ?2
            AND source_file IS NOT NULL AND source_file != ''
          LIMIT 1",
            params![project_id, occurrence_name],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;

    let (source_file, base_table_name) = match ext_info {
        Some(v) => v,
        None => return Ok(None),
    };

    let ext_project_id: Option<i64> = conn
        .query_row(
            "SELECT ext.id FROM projects ext
          JOIN projects cur ON cur.solution_id = ext.solution_id
         WHERE cur.id = ?1 AND ext.name = ?2
         LIMIT 1",
            params![project_id, source_file],
            |row| row.get(0),
        )
        .optional()?;

    let ext_project_id = match ext_project_id {
        Some(id) => id,
        None => return Ok(None),
    };

    conn.query_row(
        "SELECT bt.id, f.id, bt.name
           FROM fields f
           JOIN base_tables bt ON bt.id = f.table_id
          WHERE bt.project_id = ?1
            AND bt.name = ?2
            AND f.name = ?3
          LIMIT 1",
        params![ext_project_id, base_table_name, field_name],
        |row| {
            Ok(FieldLocation {
                table_id: row.get(0)?,
                field_id: row.get(1)?,
                table_name: row.get(2)?,
                field_project_id: ext_project_id,
            })
        },
    )
    .optional()
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::super::helpers::test_helpers::{setup, setup_cross_project, setup_with_layout_refs};
    use super::*;

    #[test]
    fn layout_refs_returns_layout_containing_field() {
        let (conn, project_id) = setup_with_layout_refs();
        let refs = get_field_layout_refs_inner(&conn, project_id, "Invoice", "Amount").unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].layout_name, "InvoiceList");
    }

    #[test]
    fn layout_refs_returns_layout_via_alias_occurrence() {
        let (conn, project_id) = setup_with_layout_refs();
        // InvoiceAlias 経由で配置された Total フィールド
        let refs = get_field_layout_refs_inner(&conn, project_id, "Invoice", "Total").unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].layout_name, "InvoiceList");
    }

    #[test]
    fn layout_refs_returns_empty_for_different_table_field() {
        let (conn, project_id) = setup_with_layout_refs();
        // Order テーブルのフィールドはこのレイアウトに配置されていない
        let refs = get_field_layout_refs_inner(&conn, project_id, "Order", "Total").unwrap();
        assert!(refs.is_empty());
    }

    #[test]
    fn resolve_layout_field_returns_location_for_existing_field() {
        let (conn, project_id) = setup();
        let loc = resolve_layout_field_inner(&conn, project_id, "Invoice", "Amount").unwrap();
        assert!(loc.is_some());
        let loc = loc.unwrap();
        assert_eq!(loc.table_name, "Invoice");
    }

    #[test]
    fn resolve_layout_field_returns_location_via_alias() {
        let (conn, project_id) = setup();
        // InvoiceAlias も Invoice ベーステーブルを指すので解決できる
        let loc = resolve_layout_field_inner(&conn, project_id, "InvoiceAlias", "Amount").unwrap();
        assert!(loc.is_some());
        let loc = loc.unwrap();
        assert_eq!(loc.table_name, "Invoice");
    }

    #[test]
    fn resolve_layout_field_returns_none_for_unknown_field() {
        let (conn, project_id) = setup();
        let loc = resolve_layout_field_inner(&conn, project_id, "Invoice", "NonExistent").unwrap();
        assert!(loc.is_none());
    }

    #[test]
    fn resolve_layout_field_returns_none_for_unknown_occurrence() {
        let (conn, project_id) = setup();
        let loc = resolve_layout_field_inner(&conn, project_id, "UnknownOcc", "Amount").unwrap();
        assert!(loc.is_none());
    }

    #[test]
    fn resolve_layout_field_returns_external_field_via_cross_project_lookup() {
        let (conn, program_project_id, data_project_id) = setup_cross_project();
        // プログラムファイルのオカレンス "Customers" 経由でデータファイルのフィールドを解決
        let loc = resolve_layout_field_inner(&conn, program_project_id, "Customers", "FirstName")
            .unwrap();
        assert!(loc.is_some(), "外部フィールドを解決できるべき");
        let loc = loc.unwrap();
        assert_eq!(loc.table_name, "Customer");
        assert_eq!(loc.field_project_id, data_project_id);
    }

    #[test]
    fn resolve_layout_field_returns_none_if_external_project_not_imported() {
        use crate::db::schema::initialize;
        use rusqlite::Connection;
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

        // 外部ファイルを参照しているが、そのファイルはインポートされていない
        conn.execute(
            "INSERT INTO table_occurrences(project_id, occurrence_name, base_table_name, source_file)
             VALUES(?1, 'Customers', 'Customer', 'DataFile')",
            [program_project_id],
        )
        .unwrap();

        let loc = resolve_layout_field_inner(&conn, program_project_id, "Customers", "FirstName")
            .unwrap();
        assert!(
            loc.is_none(),
            "外部プロジェクト未インポート時は None を返すべき"
        );
    }
}
