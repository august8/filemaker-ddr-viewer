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

pub use error::CommandError;
