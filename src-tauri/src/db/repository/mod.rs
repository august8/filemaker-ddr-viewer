//! CRUD 操作と全文検索。
//!
//! ## 主要関数
//! - [`insert_solution`] — solution を作成し ID を返す
//! - [`insert_ddr_file`] — `DdrFile` をトランザクションで一括挿入
//! - [`list_projects`] / [`delete_project`] — プロジェクト管理
//! - [`list_solutions`] / [`delete_solution`] — ソリューション管理
//! - [`search`] — FTS5 全文検索

mod catalog;
mod import;
mod search;
mod solution;

// ---------------------------------------------------------------------------
// 全 pub アイテムの再エクスポート（外部 use パスを変えない）
// ---------------------------------------------------------------------------

pub use solution::{
    delete_project, delete_solution, get_project, get_solution, get_solution_projects,
    insert_solution, list_projects, list_solutions, ProjectRow, SolutionRow, SolutionWithProjects,
};

pub use import::{insert_ddr_file, insert_layout_object_condition};

pub use catalog::{
    list_accounts, list_custom_functions, list_layout_object_conditions, list_layout_objects,
    list_layout_triggers, list_layouts, list_privilege_sets, list_relationships, list_script_steps,
    list_scripts, list_table_fields, list_table_occurrences, list_tables, list_value_list_items,
    list_value_lists, run_upgrade_check, AccountRow, CheckItemConfig, ConditionRow,
    CustomFunctionRow, FieldRow, LayoutObjectRow, LayoutRow, PredicateRow, PrivilegeSetRow,
    RelationshipRow, ScriptRow, ScriptStepRow, TableOccurrenceRow, TableRow, TriggerRow,
    UpgradeHit, ValueListRow,
};

pub use search::{search, search_contains, SearchResult};
