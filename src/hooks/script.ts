import { invoke } from "@tauri-apps/api/core";
import { useQuery } from "@tanstack/react-query";
import type { CallChainNode, ScriptRow, ScriptStepRow } from "../types/ddr";

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
