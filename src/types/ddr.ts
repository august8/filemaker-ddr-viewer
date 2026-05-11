// src/types/ddr.ts

export interface CommandError {
  kind: "Database" | "Parse" | "NotFound" | "Io" | "Internal";
  message: string;
}

export interface SolutionRow {
  id: number;
  name: string;
  summary_path: string | null;
  imported_at: string;
}

export interface ProjectRow {
  id: number;
  name: string;
  file_path: string | null;
  fm_version: string;
  imported_at: string;
}

export interface SolutionWithProjects {
  solution: SolutionRow;
  projects: ProjectRow[];
}

export interface ProjectSummary {
  project: ProjectRow;
  table_count: number;
  field_count: number;
  script_count: number;
  layout_count: number;
  table_occurrence_count: number;
  relationship_count: number;
  value_list_count: number;
  custom_function_count: number;
  account_count: number;
  privilege_set_count: number;
  external_data_source_count: number;
}

export interface SearchResult {
  project_id: number;
  element_type: string;
  element_id: number;
  name: string;
  snippet: string;
  rank: number;
  parent_id: number | null;
  parent_name: string | null;
}

export interface BrokenRef {
  kind: "performScript" | "scriptTrigger" | "brokenFieldRef" | "brokenLayoutRef";
  source_name: string;
  target_script_name: string;
}

export type Severity = "Info" | "Warning" | "Error";

export interface ReportIssue {
  severity: Severity;
  category: string;
  message: string;
  element_kind?: "script" | "layout";
  element_name?: string;
}

export interface ReportCard {
  issues: ReportIssue[];
  error_count: number;
  warning_count: number;
  info_count: number;
}

export interface CallChainNode {
  script_id: number;
  script_name: string;
  depth: number;
  is_cycle: boolean;
  children: CallChainNode[];
}

export interface OrphanScript {
  script_id: number;
  script_name: string;
}

export interface ProjectWithSolution {
  project_id: number;
  project_name: string;
  solution_id: number;
  solution_name: string;
  solution_imported_at: string;
}

export type DiffKind = "Added" | "Removed" | "Modified";

export interface DiffItem {
  kind: DiffKind;
  element_type: string;
  name: string;
  detail: string | null;
  project_id: number | null;
  compare_project_id: number | null;
}

export interface DiffResult {
  items: DiffItem[];
  added_count: number;
  removed_count: number;
  modified_count: number;
}

export interface TableRow {
  id: number;
  fm_id: number;
  name: string;
  field_count: number;
}

export interface FieldRow {
  id: number;
  fm_id: number;
  name: string;
  data_type: string;
  field_type: string;
  comment: string;
  is_global: boolean;
  max_repeat: number;
  calculation: string | null;
  auto_enter_type: string;
  auto_enter_calc: string | null;
  auto_enter_allow_editing: boolean;
  val_not_empty: boolean;
  val_unique: boolean;
  val_existing: boolean;
  val_max_length: number | null;
  val_value_list: string | null;
  val_calc: string | null;
  val_range_from: string | null;
  val_range_to: string | null;
  val_always: boolean;
  val_error_message: string | null;
  index_type: string;
  container_storage: string | null;
}

export interface ScriptRow {
  id: number;
  fm_id: number;
  name: string;
  run_with_full_access: boolean;
  step_count: number;
}

export interface ScriptStepRow {
  id: number;
  step_type_id: number;
  name: string;
  enabled: boolean;
  script_ref_name: string | null;
  script_ref_file: string | null;
  calculation: string | null;
  step_text: string | null;
  position: number;
}

export interface LayoutRow {
  id: number;
  fm_id: number;
  name: string;
  table_occurrence_name: string | null;
  trigger_count: number;
}

export interface TriggerRow {
  id: number;
  event: string;
  script_name: string;
  file_name: string;
}

export interface ValueListRow {
  id: number;
  fm_id: number;
  name: string;
  source: string;
  item_count: number;
}

export interface CustomFunctionRow {
  id: number;
  fm_id: number;
  name: string;
  parameters: string;
  calculation: string | null;
}

export interface LayoutObjectRow {
  id: number;
  object_type: string;
  object_key: number;
  object_name: string | null;
  button_label: string | null;
  field_table_occurrence: string | null;
  field_name: string | null;
  tooltip: string | null;
  hide_condition: string | null;
  position: number;
  bound_top: number | null;
  bound_left: number | null;
  bound_bottom: number | null;
  bound_right: number | null;
}

export interface FieldLocation {
  table_id: number;
  field_id: number;
  table_name: string;
  field_project_id: number;
}

export interface AllFieldRow {
  id: number;
  fm_id: number;
  name: string;
  data_type: string;
  field_type: string;
  comment: string;
  is_global: boolean;
  max_repeat: number;
  calculation: string | null;
  table_id: number;
  table_name: string;
}

export interface FieldCalcRef {
  field_id: number;
  field_name: string;
  table_name: string;
  table_id: number;
  project_id: number;
}

export interface FieldRefScript {
  script_id: number;
  script_name: string;
  project_id: number;
}

export interface FieldRefLayout {
  layout_id: number;
  layout_name: string;
  project_id: number;
}

export interface ConditionRow {
  id: number;
  rule_order: number;
  calculation: string;
  format_css: string;
}

export interface TableOccurrenceRow {
  id: number;
  occurrence_name: string;
  base_table_name: string;
  /** 外部ファイル参照元ファイル名。空文字列 = 自ファイル */
  source_file: string;
}

export interface PredicateRow {
  id: number;
  left_field: string;
  right_field: string;
  operator: string;
  position: number;
}

export interface RelationshipRow {
  id: number;
  fm_id: number;
  name: string;
  left_table: string;
  right_table: string;
  predicates: PredicateRow[];
}

export interface FieldRelKeyRef {
  relationship_id: number;
  relationship_name: string;
  left_table: string;
  right_table: string;
  operator: string;
  side: "left" | "right";
  project_id: number;
}

export interface UnusedFieldRow {
  table_name: string;
  field_name: string;
  field_type: string;
  data_type: string;
  field_id: number;
}

export interface AccountRow {
  id: number;
  fm_id: number;
  name: string;
  privilege_set: string | null;
  enabled: boolean;
}

export interface PrivilegeSetRow {
  id: number;
  fm_id: number;
  name: string;
  comment: string | null;
}

export interface ExternalDataSourceRow {
  id: number;
  fm_id: number;
  name: string;
  /** pathList 属性（例: "file:ExternalFile"） */
  path_list: string;
  /** link 属性 — 対応する DDR ファイル名（例: "ExternalFile_fmp12.xml"） */
  link: string;
}

// ---------------------------------------------------------------------------
// アップグレードチェック
// ---------------------------------------------------------------------------

export interface CheckItemConfig {
  id: string;
  detectionType: string;   // "step_type_id" | "step_external" | "text_match" | "field_attr"
  detectionValue: string;
}

export interface UpgradeHit {
  item_id: string;
  project_id: number;
  project_name: string;
  custom_function_name: string | null;
  script_id: number | null;
  script_name: string | null;
  step_id: number | null;
  step_name: string | null;
  step_text: string | null;
  field_id: number | null;
  field_name: string | null;
  table_id: number | null;
  table_name: string | null;
}

