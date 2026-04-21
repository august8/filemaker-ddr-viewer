import { invoke } from "@tauri-apps/api/core";
import { useQuery } from "@tanstack/react-query";
import type { ValueListRow, CustomFunctionRow } from "../types/ddr";

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
