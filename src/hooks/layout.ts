import { invoke } from "@tauri-apps/api/core";
import { useQuery } from "@tanstack/react-query";
import type {
  LayoutRow,
  LayoutObjectRow,
  TriggerRow,
  ConditionRow,
} from "../types/ddr";

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

// レイアウトオブジェクトの条件付き書式ルール一覧
export function useLayoutObjectConditions(objectId: number | null) {
  return useQuery({
    queryKey: ["layout_object_conditions", objectId],
    queryFn: () =>
      invoke<ConditionRow[]>("list_layout_object_conditions", { objectId }),
    enabled: objectId !== null,
  });
}
