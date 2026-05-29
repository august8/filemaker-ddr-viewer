import { invoke } from "@tauri-apps/api/core";
import { useQuery } from "@tanstack/react-query";
import type {
  BrokenRef,
  BrokenRefWithProject,
  ProjectRow,
  ReportCard,
  OrphanScript,
  UnusedFieldRow,
  UpgradeHit,
} from "../types/ddr";
import type { CheckItem } from "../stores/appStore";

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

// 孤立スクリプト
export function useOrphanScripts(projectId: number | null) {
  return useQuery({
    queryKey: ["orphan_scripts", projectId],
    queryFn: () => invoke<OrphanScript[]>("get_orphan_scripts", { projectId }),
    enabled: projectId !== null,
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

// ソリューション全体の壊れた参照（全プロジェクトをまとめて取得）
export function useSolutionBrokenRefs(solutionId: number | null) {
  return useQuery({
    queryKey: ["solution_broken_refs", solutionId],
    queryFn: async (): Promise<BrokenRefWithProject[]> => {
      const projects = await invoke<ProjectRow[]>("get_solution_projects", { solutionId });
      const allRefs = await Promise.all(
        projects.map(async (p) => {
          const refs = await invoke<BrokenRef[]>("get_broken_refs", { projectId: p.id });
          return refs.map((r) => ({ ...r, project_id: p.id, project_name: p.name }));
        })
      );
      return allRefs.flat();
    },
    enabled: solutionId !== null,
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
