import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import React from "react";
import { useSearch } from "../hooks/useTauriCommand";

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
    // staleTime: Infinity で再フェッチを防ぎ、キャッシュヒットを検証する
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });
    const wrapper = ({ children }: { children: React.ReactNode }) =>
      React.createElement(QueryClientProvider, { client: queryClient }, children);

    // projectId=10 で最初の呼び出し
    const { result: r1 } = renderHook(
      () => useSearch(10, "hello", false, "all", null),
      { wrapper }
    );
    await waitFor(() => expect(r1.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledTimes(1);

    // projectId=99 で再呼び出し: scope="all" なので queryKey は同じ → キャッシュヒット
    const { result: r2 } = renderHook(
      () => useSearch(99, "hello", false, "all", null),
      { wrapper }
    );
    await waitFor(() => expect(r2.current.isSuccess).toBe(true));
    // invoke は追加で呼ばれない（キャッシュから返される）
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
});
