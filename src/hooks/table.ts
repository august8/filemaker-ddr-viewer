import { invoke } from "@tauri-apps/api/core";
import { useQuery } from "@tanstack/react-query";
import type {
  AllFieldRow,
  TableRow,
  FieldRow,
  TableOccurrenceRow,
  RelationshipRow,
} from "../types/ddr";

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
