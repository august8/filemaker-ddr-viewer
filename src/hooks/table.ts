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
export function useAllFields(
  projectId: number | null,
  limit?: number,
  offset?: number
) {
  return useQuery({
    queryKey: ["all_fields", projectId, limit, offset],
    queryFn: () =>
      invoke<AllFieldRow[]>("list_all_fields", {
        projectId,
        limit: limit ?? null,
        offset: offset ?? null,
      }),
    enabled: projectId !== null,
  });
}

// テーブル一覧
export function useTableList(
  projectId: number | null,
  limit?: number,
  offset?: number
) {
  return useQuery({
    queryKey: ["tables", projectId, limit, offset],
    queryFn: () =>
      invoke<TableRow[]>("list_tables", {
        projectId,
        limit: limit ?? null,
        offset: offset ?? null,
      }),
    enabled: projectId !== null,
  });
}

// テーブルフィールド一覧
export function useTableFields(
  projectId: number | null,
  tableId: number | null,
  limit?: number,
  offset?: number
) {
  return useQuery({
    queryKey: ["table_fields", projectId, tableId, limit, offset],
    queryFn: () =>
      invoke<FieldRow[]>("list_table_fields", {
        projectId,
        tableId,
        limit: limit ?? null,
        offset: offset ?? null,
      }),
    enabled: projectId !== null && tableId !== null,
  });
}

// テーブルオカレンス一覧
export function useTableOccurrenceList(
  projectId: number | null,
  limit?: number,
  offset?: number
) {
  return useQuery({
    queryKey: ["table_occurrences", projectId, limit, offset],
    queryFn: () =>
      invoke<TableOccurrenceRow[]>("list_table_occurrences", {
        projectId,
        limit: limit ?? null,
        offset: offset ?? null,
      }),
    enabled: projectId !== null,
  });
}

// リレーション一覧（predicates 込み）
export function useRelationshipList(
  projectId: number | null,
  limit?: number,
  offset?: number
) {
  return useQuery({
    queryKey: ["relationships", projectId, limit, offset],
    queryFn: () =>
      invoke<RelationshipRow[]>("list_relationships", {
        projectId,
        limit: limit ?? null,
        offset: offset ?? null,
      }),
    enabled: projectId !== null,
  });
}
