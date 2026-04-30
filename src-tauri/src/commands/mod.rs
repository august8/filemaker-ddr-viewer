//! Tauri IPC コマンド群。
//!
//! 各コマンドは `lib.rs` の `invoke_handler` に登録する。
//! フロントエンドからは `invoke('command_name', { ... })` で呼び出す。

pub mod analysis;
pub mod callchain;
pub mod catalog;
pub mod diff;
pub mod error;
pub mod field_refs;
pub mod import;
pub mod search;
#[cfg(feature = "test-utils")]
pub mod test_utils;

pub use error::CommandError;

/// `state.db` のロックを取得する共通ヘルパー。
///
/// 全コマンドで繰り返されていた
/// `state.db.lock().map_err(|e| CommandError::Internal(e.to_string()))?`
/// を一箇所に集約する。
pub(crate) fn lock_db(
    state: &crate::AppState,
) -> Result<std::sync::MutexGuard<'_, crate::db::Database>, CommandError> {
    state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))
}
