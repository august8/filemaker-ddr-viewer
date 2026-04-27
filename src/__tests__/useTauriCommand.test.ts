import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import React from "react";
import { useSearch } from "../hooks/search";
import { useSolutions, useSolutionProjects, useProjectSummary, useAllProjects, useDeleteSolution, useDeleteProject, useImportSolution } from "../hooks/solutions";
import { useBrokenRefs, useReportCard, useUnusedFields, useOrphanScripts, useUpgradeCheck } from "../hooks/analysis";
import { useAllFields, useTableList, useTableFields, useRelationshipList, useTableOccurrenceList } from "../hooks/table";
import { useScriptList, useScriptSteps, useCallChain, useCallers } from "../hooks/script";
import { useLayoutList, useLayoutObjects, useLayoutTriggers, useLayoutObjectConditions } from "../hooks/layout";
import { useValueListList, useValueListItems, useCustomFunctionList } from "../hooks/catalog";
import { useResolveLayoutField, useFieldRefs, useFieldCalcRefs, useLayoutRefDebugInfo, useFieldLayoutRefs, useFieldRelationshipKeys } from "../hooks/fieldRefs";
import { useAccountList, usePrivilegeSetList } from "../hooks/security";
import { useCompareProjects, useCompareSolutions } from "../hooks/diff";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return ({ children }: { children: React.ReactNode }) =>
    React.createElement(QueryClientProvider, { client: queryClient }, children);
}

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(invoke).mockResolvedValue([]);
});

// ---------------------------------------------------------------------------
// useSearch — scope ロジックのテスト（パラメータ組み立てに条件分岐がある）
// ---------------------------------------------------------------------------
describe("useSearch", () => {
  it("passes_null_projectId_to_ipc_when_scope_all", async () => {
    const { result } = renderHook(
      () => useSearch(10, "hello", false, "all", null),
      { wrapper: createWrapper() }
    );
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("search_elements", {
      projectId: null,
      solutionId: null,
      query: "hello",
      contains: null,
    });
  });

  it("scope_all_different_projectId_hits_cache", async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });
    const wrapper = ({ children }: { children: React.ReactNode }) =>
      React.createElement(QueryClientProvider, { client: queryClient }, children);

    const { result: r1 } = renderHook(
      () => useSearch(10, "hello", false, "all", null),
      { wrapper }
    );
    await waitFor(() => expect(r1.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledTimes(1);

    const { result: r2 } = renderHook(
      () => useSearch(99, "hello", false, "all", null),
      { wrapper }
    );
    await waitFor(() => expect(r2.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it("scope_project_uses_projectId_in_ipc", async () => {
    const { result } = renderHook(
      () => useSearch(10, "hello", false, "project", null),
      { wrapper: createWrapper() }
    );
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("search_elements", {
      projectId: 10,
      solutionId: null,
      query: "hello",
      contains: null,
    });
  });

  it("scope_solution_uses_solutionId_in_ipc", async () => {
    const { result } = renderHook(
      () => useSearch(null, "hello", false, "solution", 5),
      { wrapper: createWrapper() }
    );
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("search_elements", {
      projectId: null,
      solutionId: 5,
      query: "hello",
      contains: null,
    });
  });

  it("empty_query_does_not_invoke", async () => {
    const { result } = renderHook(
      () => useSearch(null, "   ", false, "all", null),
      { wrapper: createWrapper() }
    );
    await new Promise((r) => setTimeout(r, 50));
    expect(result.current.fetchStatus).toBe("idle");
    expect(invoke).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// enabled=false ガード — null パラメータ時にフックが無効になることを確認
// ---------------------------------------------------------------------------
describe("useSolutionProjects", () => {
  it("is_disabled_when_solutionId_is_null", async () => {
    renderHook(() => useSolutionProjects(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

describe("useProjectSummary", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useProjectSummary(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

describe("useBrokenRefs", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useBrokenRefs(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

describe("useReportCard", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useReportCard(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

describe("useTableList", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useTableList(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// useTableFields — 2パラメータのどちらが null でも無効になることを確認
// ---------------------------------------------------------------------------
describe("useTableFields", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useTableFields(null, 1), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("is_disabled_when_tableId_is_null", async () => {
    renderHook(() => useTableFields(1, null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_list_table_fields_with_both_ids", async () => {
    const { result } = renderHook(() => useTableFields(2, 5), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("list_table_fields", {
      projectId: 2,
      tableId: 5,
      limit: null,
      offset: null,
    });
  });
});

describe("useScriptList", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useScriptList(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

describe("useScriptSteps", () => {
  it("is_disabled_when_scriptId_is_null", async () => {
    renderHook(() => useScriptSteps(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

describe("useLayoutList", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useLayoutList(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

describe("useLayoutObjects", () => {
  it("is_disabled_when_layoutId_is_null", async () => {
    renderHook(() => useLayoutObjects(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

describe("useLayoutTriggers", () => {
  it("is_disabled_when_layoutId_is_null", async () => {
    renderHook(() => useLayoutTriggers(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

describe("useRelationshipList", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useRelationshipList(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

describe("useTableOccurrenceList", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useTableOccurrenceList(null), {
      wrapper: createWrapper(),
    });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

describe("useAllProjects", () => {
  it("is_disabled_when_enabled_false", async () => {
    renderHook(() => useAllProjects({ enabled: false }), {
      wrapper: createWrapper(),
    });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

describe("useValueListList", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useValueListList(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

describe("useCustomFunctionList", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useCustomFunctionList(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// useFieldRefs / useFieldLayoutRefs / useFieldRelationshipKeys
// 複数パラメータの組み合わせが正しく IPC に渡ることを確認
// ---------------------------------------------------------------------------
describe("useFieldRefs", () => {
  it("is_disabled_when_any_param_is_null", async () => {
    renderHook(() => useFieldRefs(1, null, "Field1"), {
      wrapper: createWrapper(),
    });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_get_field_refs_with_all_params", async () => {
    const { result } = renderHook(
      () => useFieldRefs(1, "BaseTable", "Field1"),
      { wrapper: createWrapper() }
    );
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("get_field_refs", {
      projectId: 1,
      tableName: "BaseTable",
      fieldName: "Field1",
    });
  });
});

describe("useFieldLayoutRefs", () => {
  it("is_disabled_when_any_param_is_null", async () => {
    renderHook(() => useFieldLayoutRefs(null, "T", "F"), {
      wrapper: createWrapper(),
    });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_get_field_layout_refs_with_all_params", async () => {
    const { result } = renderHook(
      () => useFieldLayoutRefs(2, "Contacts", "Email"),
      { wrapper: createWrapper() }
    );
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("get_field_layout_refs", {
      projectId: 2,
      tableName: "Contacts",
      fieldName: "Email",
    });
  });
});

describe("useFieldRelationshipKeys", () => {
  it("is_disabled_when_any_param_is_null", async () => {
    renderHook(() => useFieldRelationshipKeys(1, "T", null), {
      wrapper: createWrapper(),
    });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_get_field_relationship_keys_with_all_params", async () => {
    const { result } = renderHook(
      () => useFieldRelationshipKeys(3, "Orders", "OrderID"),
      { wrapper: createWrapper() }
    );
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("get_field_relationship_keys", {
      projectId: 3,
      tableName: "Orders",
      fieldName: "OrderID",
    });
  });
});

describe("useUnusedFields", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useUnusedFields(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

describe("useOrphanScripts", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useOrphanScripts(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// useCallChain — 2パラメータの enabled ガード + IPC 呼び出し
// ---------------------------------------------------------------------------
describe("useCallChain", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useCallChain(null, 1), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("is_disabled_when_scriptId_is_null", async () => {
    renderHook(() => useCallChain(1, null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_get_call_chain_with_both_ids", async () => {
    vi.mocked(invoke).mockResolvedValue({ id: 1, name: "Script", children: [] });
    const { result } = renderHook(() => useCallChain(1, 2), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("get_call_chain", {
      projectId: 1,
      scriptId: 2,
    });
  });
});

describe("useAccountList", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useAccountList(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

describe("usePrivilegeSetList", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => usePrivilegeSetList(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// ミューテーション — 削除操作が正しい IPC を呼ぶことを確認
// ---------------------------------------------------------------------------
describe("useDeleteSolution", () => {
  it("calls_delete_solution_with_solutionId", async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    const { result } = renderHook(() => useDeleteSolution(), {
      wrapper: createWrapper(),
    });
    await act(async () => {
      await result.current.mutateAsync(3);
    });
    expect(invoke).toHaveBeenCalledWith("delete_solution", { solutionId: 3 });
  });
});

describe("useDeleteProject", () => {
  it("calls_delete_project_with_projectId", async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    const { result } = renderHook(() => useDeleteProject(), {
      wrapper: createWrapper(),
    });
    await act(async () => {
      await result.current.mutateAsync(9);
    });
    expect(invoke).toHaveBeenCalledWith("delete_project", { projectId: 9 });
  });
});

// ---------------------------------------------------------------------------
// useSolutions — パラメータなし、常時 enabled
// ---------------------------------------------------------------------------
describe("useSolutions", () => {
  it("calls_list_solutions", async () => {
    const { result } = renderHook(() => useSolutions(), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("list_solutions");
  });
});

// ---------------------------------------------------------------------------
// useImportSolution — ミューテーション
// ---------------------------------------------------------------------------
describe("useImportSolution", () => {
  it("calls_import_solution_with_summary_path", async () => {
    vi.mocked(invoke).mockResolvedValue({ id: 1, name: "Sol" });
    const { result } = renderHook(() => useImportSolution(), { wrapper: createWrapper() });
    await act(async () => {
      await result.current.mutateAsync("/path/to/summary.xml");
    });
    expect(invoke).toHaveBeenCalledWith("import_solution", { summaryPath: "/path/to/summary.xml" });
  });
});

// ---------------------------------------------------------------------------
// useAllFields — null ガード
// ---------------------------------------------------------------------------
describe("useAllFields", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useAllFields(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// useCallers — 2パラメータ null ガード
// ---------------------------------------------------------------------------
describe("useCallers", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useCallers(null, 1), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("is_disabled_when_scriptId_is_null", async () => {
    renderHook(() => useCallers(1, null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_get_callers_with_both_ids", async () => {
    const { result } = renderHook(() => useCallers(1, 5), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("get_callers", { projectId: 1, scriptId: 5 });
  });
});

// ---------------------------------------------------------------------------
// useLayoutObjectConditions — null ガード
// ---------------------------------------------------------------------------
describe("useLayoutObjectConditions", () => {
  it("is_disabled_when_objectId_is_null", async () => {
    renderHook(() => useLayoutObjectConditions(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// useValueListItems — null ガード
// ---------------------------------------------------------------------------
describe("useValueListItems", () => {
  it("is_disabled_when_valueListId_is_null", async () => {
    renderHook(() => useValueListItems(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// useResolveLayoutField — 3パラメータ null ガード + IPC 呼び出し
// ---------------------------------------------------------------------------
describe("useResolveLayoutField", () => {
  it("is_disabled_when_any_param_is_null", async () => {
    renderHook(() => useResolveLayoutField(1, null, "Name"), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_resolve_layout_field_with_all_params", async () => {
    vi.mocked(invoke).mockResolvedValue(null);
    const { result } = renderHook(
      () => useResolveLayoutField(1, "Contact", "Name"),
      { wrapper: createWrapper() }
    );
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("resolve_layout_field", {
      projectId: 1,
      occurrenceName: "Contact",
      fieldName: "Name",
    });
  });
});

// ---------------------------------------------------------------------------
// useFieldCalcRefs — 3パラメータ null ガード + IPC 呼び出し
// ---------------------------------------------------------------------------
describe("useFieldCalcRefs", () => {
  it("is_disabled_when_any_param_is_null", async () => {
    renderHook(() => useFieldCalcRefs(1, null, "Field1"), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_get_field_calc_refs_with_all_params", async () => {
    const { result } = renderHook(
      () => useFieldCalcRefs(1, "BaseTable", "Field1"),
      { wrapper: createWrapper() }
    );
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("get_field_calc_refs", {
      projectId: 1,
      tableName: "BaseTable",
      fieldName: "Field1",
    });
  });
});

// ---------------------------------------------------------------------------
// useLayoutRefDebugInfo — null ガード
// ---------------------------------------------------------------------------
describe("useLayoutRefDebugInfo", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useLayoutRefDebugInfo(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// useCompareProjects — 2パラメータ null ガード + IPC 呼び出し
// ---------------------------------------------------------------------------
describe("useCompareProjects", () => {
  it("is_disabled_when_any_param_is_null", async () => {
    renderHook(() => useCompareProjects(null, 2), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_compare_projects_with_both_ids", async () => {
    vi.mocked(invoke).mockResolvedValue({ sections: [] });
    const { result } = renderHook(() => useCompareProjects(1, 2), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("compare_projects", { projectIdA: 1, projectIdB: 2 });
  });
});

// ---------------------------------------------------------------------------
// useCompareSolutions — 2パラメータ null ガード + IPC 呼び出し
// ---------------------------------------------------------------------------
describe("useCompareSolutions", () => {
  it("is_disabled_when_any_param_is_null", async () => {
    renderHook(() => useCompareSolutions(null, 2), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_compare_solutions_with_both_ids", async () => {
    vi.mocked(invoke).mockResolvedValue({ sections: [] });
    const { result } = renderHook(() => useCompareSolutions(1, 2), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("compare_solutions", { solutionIdA: 1, solutionIdB: 2 });
  });
});

// ---------------------------------------------------------------------------
// useUpgradeCheck — enabled 条件の分岐テスト
// ---------------------------------------------------------------------------
describe("useUpgradeCheck", () => {
  const makeItem = (overrides: { enabled: boolean; id?: string }) => ({
    id: overrides.id ?? "item1",
    label: "X",
    category: "step" as const,
    detectionType: "step_type_id" as const,
    detectionValue: "89",
    builtin: true,
    enabled: overrides.enabled,
  });

  it("is_disabled_when_solutionId_is_null", async () => {
    renderHook(() => useUpgradeCheck(null, [makeItem({ enabled: true })]), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("is_disabled_when_no_enabled_items", async () => {
    renderHook(() => useUpgradeCheck(1, [makeItem({ enabled: false })]), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_get_upgrade_check_with_enabled_items_only", async () => {
    const items = [
      makeItem({ enabled: true, id: "item1" }),
      makeItem({ enabled: false, id: "item2" }),
    ];
    const { result } = renderHook(() => useUpgradeCheck(1, items), { wrapper: createWrapper() });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("get_upgrade_check", {
      solutionId: 1,
      checkItems: [{ id: "item1", detectionType: "step_type_id", detectionValue: "89" }],
    });
  });
});
