//! SQLite データ層。
//!
//! ## 使い方
//! ```rust,no_run
//! use filemaker_ddr_viewer_lib::db::Database;
//!
//! let mut db = Database::open_in_memory().expect("DB open");
//! ```
//!
//! ## モジュール構成
//! - [`schema`] — テーブル定義・マイグレーション
//! - [`repository`] — CRUD 操作・全文検索

pub mod repository;
pub mod schema;

use rusqlite::Connection;

// ---------------------------------------------------------------------------
// エラー型
// ---------------------------------------------------------------------------

/// DB操作で発生するエラー。
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Record not found: {0}")]
    NotFound(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),
}

// ---------------------------------------------------------------------------
// Database 構造体
// ---------------------------------------------------------------------------

/// SQLite 接続のラッパー。スキーマ初期化済みの状態で返す。
pub struct Database {
    pub(crate) conn: Connection,
}

impl Database {
    /// インメモリ DB を開く（テスト用）。
    pub fn open_in_memory() -> Result<Self, DbError> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        schema::initialize(&db.conn)?;
        Ok(db)
    }

    /// ファイルパスを指定して DB を開く。
    pub fn open(path: &str) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        schema::initialize(&db.conn)?;
        Ok(db)
    }
}

// ---------------------------------------------------------------------------
// 公開型の再エクスポート
// ---------------------------------------------------------------------------

pub use repository::{
    list_custom_functions, list_layout_objects, list_layout_triggers, list_layouts,
    list_script_steps, list_scripts, list_table_fields, list_tables, list_value_list_items,
    list_value_lists, CustomFunctionRow, FieldRow, LayoutObjectRow, LayoutRow, ProjectRow,
    ScriptRow, ScriptStepRow, SearchResult, SolutionRow, SolutionWithProjects, TableRow,
    TriggerRow, ValueListRow,
};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_in_memory_succeeds() {
        Database::open_in_memory().expect("should open in-memory DB");
    }

    #[test]
    fn db_error_display_sqlite() {
        let e = DbError::NotFound("script:99".to_owned());
        assert!(e.to_string().contains("99"));
    }
}
