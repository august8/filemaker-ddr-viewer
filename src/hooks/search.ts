import { invoke } from "@tauri-apps/api/core";
import { useQuery } from "@tanstack/react-query";
import type { SearchResult } from "../types/ddr";

// 検索
export function useSearch(
  projectId: number | null,
  query: string,
  contains: boolean = false,
  scope: "all" | "solution" | "project" = "all",
  solutionId: number | null = null,
) {
  const effectiveProjectId = scope === "project" ? projectId : null;
  const effectiveSolutionId = scope === "solution" ? solutionId : null;
  return useQuery({
    queryKey: ["search", query, contains, scope, effectiveProjectId, effectiveSolutionId],
    queryFn: () =>
      invoke<SearchResult[]>("search_elements", {
        projectId: effectiveProjectId,
        solutionId: effectiveSolutionId,
        query,
        contains: contains || null,
        // limit 省略 → Rust 側が None として全件返却（SQLite LIMIT -1）
      }),
    enabled: query.trim().length > 0,
  });
}
