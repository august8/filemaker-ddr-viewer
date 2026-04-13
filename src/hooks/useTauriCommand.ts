// src/hooks/useTauriCommand.ts
import { invoke } from "@tauri-apps/api/core";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import type {
  SolutionRow,
  SolutionWithProjects,
  ProjectRow,
  ProjectSummary,
  SearchResult,
  BrokenRef,
  ReportCard,
  CallChainNode,
  OrphanScript,
  DiffResult,
  ProjectWithSolution,
  TableRow,
  FieldRow,
  AllFieldRow,
  ScriptRow,
  ScriptStepRow,
  LayoutRow,
  LayoutObjectRow,
  TriggerRow,
  ValueListRow,
  CustomFunctionRow,
  FieldLocation,
  FieldRefScript,
  FieldRefLayout,
  FieldRelKeyRef,
  UnusedFieldRow,
  ConditionRow,
  TableOccurrenceRow,
  RelationshipRow,
  AccountRow,
  PrivilegeSetRow,
  UpgradeHit,
} from "../types/ddr";
import type { CheckItem } from "../stores/appStore";

// ソリューション一覧
export function useSolutions() {
  return useQuery({
    queryKey: ["solutions"],
    queryFn: () => invoke<SolutionRow[]>("list_solutions"),
  });
}

// ソリューション内のプロジェクト一覧
export function useSolutionProjects(solutionId: number | null) {
  return useQuery({
    queryKey: ["solution_projects", solutionId],
    queryFn: () =>
      invoke<ProjectRow[]>("get_solution_projects", { solutionId }),
    enabled: solutionId !== null,
  });
}

// ソリューションインポート
export function useImportSolution() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (summaryPath: string) =>
      invoke<SolutionWithProjects>("import_solution", { summaryPath }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["solutions"] });
      qc.invalidateQueries({ queryKey: ["all_projects"] });
    },
  });
}

// ソリューション削除
export function useDeleteSolution() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (solutionId: number) =>
      invoke<void>("delete_solution", { solutionId }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["solutions"] });
      qc.invalidateQueries({ queryKey: ["all_projects"] });
    },
  });
}

// プロジェクト削除
export function useDeleteProject() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (projectId: number) =>
      invoke<void>("delete_project", { projectId }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["solutions"] });
      qc.invalidateQueries({ queryKey: ["all_projects"] });
    },
  });
}

// プロジェクトサマリー
export function useProjectSummary(projectId: number | null) {
  return useQuery({
    queryKey: ["project_summary", projectId],
    queryFn: () => invoke<ProjectSummary>("get_project_summary", { projectId }),
    enabled: projectId !== null,
  });
}

// 検索
export function useSearch(
  projectId: number | null,
  query: string,
  contains: boolean = false,
  scope: "all" | "solution" | "project" = "all",
  solutionId: number | null = null,
) {
  const effectiveProjectId = scope === "project" ? projectId : null;
  const effectiveSolutionId = scope === "solution" ? solutionId : null;
  return useQuery({
    queryKey: ["search", query, contains, scope, effectiveProjectId, effectiveSolutionId],
    queryFn: () =>
      invoke<SearchResult[]>("search_elements", {
        projectId: effectiveProjectId,
        solutionId: effectiveSolutionId,
        query,
        contains: contains || null,
        // limit 省略 → Rust 側が None として全件返却（SQLite LIMIT -1）
      }),
    enabled: query.trim().length > 0,
  });
}

// 壊れた参照
export function useBrokenRefs(projectId: number | null) {
  return useQuery({
    queryKey: ["broken_refs", projectId],
    queryFn: () => invoke<BrokenRef[]>("get_broken_refs", { projectId }),
    enabled: projectId !== null,
  });
}

// レポートカード
export function useReportCard(projectId: number | null) {
  return useQuery({
    queryKey: ["report_card", projectId],
    queryFn: () => invoke<ReportCard>("get_report_card", { projectId }),
    enabled: projectId !== null,
  });
}

// コールチェーン
export function useCallChain(projectId: number | null, scriptId: number | null) {
  return useQuery({
    queryKey: ["call_chain", projectId, scriptId],
    queryFn: () =>
      invoke<CallChainNode>("get_call_chain", { projectId, scriptId }),
    enabled: projectId !== null && scriptId !== null,
  });
}

// 呼び出し元スクリプト ID 一覧
export function useCallers(projectId: number | null, scriptId: number | null) {
  return useQuery({
    queryKey: ["callers", projectId, scriptId],
    queryFn: () =>
      invoke<number[]>("get_callers", { projectId, scriptId }),
    enabled: projectId !== null && scriptId !== null,
  });
}

// 孤立スクリプト
export function useOrphanScripts(projectId: number | null) {
  return useQuery({
    queryKey: ["orphan_scripts", projectId],
    queryFn: () => invoke<OrphanScript[]>("get_orphan_scripts", { projectId }),
    enabled: projectId !== null,
  });
}

// 全プロジェクト一覧（プロジェクト選択ドロップダウン用）
export function useAllProjects(options?: { enabled?: boolean }) {
  return useQuery({
    queryKey: ["all_projects"],
    queryFn: () => invoke<ProjectWithSolution[]>("list_all_projects"),
    enabled: options?.enabled ?? true,
  });
}

// プロジェクト比較
export function useCompareProjects(
  projectIdA: number | null,
  projectIdB: number | null
) {
  return useQuery({
    queryKey: ["diff", projectIdA, projectIdB],
    queryFn: () =>
      invoke<DiffResult>("compare_projects", { projectIdA, projectIdB }),
    enabled: projectIdA !== null && projectIdB !== null,
  });
}

// ソリューション単位の比較
export function useCompareSolutions(
  solutionIdA: number | null,
  solutionIdB: number | null
) {
  return useQuery({
    queryKey: ["diff_solutions", solutionIdA, solutionIdB],
    queryFn: () =>
      invoke<DiffResult>("compare_solutions", { solutionIdA, solutionIdB }),
    enabled: solutionIdA !== null && solutionIdB !== null,
  });
}

// プロジェクト内全フィールド（テーブル横断）
export function useAllFields(projectId: number | null) {
  return useQuery({
    queryKey: ["all_fields", projectId],
    queryFn: () => invoke<AllFieldRow[]>("list_all_fields", { projectId }),
    enabled: projectId !== null,
  });
}

// テーブル一覧
export function useTableList(projectId: number | null) {
  return useQuery({
    queryKey: ["tables", projectId],
    queryFn: () => invoke<TableRow[]>("list_tables", { projectId }),
    enabled: projectId !== null,
  });
}

// テーブルフィールド一覧
export function useTableFields(
  projectId: number | null,
  tableId: number | null
) {
  return useQuery({
    queryKey: ["table_fields", projectId, tableId],
    queryFn: () =>
      invoke<FieldRow[]>("list_table_fields", { projectId, tableId }),
    enabled: projectId !== null && tableId !== null,
  });
}

// スクリプト一覧
export function useScriptList(projectId: number | null) {
  return useQuery({
    queryKey: ["scripts", projectId],
    queryFn: () => invoke<ScriptRow[]>("list_scripts", { projectId }),
    enabled: projectId !== null,
  });
}

// スクリプトステップ一覧
export function useScriptSteps(scriptId: number | null) {
  return useQuery({
    queryKey: ["script_steps", scriptId],
    queryFn: () => invoke<ScriptStepRow[]>("list_script_steps", { scriptId }),
    enabled: scriptId !== null,
  });
}

// レイアウト一覧
export function useLayoutList(projectId: number | null) {
  return useQuery({
    queryKey: ["layouts", projectId],
    queryFn: () => invoke<LayoutRow[]>("list_layouts", { projectId }),
    enabled: projectId !== null,
  });
}

// レイアウトオブジェクト一覧
export function useLayoutObjects(layoutId: number | null) {
  return useQuery({
    queryKey: ["layout_objects", layoutId],
    queryFn: () => invoke<LayoutObjectRow[]>("list_layout_objects", { layoutId }),
    enabled: layoutId !== null,
  });
}

// レイアウトトリガー一覧
export function useLayoutTriggers(layoutId: number | null) {
  return useQuery({
    queryKey: ["layout_triggers", layoutId],
    queryFn: () => invoke<TriggerRow[]>("list_layout_triggers", { layoutId }),
    enabled: layoutId !== null,
  });
}

// バリューリスト一覧
export function useValueListList(projectId: number | null) {
  return useQuery({
    queryKey: ["value_lists", projectId],
    queryFn: () => invoke<ValueListRow[]>("list_value_lists", { projectId }),
    enabled: projectId !== null,
  });
}

// バリューリスト値一覧
export function useValueListItems(valueListId: number | null) {
  return useQuery({
    queryKey: ["value_list_items", valueListId],
    queryFn: () =>
      invoke<string[]>("list_value_list_items", { valueListId }),
    enabled: valueListId !== null,
  });
}

// カスタム関数一覧
export function useCustomFunctionList(projectId: number | null) {
  return useQuery({
    queryKey: ["custom_functions", projectId],
    queryFn: () =>
      invoke<CustomFunctionRow[]>("list_custom_functions", { projectId }),
    enabled: projectId !== null,
  });
}

// オカレンス名+フィールド名からフィールド DB ID を解決
export function useResolveLayoutField(
  projectId: number | null,
  occurrenceName: string | null,
  fieldName: string | null
) {
  return useQuery({
    queryKey: ["resolve_layout_field", projectId, occurrenceName, fieldName],
    queryFn: () =>
      invoke<FieldLocation | null>("resolve_layout_field", {
        projectId,
        occurrenceName,
        fieldName,
      }),
    enabled: projectId !== null && occurrenceName !== null && fieldName !== null,
  });
}

// フィールド参照スクリプト一覧
export function useFieldRefs(
  projectId: number | null,
  tableName: string | null,
  fieldName: string | null
) {
  return useQuery({
    queryKey: ["field_refs", projectId, tableName, fieldName],
    queryFn: () =>
      invoke<FieldRefScript[]>("get_field_refs", { projectId, tableName, fieldName }),
    enabled: projectId !== null && tableName !== null && fieldName !== null,
  });
}

// レイアウトフィールド参照のデバッグ情報
export function useLayoutRefDebugInfo(projectId: number | null) {
  return useQuery({
    queryKey: ["layout_ref_debug", projectId],
    queryFn: () =>
      invoke<{
        occurrence_count: number;
        layout_field_ref_count: number;
        sample_occurrences: string[];
        sample_field_refs: string[];
      }>("get_layout_ref_debug_info", { projectId }),
    enabled: projectId !== null,
  });
}

// レイアウトオブジェクトの条件付き書式ルール一覧
export function useLayoutObjectConditions(objectId: number | null) {
  return useQuery({
    queryKey: ["layout_object_conditions", objectId],
    queryFn: () =>
      invoke<ConditionRow[]>("list_layout_object_conditions", { objectId }),
    enabled: objectId !== null,
  });
}

// テーブルオカレンス一覧
export function useTableOccurrenceList(projectId: number | null) {
  return useQuery({
    queryKey: ["table_occurrences", projectId],
    queryFn: () =>
      invoke<TableOccurrenceRow[]>("list_table_occurrences", { projectId }),
    enabled: projectId !== null,
  });
}

// リレーション一覧（predicates 込み）
export function useRelationshipList(projectId: number | null) {
  return useQuery({
    queryKey: ["relationships", projectId],
    queryFn: () =>
      invoke<RelationshipRow[]>("list_relationships", { projectId }),
    enabled: projectId !== null,
  });
}

// アカウント一覧
export function useAccountList(projectId: number | null) {
  return useQuery({
    queryKey: ["accounts", projectId],
    queryFn: () => invoke<AccountRow[]>("list_accounts", { projectId }),
    enabled: projectId !== null,
  });
}

// 権限セット一覧
export function usePrivilegeSetList(projectId: number | null) {
  return useQuery({
    queryKey: ["privilege_sets", projectId],
    queryFn: () => invoke<PrivilegeSetRow[]>("list_privilege_sets", { projectId }),
    enabled: projectId !== null,
  });
}

// フィールドが配置されているレイアウト一覧
export function useFieldLayoutRefs(
  projectId: number | null,
  tableName: string | null,
  fieldName: string | null
) {
  return useQuery({
    queryKey: ["field_layout_refs", projectId, tableName, fieldName],
    queryFn: () =>
      invoke<FieldRefLayout[]>("get_field_layout_refs", { projectId, tableName, fieldName }),
    enabled: projectId !== null && tableName !== null && fieldName !== null,
  });
}

// フィールドがリレーションキーとして使用されているリレーション一覧
export function useFieldRelationshipKeys(
  projectId: number | null,
  tableName: string | null,
  fieldName: string | null
) {
  return useQuery({
    queryKey: ["field_relationship_keys", projectId, tableName, fieldName],
    queryFn: () =>
      invoke<FieldRelKeyRef[]>("get_field_relationship_keys", { projectId, tableName, fieldName }),
    enabled: projectId !== null && tableName !== null && fieldName !== null,
  });
}

// 未使用フィールド一覧（レイアウト・リレーションから参照されていないフィールド）
export function useUnusedFields(projectId: number | null) {
  return useQuery({
    queryKey: ["unused_fields", projectId],
    queryFn: () => invoke<UnusedFieldRow[]>("list_unused_fields", { projectId }),
    enabled: projectId !== null,
  });
}

// アップグレードチェック
export function useUpgradeCheck(
  solutionId: number | null,
  items: CheckItem[]
) {
  const enabledConfigs = items
    .filter((i) => i.enabled)
    .map((i) => ({
      id: i.id,
      detectionType: i.detectionType,
      detectionValue: i.detectionValue,
    }));
  return useQuery({
    queryKey: ["upgrade_check", solutionId, enabledConfigs],
    queryFn: () =>
      invoke<UpgradeHit[]>("get_upgrade_check", {
        solutionId,
        checkItems: enabledConfigs,
      }),
    enabled: solutionId !== null && enabledConfigs.length > 0,
  });
}
