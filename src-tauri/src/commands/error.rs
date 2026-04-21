// src-tauri/src/commands/error.rs
// Task F: IPC エラー型構造化 (ADR-016)
// フロントエンドに { "kind": "Database", "message": "..." } 形式で送信する

use serde::Serialize;

/// Tauri IPC コマンドが返す構造化エラー型。
/// `#[serde(tag = "kind", content = "message")]` により
/// `{ "kind": "Database", "message": "..." }` 形式でシリアライズされる。
#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", content = "message")]
pub enum CommandError {
    #[error("データベースエラー: {0}")]
    Database(String),

    #[error("パースエラー: {0}")]
    Parse(String),

    #[error("見つかりません: {0}")]
    NotFound(String),

    #[error("IOエラー: {0}")]
    Io(String),

    #[error("内部エラー: {0}")]
    Internal(String),
}

impl From<rusqlite::Error> for CommandError {
    fn from(e: rusqlite::Error) -> Self {
        CommandError::Database(e.to_string())
    }
}

impl From<std::io::Error> for CommandError {
    fn from(e: std::io::Error) -> Self {
        CommandError::Io(e.to_string())
    }
}

impl From<String> for CommandError {
    fn from(s: String) -> Self {
        CommandError::Internal(s)
    }
}

impl From<&str> for CommandError {
    fn from(s: &str) -> Self {
        CommandError::Internal(s.to_string())
    }
}

impl From<crate::parser::ParseError> for CommandError {
    fn from(e: crate::parser::ParseError) -> Self {
        CommandError::Parse(e.to_string())
    }
}

impl From<crate::db::DbError> for CommandError {
    fn from(e: crate::db::DbError) -> Self {
        match e {
            crate::db::DbError::Sqlite(inner) => CommandError::Database(inner.to_string()),
            crate::db::DbError::NotFound(msg) => CommandError::NotFound(msg),
            crate::db::DbError::InvalidData(msg) => CommandError::Internal(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_string_maps_to_internal() {
        let err = CommandError::from("something went wrong".to_string());
        assert!(matches!(err, CommandError::Internal(_)));
        assert_eq!(err.to_string(), "内部エラー: something went wrong");
    }

    #[test]
    fn from_str_maps_to_internal() {
        let err = CommandError::from("oops");
        assert!(matches!(err, CommandError::Internal(_)));
    }

    #[test]
    fn from_io_error_maps_to_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = CommandError::from(io_err);
        assert!(matches!(err, CommandError::Io(_)));
        assert!(err.to_string().contains("IOエラー"));
    }

    #[test]
    fn from_rusqlite_error_maps_to_database() {
        let db_err = rusqlite::Error::QueryReturnedNoRows;
        let err = CommandError::from(db_err);
        assert!(matches!(err, CommandError::Database(_)));
        assert!(err.to_string().contains("データベースエラー"));
    }

    #[test]
    fn serialize_database_error_has_kind_and_message() {
        let err = CommandError::Database("table not found".to_string());
        let json = serde_json::to_string(&err).unwrap();
        // { "kind": "Database", "message": "table not found" }
        assert!(json.contains(r#""kind":"Database""#));
        assert!(json.contains(r#""message":"table not found""#));
    }

    #[test]
    fn serialize_not_found_error() {
        let err = CommandError::NotFound("script 999".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains(r#""kind":"NotFound""#));
        assert!(json.contains(r#""message":"script 999""#));
    }

    #[test]
    fn serialize_io_error() {
        let err = CommandError::Io("permission denied".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains(r#""kind":"Io""#));
    }

    #[test]
    fn serialize_internal_error() {
        let err = CommandError::Internal("lock poisoned".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains(r#""kind":"Internal""#));
    }

    #[test]
    fn display_messages_are_correct() {
        assert_eq!(
            CommandError::Database("db err".to_string()).to_string(),
            "データベースエラー: db err"
        );
        assert_eq!(
            CommandError::Parse("parse err".to_string()).to_string(),
            "パースエラー: parse err"
        );
        assert_eq!(
            CommandError::NotFound("not found".to_string()).to_string(),
            "見つかりません: not found"
        );
        assert_eq!(
            CommandError::Io("io err".to_string()).to_string(),
            "IOエラー: io err"
        );
        assert_eq!(
            CommandError::Internal("internal".to_string()).to_string(),
            "内部エラー: internal"
        );
    }
}
