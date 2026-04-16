import type {
  FieldRow,
  TableOccurrenceRow,
  ScriptRow,
  LayoutRow,
  RelationshipRow,
  PredicateRow,
  TableRow,
  ValueListRow,
  CustomFunctionRow,
  ScriptStepRow,
  LayoutObjectRow,
  TriggerRow,
  ConditionRow,
  AllFieldRow,
} from "../types/ddr";

export function makeTableRow(overrides: Partial<TableRow> = {}): TableRow {
  return { id: 1, fm_id: 1, name: "TestTable", field_count: 0, ...overrides };
}

export function makeFieldRow(overrides: Partial<FieldRow> = {}): FieldRow {
  return {
    id: 1,
    fm_id: 1,
    name: "TestField",
    data_type: "Text",
    field_type: "Normal",
    comment: "",
    is_global: false,
    max_repeat: 1,
    calculation: null,
    auto_enter_type: "",
    auto_enter_calc: null,
    auto_enter_allow_editing: false,
    val_not_empty: false,
    val_unique: false,
    val_existing: false,
    val_max_length: null,
    val_value_list: null,
    val_calc: null,
    val_range_from: null,
    val_range_to: null,
    val_always: false,
    val_error_message: null,
    index_type: "",
    container_storage: null,
    ...overrides,
  };
}

export function makeScriptRow(overrides: Partial<ScriptRow> = {}): ScriptRow {
  return { id: 1, fm_id: 1, name: "TestScript", run_with_full_access: false, step_count: 0, ...overrides };
}

export function makeScriptStepRow(overrides: Partial<ScriptStepRow> = {}): ScriptStepRow {
  return {
    id: 1,
    step_type_id: 1,
    name: "TestStep",
    enabled: true,
    script_ref_name: null,
    script_ref_file: null,
    calculation: null,
    step_text: null,
    position: 0,
    ...overrides,
  };
}

export function makeLayoutRow(overrides: Partial<LayoutRow> = {}): LayoutRow {
  return { id: 1, fm_id: 1, name: "TestLayout", table_occurrence_name: null, trigger_count: 0, ...overrides };
}

export function makeTriggerRow(overrides: Partial<TriggerRow> = {}): TriggerRow {
  return { id: 1, event: "OnRecordLoad", script_name: "TestScript", file_name: "", ...overrides };
}

export function makeValueListRow(overrides: Partial<ValueListRow> = {}): ValueListRow {
  return { id: 1, fm_id: 1, name: "TestValueList", source: "Custom", item_count: 0, ...overrides };
}

export function makeCustomFunctionRow(overrides: Partial<CustomFunctionRow> = {}): CustomFunctionRow {
  return { id: 1, fm_id: 1, name: "TestFunction", parameters: "", calculation: null, ...overrides };
}

export function makeLayoutObjectRow(overrides: Partial<LayoutObjectRow> = {}): LayoutObjectRow {
  return {
    id: 1,
    object_type: "Field",
    object_key: 1,
    object_name: null,
    button_label: null,
    field_table_occurrence: null,
    field_name: null,
    tooltip: null,
    hide_condition: null,
    position: 0,
    bound_top: null,
    bound_left: null,
    bound_bottom: null,
    bound_right: null,
    ...overrides,
  };
}

export function makeTableOccurrenceRow(overrides: Partial<TableOccurrenceRow> = {}): TableOccurrenceRow {
  return { id: 1, occurrence_name: "TestOccurrence", base_table_name: "TestTable", source_file: "", ...overrides };
}

export function makePredicateRow(overrides: Partial<PredicateRow> = {}): PredicateRow {
  return { id: 1, left_field: "id", right_field: "id", operator: "=", position: 0, ...overrides };
}

export function makeRelationshipRow(overrides: Partial<RelationshipRow> = {}): RelationshipRow {
  return { id: 1, fm_id: 1, name: "TestRelationship", left_table: "Left", right_table: "Right", predicates: [], ...overrides };
}

export function makeConditionRow(overrides: Partial<ConditionRow> = {}): ConditionRow {
  return { id: 1, rule_order: 0, calculation: "", format_css: "", ...overrides };
}

export function makeAllFieldRow(overrides: Partial<AllFieldRow> = {}): AllFieldRow {
  return {
    id: 1,
    fm_id: 1,
    name: "TestField",
    data_type: "Text",
    field_type: "Normal",
    comment: "",
    is_global: false,
    max_repeat: 1,
    calculation: null,
    table_id: 1,
    table_name: "TestTable",
    ...overrides,
  };
}
