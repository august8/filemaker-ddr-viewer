import { invoke } from "@tauri-apps/api/core";
import { useQuery } from "@tanstack/react-query";
import type { AccountRow, PrivilegeSetRow } from "../types/ddr";

// アカウント一覧
export function useAccountList(projectId: number | null) {
  return useQuery({
    queryKey: ["accounts", projectId],
    queryFn: () => invoke<AccountRow[]>("list_accounts", { projectId }),
    enabled: projectId !== null,
  });
}

// 権限セット一覧
export function usePrivilegeSetList(projectId: number | null) {
  return useQuery({
    queryKey: ["privilege_sets", projectId],
    queryFn: () => invoke<PrivilegeSetRow[]>("list_privilege_sets", { projectId }),
    enabled: projectId !== null,
  });
}
