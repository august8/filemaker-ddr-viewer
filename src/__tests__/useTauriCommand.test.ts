import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import React from "react";
import {
  useSearch,
  useSolutions,
  useSolutionProjects,
  useProjectSummary,
  useBrokenRefs,
  useReportCard,
  useTableList,
  useTableFields,
  useScriptList,
  useScriptSteps,
  useLayoutList,
  useLayoutObjects,
  useLayoutTriggers,
  useRelationshipList,
  useTableOccurrenceList,
  useAllProjects,
  useDeleteSolution,
  useDeleteProject,
  useValueListList,
  useCustomFunctionList,
  useFieldRefs,
  useFieldLayoutRefs,
  useFieldRelationshipKeys,
  useUnusedFields,
  useOrphanScripts,
  useCallChain,
  useAccountList,
  usePrivilegeSetList,
} from "../hooks/useTauriCommand";

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
// useSearch
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
    // wait a tick to ensure no async invoke fires
    await new Promise((r) => setTimeout(r, 50));
    expect(result.current.fetchStatus).toBe("idle");
    expect(invoke).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// useSolutions
// ---------------------------------------------------------------------------
describe("useSolutions", () => {
  it("calls_list_solutions_with_no_params", async () => {
    const { result } = renderHook(() => useSolutions(), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("list_solutions");
  });

  it("returns_data_from_invoke", async () => {
    const mockData = [{ id: 1, name: "MySolution" }];
    vi.mocked(invoke).mockResolvedValue(mockData);
    const { result } = renderHook(() => useSolutions(), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual(mockData);
  });
});

// ---------------------------------------------------------------------------
// useSolutionProjects
// ---------------------------------------------------------------------------
describe("useSolutionProjects", () => {
  it("is_disabled_when_solutionId_is_null", async () => {
    renderHook(() => useSolutionProjects(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_get_solution_projects_with_solutionId", async () => {
    const { result } = renderHook(() => useSolutionProjects(5), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("get_solution_projects", {
      solutionId: 5,
    });
  });
});

// ---------------------------------------------------------------------------
// useProjectSummary
// ---------------------------------------------------------------------------
describe("useProjectSummary", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useProjectSummary(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_get_project_summary_with_projectId", async () => {
    vi.mocked(invoke).mockResolvedValue({ table_count: 3, script_count: 5 });
    const { result } = renderHook(() => useProjectSummary(7), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("get_project_summary", {
      projectId: 7,
    });
  });
});

// ---------------------------------------------------------------------------
// useBrokenRefs
// ---------------------------------------------------------------------------
describe("useBrokenRefs", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useBrokenRefs(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_get_broken_refs_with_projectId", async () => {
    const { result } = renderHook(() => useBrokenRefs(3), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("get_broken_refs", { projectId: 3 });
  });
});

// ---------------------------------------------------------------------------
// useReportCard
// ---------------------------------------------------------------------------
describe("useReportCard", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useReportCard(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_get_report_card_with_projectId", async () => {
    vi.mocked(invoke).mockResolvedValue({ score: 90, items: [] });
    const { result } = renderHook(() => useReportCard(4), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("get_report_card", { projectId: 4 });
  });
});

// ---------------------------------------------------------------------------
// useTableList
// ---------------------------------------------------------------------------
describe("useTableList", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useTableList(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_list_tables_with_projectId", async () => {
    const { result } = renderHook(() => useTableList(2), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("list_tables", { projectId: 2 });
  });
});

// ---------------------------------------------------------------------------
// useTableFields
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
    });
  });
});

// ---------------------------------------------------------------------------
// useScriptList
// ---------------------------------------------------------------------------
describe("useScriptList", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useScriptList(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_list_scripts_with_projectId", async () => {
    const { result } = renderHook(() => useScriptList(6), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("list_scripts", { projectId: 6 });
  });
});

// ---------------------------------------------------------------------------
// useScriptSteps
// ---------------------------------------------------------------------------
describe("useScriptSteps", () => {
  it("is_disabled_when_scriptId_is_null", async () => {
    renderHook(() => useScriptSteps(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_list_script_steps_with_scriptId", async () => {
    const { result } = renderHook(() => useScriptSteps(9), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("list_script_steps", { scriptId: 9 });
  });
});

// ---------------------------------------------------------------------------
// useLayoutList
// ---------------------------------------------------------------------------
describe("useLayoutList", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useLayoutList(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_list_layouts_with_projectId", async () => {
    const { result } = renderHook(() => useLayoutList(1), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("list_layouts", { projectId: 1 });
  });
});

// ---------------------------------------------------------------------------
// useLayoutObjects
// ---------------------------------------------------------------------------
describe("useLayoutObjects", () => {
  it("is_disabled_when_layoutId_is_null", async () => {
    renderHook(() => useLayoutObjects(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_list_layout_objects_with_layoutId", async () => {
    const { result } = renderHook(() => useLayoutObjects(3), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("list_layout_objects", {
      layoutId: 3,
    });
  });
});

// ---------------------------------------------------------------------------
// useLayoutTriggers
// ---------------------------------------------------------------------------
describe("useLayoutTriggers", () => {
  it("is_disabled_when_layoutId_is_null", async () => {
    renderHook(() => useLayoutTriggers(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_list_layout_triggers_with_layoutId", async () => {
    const { result } = renderHook(() => useLayoutTriggers(7), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("list_layout_triggers", {
      layoutId: 7,
    });
  });
});

// ---------------------------------------------------------------------------
// useRelationshipList
// ---------------------------------------------------------------------------
describe("useRelationshipList", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useRelationshipList(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_list_relationships_with_projectId", async () => {
    const { result } = renderHook(() => useRelationshipList(8), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("list_relationships", {
      projectId: 8,
    });
  });
});

// ---------------------------------------------------------------------------
// useTableOccurrenceList
// ---------------------------------------------------------------------------
describe("useTableOccurrenceList", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useTableOccurrenceList(null), {
      wrapper: createWrapper(),
    });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_list_table_occurrences_with_projectId", async () => {
    const { result } = renderHook(() => useTableOccurrenceList(10), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("list_table_occurrences", {
      projectId: 10,
    });
  });
});

// ---------------------------------------------------------------------------
// useAllProjects
// ---------------------------------------------------------------------------
describe("useAllProjects", () => {
  it("calls_list_all_projects_with_no_params", async () => {
    const { result } = renderHook(() => useAllProjects(), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("list_all_projects");
  });

  it("is_disabled_when_enabled_false", async () => {
    renderHook(() => useAllProjects({ enabled: false }), {
      wrapper: createWrapper(),
    });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// useValueListList
// ---------------------------------------------------------------------------
describe("useValueListList", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useValueListList(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_list_value_lists_with_projectId", async () => {
    const { result } = renderHook(() => useValueListList(2), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("list_value_lists", { projectId: 2 });
  });
});

// ---------------------------------------------------------------------------
// useCustomFunctionList
// ---------------------------------------------------------------------------
describe("useCustomFunctionList", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useCustomFunctionList(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_list_custom_functions_with_projectId", async () => {
    const { result } = renderHook(() => useCustomFunctionList(11), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("list_custom_functions", {
      projectId: 11,
    });
  });
});

// ---------------------------------------------------------------------------
// useFieldRefs
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

// ---------------------------------------------------------------------------
// useFieldLayoutRefs
// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
// useFieldRelationshipKeys
// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
// useUnusedFields
// ---------------------------------------------------------------------------
describe("useUnusedFields", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useUnusedFields(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_list_unused_fields_with_projectId", async () => {
    const { result } = renderHook(() => useUnusedFields(4), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("list_unused_fields", {
      projectId: 4,
    });
  });
});

// ---------------------------------------------------------------------------
// useOrphanScripts
// ---------------------------------------------------------------------------
describe("useOrphanScripts", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useOrphanScripts(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_get_orphan_scripts_with_projectId", async () => {
    const { result } = renderHook(() => useOrphanScripts(5), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("get_orphan_scripts", {
      projectId: 5,
    });
  });
});

// ---------------------------------------------------------------------------
// useCallChain
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

// ---------------------------------------------------------------------------
// useAccountList
// ---------------------------------------------------------------------------
describe("useAccountList", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => useAccountList(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_list_accounts_with_projectId", async () => {
    const { result } = renderHook(() => useAccountList(6), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("list_accounts", { projectId: 6 });
  });
});

// ---------------------------------------------------------------------------
// usePrivilegeSetList
// ---------------------------------------------------------------------------
describe("usePrivilegeSetList", () => {
  it("is_disabled_when_projectId_is_null", async () => {
    renderHook(() => usePrivilegeSetList(null), { wrapper: createWrapper() });
    await new Promise((r) => setTimeout(r, 50));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("calls_list_privilege_sets_with_projectId", async () => {
    const { result } = renderHook(() => usePrivilegeSetList(7), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("list_privilege_sets", {
      projectId: 7,
    });
  });
});

// ---------------------------------------------------------------------------
// useDeleteSolution (mutation)
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

// ---------------------------------------------------------------------------
// useDeleteProject (mutation)
// ---------------------------------------------------------------------------
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
