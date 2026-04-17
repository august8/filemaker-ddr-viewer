//! SQLite スキーマ定義とマイグレーション。
//!
//! `initialize()` を呼ぶと、まだ存在しないテーブルを全て作成し
//! `schema_version` を更新する。

use rusqlite::Connection;

use crate::db::DbError;

/// 現在のスキーマバージョン。
pub const CURRENT_SCHEMA_VERSION: i32 = 15;

// ---------------------------------------------------------------------------
// 公開 API
// ---------------------------------------------------------------------------

/// 全テーブルを作成し、スキーマバージョンを設定する。
/// 既に存在するテーブルはスキップする（`IF NOT EXISTS`）。
pub fn initialize(conn: &Connection) -> Result<(), DbError> {
    // WAL モードで高速化
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

    conn.execute_batch(DDL_CORE)?;
    conn.execute_batch(DDL_FTS)?;

    // スキーマバージョンが未設定の場合のみ挿入
    conn.execute(
        "INSERT OR IGNORE INTO schema_version(version) VALUES(?1)",
        [CURRENT_SCHEMA_VERSION],
    )?;

    // カラム追加マイグレーション（ALTER TABLE は既存カラムがあってもエラーにならない）
    migrate(conn)?;

    Ok(())
}

/// 既存 DB に対してカラム追加等のマイグレーションを適用する。
/// `ALTER TABLE ... ADD COLUMN` は列が既に存在する場合はエラーになるため、
/// 事前にカラム有無をチェックしてから実行する。
fn migrate(conn: &Connection) -> Result<(), DbError> {
    // v3: script_steps に step_text カラムを追加
    let has_step_text: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('script_steps') WHERE name='step_text'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        != 0;

    if !has_step_text {
        conn.execute_batch("ALTER TABLE script_steps ADD COLUMN step_text TEXT;")?;
    }

    // v4: scripts / layouts に position カラムを追加
    let has_script_position: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('scripts') WHERE name='position'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        != 0;
    if !has_script_position {
        conn.execute_batch("ALTER TABLE scripts ADD COLUMN position INTEGER NOT NULL DEFAULT 0;")?;
    }

    let has_layout_position: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('layouts') WHERE name='position'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        != 0;
    if !has_layout_position {
        conn.execute_batch("ALTER TABLE layouts ADD COLUMN position INTEGER NOT NULL DEFAULT 0;")?;
    }

    // v6: layout_objects テーブルを追加（既存DBのみ）
    let has_layout_objects: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='layout_objects'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        != 0;
    if !has_layout_objects {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS layout_objects (
                id                     INTEGER PRIMARY KEY,
                layout_id              INTEGER NOT NULL REFERENCES layouts(id) ON DELETE CASCADE,
                object_type            TEXT    NOT NULL,
                object_key             INTEGER NOT NULL,
                object_name            TEXT,
                field_table_occurrence TEXT,
                field_name             TEXT,
                tooltip                TEXT,
                hide_condition         TEXT,
                position               INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_layout_objects_layout ON layout_objects(layout_id);",
        )?;
    }

    // v7: layout_objects に object_name カラムを追加（既存テーブルがある場合）
    let has_object_name: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('layout_objects') WHERE name='object_name'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        != 0;
    if !has_object_name {
        conn.execute_batch("ALTER TABLE layout_objects ADD COLUMN object_name TEXT;")?;
    }

    // v8: layout_objects に button_label カラムを追加
    let has_button_label: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('layout_objects') WHERE name='button_label'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        != 0;
    if !has_button_label {
        conn.execute_batch("ALTER TABLE layout_objects ADD COLUMN button_label TEXT;")?;
    }

    // v9: layout_objects に位置・サイズ情報カラムを追加
    let has_bound_top: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('layout_objects') WHERE name='bound_top'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        != 0;
    if !has_bound_top {
        conn.execute_batch(
            "ALTER TABLE layout_objects ADD COLUMN bound_top    REAL;
             ALTER TABLE layout_objects ADD COLUMN bound_left   REAL;
             ALTER TABLE layout_objects ADD COLUMN bound_bottom REAL;
             ALTER TABLE layout_objects ADD COLUMN bound_right  REAL;",
        )?;
    }

    // v10: layout_object_conditions テーブルを追加（条件付き書式）
    let has_conditions_table: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='layout_object_conditions'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        != 0;
    if !has_conditions_table {
        conn.execute_batch(
            "CREATE TABLE layout_object_conditions (
                id         INTEGER PRIMARY KEY,
                object_id  INTEGER NOT NULL REFERENCES layout_objects(id) ON DELETE CASCADE,
                rule_order INTEGER NOT NULL,
                calculation TEXT NOT NULL,
                format_css  TEXT NOT NULL
            );",
        )?;
    }

    // v13: fields に Validation / index_type カラムを追加
    let has_val_not_empty: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('fields') WHERE name='val_not_empty'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        != 0;
    if !has_val_not_empty {
        conn.execute_batch(
            "ALTER TABLE fields ADD COLUMN val_not_empty    INTEGER NOT NULL DEFAULT 0;
             ALTER TABLE fields ADD COLUMN val_unique        INTEGER NOT NULL DEFAULT 0;
             ALTER TABLE fields ADD COLUMN val_existing      INTEGER NOT NULL DEFAULT 0;
             ALTER TABLE fields ADD COLUMN val_max_length    INTEGER;
             ALTER TABLE fields ADD COLUMN val_value_list    TEXT;
             ALTER TABLE fields ADD COLUMN val_calc          TEXT;
             ALTER TABLE fields ADD COLUMN val_range_from    TEXT;
             ALTER TABLE fields ADD COLUMN val_range_to      TEXT;
             ALTER TABLE fields ADD COLUMN val_always        INTEGER NOT NULL DEFAULT 0;
             ALTER TABLE fields ADD COLUMN val_error_message TEXT;
             ALTER TABLE fields ADD COLUMN index_type        TEXT NOT NULL DEFAULT '';",
        )?;
    }

    // v12: fields に auto_enter 関連カラムを追加
    let has_auto_enter_type: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('fields') WHERE name='auto_enter_type'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        != 0;
    if !has_auto_enter_type {
        conn.execute_batch(
            "ALTER TABLE fields ADD COLUMN auto_enter_type          TEXT    NOT NULL DEFAULT '';
             ALTER TABLE fields ADD COLUMN auto_enter_calc          TEXT;
             ALTER TABLE fields ADD COLUMN auto_enter_allow_editing INTEGER NOT NULL DEFAULT 1;",
        )?;
    }

    // v11: table_occurrences に source_file カラムを追加（外部ファイル参照）
    let has_source_file: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('table_occurrences') WHERE name='source_file'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        != 0;
    if !has_source_file {
        conn.execute_batch(
            "ALTER TABLE table_occurrences ADD COLUMN source_file TEXT NOT NULL DEFAULT '';",
        )?;
    }

    // v14: fields に container_storage カラムを追加
    let has_container_storage: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('fields') WHERE name='container_storage'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        != 0;
    if !has_container_storage {
        conn.execute_batch("ALTER TABLE fields ADD COLUMN container_storage TEXT;")?;
    }

    Ok(())
}

/// 現在のスキーマバージョンを返す。
pub fn schema_version(conn: &Connection) -> Result<i32, DbError> {
    let v: i32 = conn.query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
        row.get(0)
    })?;
    Ok(v)
}

// ---------------------------------------------------------------------------
// DDL — コアテーブル
// ---------------------------------------------------------------------------

const DDL_CORE: &str = r#"
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER NOT NULL
);

-- 1 インポート操作 = 1 solution（概要.xml 起点）
CREATE TABLE IF NOT EXISTS solutions (
    id           INTEGER PRIMARY KEY,
    name         TEXT    NOT NULL,
    summary_path TEXT,
    imported_at  TEXT    NOT NULL DEFAULT (datetime('now', 'localtime'))
);

-- 1 solution = 複数 project（各 DBファイル）
CREATE TABLE IF NOT EXISTS projects (
    id          INTEGER PRIMARY KEY,
    solution_id INTEGER NOT NULL REFERENCES solutions(id) ON DELETE CASCADE,
    name        TEXT    NOT NULL,
    file_path   TEXT,
    fm_version  TEXT    NOT NULL,
    imported_at TEXT    NOT NULL DEFAULT (datetime('now', 'localtime'))
);
CREATE INDEX IF NOT EXISTS idx_projects_solution ON projects(solution_id);

-- ベーステーブル
CREATE TABLE IF NOT EXISTS base_tables (
    id         INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    fm_id      INTEGER NOT NULL,
    name       TEXT    NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_base_tables_project ON base_tables(project_id);

-- フィールド
CREATE TABLE IF NOT EXISTS fields (
    id         INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    table_id   INTEGER NOT NULL REFERENCES base_tables(id) ON DELETE CASCADE,
    fm_id      INTEGER NOT NULL,
    name       TEXT    NOT NULL,
    data_type  TEXT    NOT NULL,
    field_type TEXT    NOT NULL,
    comment    TEXT    NOT NULL DEFAULT '',
    is_global  INTEGER NOT NULL DEFAULT 0,
    max_repeat INTEGER NOT NULL DEFAULT 1,
    calculation TEXT
);
CREATE INDEX IF NOT EXISTS idx_fields_project ON fields(project_id);
CREATE INDEX IF NOT EXISTS idx_fields_table   ON fields(table_id);

-- スクリプト
CREATE TABLE IF NOT EXISTS scripts (
    id                   INTEGER PRIMARY KEY,
    project_id           INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    fm_id                INTEGER NOT NULL,
    name                 TEXT    NOT NULL,
    run_with_full_access INTEGER NOT NULL DEFAULT 0,
    position             INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_scripts_project ON scripts(project_id);

-- スクリプトステップ
CREATE TABLE IF NOT EXISTS script_steps (
    id              INTEGER PRIMARY KEY,
    script_id       INTEGER NOT NULL REFERENCES scripts(id) ON DELETE CASCADE,
    step_type_id    INTEGER NOT NULL,
    name            TEXT    NOT NULL,
    enabled         INTEGER NOT NULL DEFAULT 1,
    script_ref_name TEXT,
    script_ref_file TEXT,
    calculation     TEXT,
    step_text       TEXT,
    position        INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_script_steps_script ON script_steps(script_id);

-- レイアウト
CREATE TABLE IF NOT EXISTS layouts (
    id                    INTEGER PRIMARY KEY,
    project_id            INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    fm_id                 INTEGER NOT NULL,
    name                  TEXT    NOT NULL,
    table_occurrence_name TEXT,
    position              INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_layouts_project ON layouts(project_id);

-- スクリプトトリガー
CREATE TABLE IF NOT EXISTS script_triggers (
    id          INTEGER PRIMARY KEY,
    layout_id   INTEGER NOT NULL REFERENCES layouts(id) ON DELETE CASCADE,
    event       TEXT    NOT NULL,
    script_name TEXT    NOT NULL,
    file_name   TEXT    NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS idx_script_triggers_layout ON script_triggers(layout_id);

-- リレーション
CREATE TABLE IF NOT EXISTS relationships (
    id          INTEGER PRIMARY KEY,
    project_id  INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    fm_id       INTEGER NOT NULL,
    name        TEXT    NOT NULL,
    left_table  TEXT    NOT NULL,
    right_table TEXT    NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_relationships_project ON relationships(project_id);

-- 結合条件
CREATE TABLE IF NOT EXISTS join_predicates (
    id              INTEGER PRIMARY KEY,
    relationship_id INTEGER NOT NULL REFERENCES relationships(id) ON DELETE CASCADE,
    left_field      TEXT    NOT NULL,
    right_field     TEXT    NOT NULL,
    operator        TEXT    NOT NULL,
    position        INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_join_predicates_rel ON join_predicates(relationship_id);

-- バリューリスト
CREATE TABLE IF NOT EXISTS value_lists (
    id         INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    fm_id      INTEGER NOT NULL,
    name       TEXT    NOT NULL,
    source     TEXT    NOT NULL DEFAULT 'Custom'
);
CREATE INDEX IF NOT EXISTS idx_value_lists_project ON value_lists(project_id);

-- バリューリストの値
CREATE TABLE IF NOT EXISTS value_list_items (
    id            INTEGER PRIMARY KEY,
    value_list_id INTEGER NOT NULL REFERENCES value_lists(id) ON DELETE CASCADE,
    value         TEXT    NOT NULL,
    position      INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_value_list_items_vl ON value_list_items(value_list_id);

-- バリューリストのフィールド参照（<PrimaryField> / <SecondaryField>）
CREATE TABLE IF NOT EXISTS value_list_field_refs (
    id            INTEGER PRIMARY KEY,
    value_list_id INTEGER NOT NULL REFERENCES value_lists(id) ON DELETE CASCADE,
    table_occurrence TEXT NOT NULL,
    field_name    TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_value_list_field_refs_vl ON value_list_field_refs(value_list_id);

-- カスタム関数
CREATE TABLE IF NOT EXISTS custom_functions (
    id          INTEGER PRIMARY KEY,
    project_id  INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    fm_id       INTEGER NOT NULL,
    name        TEXT    NOT NULL,
    parameters  TEXT    NOT NULL DEFAULT '',
    calculation TEXT
);
CREATE INDEX IF NOT EXISTS idx_custom_functions_project ON custom_functions(project_id);

-- テーブルオカレンス（リレーショングラフの TableList から取得）
CREATE TABLE IF NOT EXISTS table_occurrences (
    id               INTEGER PRIMARY KEY,
    project_id       INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    occurrence_name  TEXT    NOT NULL,
    base_table_name  TEXT    NOT NULL,
    source_file      TEXT    NOT NULL DEFAULT '',
    UNIQUE (project_id, occurrence_name)
);
CREATE INDEX IF NOT EXISTS idx_table_occurrences_project ON table_occurrences(project_id);

-- レイアウト上のフィールド参照（ObjectList > Object > FieldReference から取得）
CREATE TABLE IF NOT EXISTS layout_field_refs (
    id               INTEGER PRIMARY KEY,
    layout_id        INTEGER NOT NULL REFERENCES layouts(id) ON DELETE CASCADE,
    table_occurrence TEXT    NOT NULL,
    field_name       TEXT    NOT NULL,
    UNIQUE (layout_id, table_occurrence, field_name)
);
CREATE INDEX IF NOT EXISTS idx_layout_field_refs_layout ON layout_field_refs(layout_id);
CREATE INDEX IF NOT EXISTS idx_layout_field_refs_field  ON layout_field_refs(field_name);
CREATE INDEX IF NOT EXISTS idx_layout_field_refs_toc    ON layout_field_refs(table_occurrence);

-- レイアウト上のオブジェクト（Object 要素から収集）
CREATE TABLE IF NOT EXISTS layout_objects (
    id                     INTEGER PRIMARY KEY,
    layout_id              INTEGER NOT NULL REFERENCES layouts(id) ON DELETE CASCADE,
    object_type            TEXT    NOT NULL,
    object_key             INTEGER NOT NULL,
    object_name            TEXT,
    button_label           TEXT,
    field_table_occurrence TEXT,
    field_name             TEXT,
    tooltip                TEXT,
    hide_condition         TEXT,
    position               INTEGER NOT NULL DEFAULT 0,
    bound_top              REAL,
    bound_left             REAL,
    bound_bottom           REAL,
    bound_right            REAL
);
CREATE INDEX IF NOT EXISTS idx_layout_objects_layout ON layout_objects(layout_id);

-- アカウント
CREATE TABLE IF NOT EXISTS accounts (
    id            INTEGER PRIMARY KEY,
    project_id    INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    fm_id         INTEGER NOT NULL,
    name          TEXT    NOT NULL,
    privilege_set TEXT,
    enabled       INTEGER NOT NULL DEFAULT 1
);

-- 権限セット
CREATE TABLE IF NOT EXISTS privilege_sets (
    id         INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    fm_id      INTEGER NOT NULL,
    name       TEXT    NOT NULL,
    comment    TEXT
);
"#;

// ---------------------------------------------------------------------------
// DDL — FTS5 全文検索インデックス
// ---------------------------------------------------------------------------

const DDL_FTS: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS search_index USING fts5(
    project_id  UNINDEXED,
    element_type,
    element_id  UNINDEXED,
    name,
    content,
    tokenize = 'unicode61'
);
"#;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn in_memory() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn initialize_idempotent() {
        let conn = in_memory();
        // 2回呼んでもエラーにならない
        initialize(&conn).unwrap();
    }

    #[test]
    fn schema_version_is_correct() {
        let conn = in_memory();
        assert_eq!(schema_version(&conn).unwrap(), CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn all_core_tables_exist() {
        let conn = in_memory();
        let tables = [
            "solutions",
            "projects",
            "base_tables",
            "fields",
            "scripts",
            "script_steps",
            "layouts",
            "script_triggers",
            "table_occurrences",
            "layout_field_refs",
            "relationships",
            "join_predicates",
            "value_lists",
            "value_list_items",
            "custom_functions",
            "accounts",
            "privilege_sets",
            "layout_objects",
        ];
        for table in &tables {
            let n: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            assert_eq!(n, 1, "table '{table}' not found");
        }
    }

    #[test]
    fn fts5_table_exists() {
        let conn = in_memory();
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='search_index'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(n, 1, "search_index FTS5 table not found");
    }

    #[test]
    fn foreign_key_pragma_on() {
        let conn = in_memory();
        let v: i32 = conn
            .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, 1);
    }
}
