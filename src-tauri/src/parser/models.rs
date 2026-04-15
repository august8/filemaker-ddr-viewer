use serde::{Deserialize, Serialize};

use crate::parser::version::FmVersion;

// ---------------------------------------------------------------------------
// Newtype IDs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TableId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FieldId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScriptId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LayoutId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RelationshipId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ValueListId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CustomFunctionId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PrivilegeSetId(pub u64);

// ---------------------------------------------------------------------------
// Top-level DDR file
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdrFile {
    pub file_name: String,
    pub fm_version: FmVersion,
    pub tables: Vec<Table>,
    pub scripts: Vec<Script>,
    pub layouts: Vec<Layout>,
    pub relationships: Vec<Relationship>,
    /// リレーショングラフ内の全テーブルオカレンス（オカレンス名→ベーステーブル名マッピング）。
    pub table_occurrences: Vec<TableOccurrence>,
    pub value_lists: Vec<ValueList>,
    pub custom_functions: Vec<CustomFunction>,
    pub accounts: Vec<Account>,
    pub privilege_sets: Vec<PrivilegeSet>,
    /// ファイルオプション > スクリプトトリガーで呼ばれるスクリプト名（WindowTriggers）
    pub file_script_triggers: Vec<String>,
}

// ---------------------------------------------------------------------------
// Tables & Fields
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub id: TableId,
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub id: FieldId,
    pub name: String,
    pub data_type: DataType,
    pub field_type: FieldKind,
    pub comment: String,
    pub is_global: bool,
    pub max_repeat: u32,
    pub calculation: Option<String>,
    /// 自動入力種別。空文字 = 自動入力なし。
    /// 値例: "Calculation", "ConstantData", "Serial", "Lookup",
    /// "ModificationTimeStamp", "ModificationDate" など。
    pub auto_enter_type: String,
    /// 自動入力の計算式 / 定数値 / シリアル情報文字列（type に応じた内容）。
    pub auto_enter_calc: Option<String>,
    /// 自動入力後に編集可能かどうか。
    pub auto_enter_allow_editing: bool,
    // --- Validation ---
    pub val_not_empty: bool,
    pub val_unique: bool,
    pub val_existing: bool,
    pub val_max_length: Option<i64>,
    pub val_value_list: Option<String>,
    pub val_calc: Option<String>,
    pub val_range_from: Option<String>,
    pub val_range_to: Option<String>,
    pub val_always: bool,
    pub val_error_message: Option<String>,
    // --- Storage ---
    /// インデックス種別。"All" / "Minimal" / "None" / ""。
    pub index_type: String,
    /// コンテナフィールドの保存方法。"Internal" / "Secure" / "Open"。非コンテナは None。
    pub container_storage: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DataType {
    Text,
    Number,
    Date,
    Time,
    Timestamp,
    Container,
    Unknown(String),
}

impl DataType {
    pub fn parse_xml(s: &str) -> Self {
        match s {
            "Text" => DataType::Text,
            "Number" => DataType::Number,
            "Date" => DataType::Date,
            "Time" => DataType::Time,
            "Timestamp" => DataType::Timestamp,
            "Container" | "Binary" => DataType::Container,
            other => DataType::Unknown(other.to_owned()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FieldKind {
    Normal,
    Calculated,
    Summary,
    Unknown(String),
}

impl FieldKind {
    pub fn parse_xml(s: &str) -> Self {
        match s {
            "Normal" => FieldKind::Normal,
            "Calculated" | "CalculatedField" => FieldKind::Calculated,
            "Summary" | "SummaryField" => FieldKind::Summary,
            other => FieldKind::Unknown(other.to_owned()),
        }
    }
}

// ---------------------------------------------------------------------------
// Scripts & Steps
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: ScriptId,
    pub name: String,
    pub run_with_full_access: bool,
    pub steps: Vec<ScriptStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptStep {
    /// Script step type ID (e.g. 89 = Perform Script)
    pub step_id: u32,
    pub name: String,
    pub enabled: bool,
    pub script_ref: Option<ScriptRef>,
    pub calculation: Option<String>,
    /// 実DDR <StepText> — 人間可読な表示テキスト
    pub step_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptRef {
    pub name: String,
    pub file_name: String,
}

// ---------------------------------------------------------------------------
// Layouts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    pub id: LayoutId,
    pub name: String,
    pub table_occurrence_name: Option<String>,
    pub script_triggers: Vec<ScriptTrigger>,
    /// ボタン・オブジェクトから直接呼ばれるスクリプト名（孤立判定に使用）
    pub button_script_refs: Vec<String>,
    /// レイアウト上に配置されたフィールド参照（cross-reference 用）
    pub field_refs: Vec<LayoutFieldRef>,
    /// レイアウト上の全オブジェクト（`<Object>` 要素から収集）
    pub layout_objects: Vec<LayoutObject>,
}

/// レイアウトオブジェクトの位置・サイズ（`<Bounds>` 要素から取得）。
/// 単位は FileMaker の内部単位（ポイント相当）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bounds {
    pub top: f64,
    pub left: f64,
    pub bottom: f64,
    pub right: f64,
}

/// 条件付き書式の単一ルール（`<ConditionalFormatting><Item>` から取得）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalFormat {
    /// ルール順序（`<Item id="...">` の値）
    pub rule_order: u32,
    /// 条件式 CDATA（`<Condition><Calculation>` の内容）
    pub calculation: String,
    /// 書式 CSS 文字列（`<Format><Styles><LocalCSS>` の内容）
    pub format_css: String,
}

/// レイアウト上の単一オブジェクト（`<Object>` 要素から取得）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutObject {
    pub object_type: String,
    pub object_key: u64,
    /// `<Object name="...">` 属性で設定されたオブジェクト名（オブジェクト情報で設定）
    pub object_name: Option<String>,
    /// `<TextObj><CharacterStyleVector><Style><Data>` から取得したボタンラベルテキスト
    pub button_label: Option<String>,
    /// type="Field" のみ: `<FieldObj><DDRInfo><Field table="..."/>` のテーブルオカレンス名
    pub field_table_occurrence: Option<String>,
    /// type="Field" のみ: `<FieldObj><DDRInfo><Field name="..."/>` のフィールド名
    pub field_name: Option<String>,
    /// `<ToolTip><Calculation>` CDATA の内容
    pub tooltip: Option<String>,
    /// `<HideCondition><Calculation>` CDATA の内容
    pub hide_condition: Option<String>,
    /// `<Bounds top=... left=... bottom=... right=.../>` からの位置・サイズ情報
    pub bounds: Option<Bounds>,
    /// `<ConditionalFormatting>` から取得した条件付き書式ルール
    pub conditional_formats: Vec<ConditionalFormat>,
}

/// レイアウト上のフィールド参照 (`<FieldReference>` 要素から取得)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutFieldRef {
    /// テーブルオカレンス名 (`tableOccurrence` 属性)
    pub table_occurrence: String,
    /// フィールド名 (`field` 属性)
    pub field_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptTrigger {
    pub event: String,
    pub script_name: String,
    pub file_name: String,
}

// ---------------------------------------------------------------------------
// Table Occurrences
// ---------------------------------------------------------------------------

/// テーブルオカレンス（リレーショングラフの `<TableList>` から取得）。
/// オカレンス名からベーステーブル名へのマッピングを保持する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableOccurrence {
    /// オカレンス名 (`name` 属性)
    pub occurrence_name: String,
    /// ベーステーブル名 (`baseTable` 属性)
    pub base_table_name: String,
    /// 外部ファイル参照元ファイル名 (`<FileReference name="..."/>` から取得)。
    /// None = 自ファイルのテーブル
    pub source_file: Option<String>,
}

// ---------------------------------------------------------------------------
// Relationships
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: RelationshipId,
    pub name: String,
    pub left_table: String,
    pub right_table: String,
    pub predicates: Vec<JoinPredicate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinPredicate {
    pub left_field: String,
    pub right_field: String,
    /// e.g. "Equal", "NotEqual", "Less", "Greater", etc.
    pub operator: String,
}

// ---------------------------------------------------------------------------
// Value Lists
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueList {
    pub id: ValueListId,
    pub name: String,
    pub source: ValueListSource,
    pub custom_values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ValueListSource {
    Custom,
    Field,
    Unknown(String),
}

impl ValueListSource {
    pub fn parse_xml(s: &str) -> Self {
        match s {
            "Custom" => ValueListSource::Custom,
            "Field" => ValueListSource::Field,
            other => ValueListSource::Unknown(other.to_owned()),
        }
    }
}

// ---------------------------------------------------------------------------
// Custom Functions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFunction {
    pub id: CustomFunctionId,
    pub name: String,
    pub parameters: Vec<String>,
    pub calculation: Option<String>,
}

// ---------------------------------------------------------------------------
// Accounts & Privileges
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: AccountId,
    pub name: String,
    pub privilege_set: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivilegeSet {
    pub id: PrivilegeSetId,
    pub name: String,
    pub comment: Option<String>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_type_parse_xml_known() {
        assert_eq!(DataType::parse_xml("Text"), DataType::Text);
        assert_eq!(DataType::parse_xml("Number"), DataType::Number);
        assert_eq!(DataType::parse_xml("Date"), DataType::Date);
        assert_eq!(DataType::parse_xml("Time"), DataType::Time);
        assert_eq!(DataType::parse_xml("Timestamp"), DataType::Timestamp);
        assert_eq!(DataType::parse_xml("Container"), DataType::Container);
    }

    #[test]
    fn data_type_parse_xml_unknown() {
        assert_eq!(
            DataType::parse_xml("Blob"),
            DataType::Unknown("Blob".to_owned())
        );
    }

    #[test]
    fn field_kind_parse_xml_variants() {
        assert_eq!(FieldKind::parse_xml("Normal"), FieldKind::Normal);
        assert_eq!(FieldKind::parse_xml("Calculated"), FieldKind::Calculated);
        assert_eq!(
            FieldKind::parse_xml("CalculatedField"),
            FieldKind::Calculated
        );
        assert_eq!(FieldKind::parse_xml("Summary"), FieldKind::Summary);
        assert_eq!(FieldKind::parse_xml("SummaryField"), FieldKind::Summary);
        assert_eq!(
            FieldKind::parse_xml("Foo"),
            FieldKind::Unknown("Foo".to_owned())
        );
    }

    #[test]
    fn value_list_source_parse_xml() {
        assert_eq!(
            ValueListSource::parse_xml("Custom"),
            ValueListSource::Custom
        );
        assert_eq!(ValueListSource::parse_xml("Field"), ValueListSource::Field);
        assert_eq!(
            ValueListSource::parse_xml("Other"),
            ValueListSource::Unknown("Other".to_owned())
        );
    }

    #[test]
    fn newtype_ids_eq() {
        assert_eq!(TableId(1), TableId(1));
        assert_ne!(TableId(1), TableId(2));
        assert_eq!(ScriptId(42), ScriptId(42));
    }
}
