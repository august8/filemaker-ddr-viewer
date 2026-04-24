import { invoke } from "@tauri-apps/api/core";
import { useQuery } from "@tanstack/react-query";
import type { ValueListRow, CustomFunctionRow } from "../types/ddr";

// バリューリスト一覧
export function useValueListList(
  projectId: number | null,
  limit?: number,
  offset?: number
) {
  return useQuery({
    queryKey: ["value_lists", projectId, limit, offset],
    queryFn: () =>
      invoke<ValueListRow[]>("list_value_lists", {
        projectId,
        limit: limit ?? null,
        offset: offset ?? null,
      }),
    enabled: projectId !== null,
  });
}

// バリューリスト値一覧
export function useValueListItems(
  valueListId: number | null,
  limit?: number,
  offset?: number
) {
  return useQuery({
    queryKey: ["value_list_items", valueListId, limit, offset],
    queryFn: () =>
      invoke<string[]>("list_value_list_items", {
        valueListId,
        limit: limit ?? null,
        offset: offset ?? null,
      }),
    enabled: valueListId !== null,
  });
}

// カスタム関数一覧
export function useCustomFunctionList(
  projectId: number | null,
  limit?: number,
  offset?: number
) {
  return useQuery({
    queryKey: ["custom_functions", projectId, limit, offset],
    queryFn: () =>
      invoke<CustomFunctionRow[]>("list_custom_functions", {
        projectId,
        limit: limit ?? null,
        offset: offset ?? null,
      }),
    enabled: projectId !== null,
  });
}
