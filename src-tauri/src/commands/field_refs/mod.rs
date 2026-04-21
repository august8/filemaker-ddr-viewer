//! フィールド参照解析コマンド。

mod debug;
mod helpers;
mod layout;
mod relationships;
mod script_and_calc;
mod unused_fields;

use serde::{Deserialize, Serialize};

pub use debug::get_layout_ref_debug_info;
pub use layout::{get_field_layout_refs, resolve_layout_field};
pub use relationships::get_field_relationship_keys;
pub use script_and_calc::{get_field_calc_refs, get_field_refs};
pub use unused_fields::list_unused_fields;

// Tauri の generate_handler! は関数名と同じモジュールに __cmd__* を探すため、
// サブモジュールで定義したコマンドの __cmd__* も field_refs 直下に再公開する。
pub use debug::__cmd__get_layout_ref_debug_info;
pub use layout::{__cmd__get_field_layout_refs, __cmd__resolve_layout_field};
pub use relationships::__cmd__get_field_relationship_keys;
pub use script_and_calc::{__cmd__get_field_calc_refs, __cmd__get_field_refs};
pub use unused_fields::__cmd__list_unused_fields;

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
