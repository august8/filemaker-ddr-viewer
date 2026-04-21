import { invoke } from "@tauri-apps/api/core";
import { useQuery } from "@tanstack/react-query";
import type {
  FieldLocation,
  FieldRefScript,
  FieldCalcRef,
  FieldRefLayout,
  FieldRelKeyRef,
} from "../types/ddr";

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

// 計算式でこのフィールドを参照している他フィールド一覧
export function useFieldCalcRefs(
  projectId: number | null,
  tableName: string | null,
  fieldName: string | null
) {
  return useQuery({
    queryKey: ["field_calc_refs", projectId, tableName, fieldName],
    queryFn: () =>
      invoke<FieldCalcRef[]>("get_field_calc_refs", { projectId, tableName, fieldName }),
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
