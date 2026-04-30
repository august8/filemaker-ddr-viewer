import { invoke } from "@tauri-apps/api/core";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import type {
  SolutionRow,
  SolutionWithProjects,
  ProjectRow,
  ProjectSummary,
  ProjectWithSolution,
} from "../types/ddr";

// ソリューション一覧
export function useSolutions() {
  return useQuery({
    queryKey: ["solutions"],
    queryFn: () => invoke<SolutionRow[]>("list_solutions"),
    retry: 2,
    retryDelay: 1000,
  });
}

// ソリューション内のプロジェクト一覧
export function useSolutionProjects(solutionId: number | null) {
  return useQuery({
    queryKey: ["solution_projects", solutionId],
    queryFn: () =>
      invoke<ProjectRow[]>("get_solution_projects", { solutionId }),
    enabled: solutionId !== null,
  });
}

// ソリューションインポート
export function useImportSolution() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (summaryPath: string) =>
      invoke<SolutionWithProjects>("import_solution", { summaryPath }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["solutions"] });
      qc.invalidateQueries({ queryKey: ["all_projects"] });
    },
  });
}

// ソリューション削除
export function useDeleteSolution() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (solutionId: number) =>
      invoke<void>("delete_solution", { solutionId }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["solutions"] });
      qc.invalidateQueries({ queryKey: ["all_projects"] });
    },
  });
}

// プロジェクト削除
export function useDeleteProject() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (projectId: number) =>
      invoke<void>("delete_project", { projectId }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["solutions"] });
      qc.invalidateQueries({ queryKey: ["all_projects"] });
    },
  });
}

// プロジェクトサマリー
export function useProjectSummary(projectId: number | null) {
  return useQuery({
    queryKey: ["project_summary", projectId],
    queryFn: () => invoke<ProjectSummary>("get_project_summary", { projectId }),
    enabled: projectId !== null,
  });
}

// 全プロジェクト一覧（プロジェクト選択ドロップダウン用）
export function useAllProjects(options?: { enabled?: boolean }) {
  return useQuery({
    queryKey: ["all_projects"],
    queryFn: () => invoke<ProjectWithSolution[]>("list_all_projects"),
    enabled: options?.enabled ?? true,
  });
}
