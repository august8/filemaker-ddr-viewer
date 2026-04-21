import { invoke } from "@tauri-apps/api/core";
import { useQuery } from "@tanstack/react-query";
import type { DiffResult } from "../types/ddr";

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
