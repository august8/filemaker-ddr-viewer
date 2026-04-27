import { invoke } from "@tauri-apps/api/core";
import { useQuery } from "@tanstack/react-query";
import type {
  LayoutRow,
  LayoutObjectRow,
  TriggerRow,
  ConditionRow,
} from "../types/ddr";

// レイアウト一覧
export function useLayoutList(
  projectId: number | null,
  limit?: number,
  offset?: number
) {
  return useQuery({
    queryKey: ["layouts", projectId, limit, offset],
    queryFn: () =>
      invoke<LayoutRow[]>("list_layouts", {
        projectId,
        limit: limit ?? null,
        offset: offset ?? null,
      }),
    enabled: projectId !== null,
  });
}

// レイアウトオブジェクト一覧
export function useLayoutObjects(
  layoutId: number | null,
  limit?: number,
  offset?: number
) {
  return useQuery({
    queryKey: ["layout_objects", layoutId, limit, offset],
    queryFn: () =>
      invoke<LayoutObjectRow[]>("list_layout_objects", {
        layoutId,
        limit: limit ?? null,
        offset: offset ?? null,
      }),
    enabled: layoutId !== null,
  });
}

// レイアウトトリガー一覧
export function useLayoutTriggers(
  layoutId: number | null,
  limit?: number,
  offset?: number
) {
  return useQuery({
    queryKey: ["layout_triggers", layoutId, limit, offset],
    queryFn: () =>
      invoke<TriggerRow[]>("list_layout_triggers", {
        layoutId,
        limit: limit ?? null,
        offset: offset ?? null,
      }),
    enabled: layoutId !== null,
  });
}

// レイアウトオブジェクトの条件付き書式ルール一覧
export function useLayoutObjectConditions(
  objectId: number | null,
  limit?: number,
  offset?: number
) {
  return useQuery({
    queryKey: ["layout_object_conditions", objectId, limit, offset],
    queryFn: () =>
      invoke<ConditionRow[]>("list_layout_object_conditions", {
        objectId,
        limit: limit ?? null,
        offset: offset ?? null,
      }),
    enabled: objectId !== null,
  });
}
