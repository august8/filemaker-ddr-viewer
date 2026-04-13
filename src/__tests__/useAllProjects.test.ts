// src/__tests__/useAllProjects.test.ts
// Task D: useAllProjects の条件付きフェッチ (ADR-014)
// フェーズ1: Red テスト — enabled オプションが存在しないため現状は失敗する

import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import React from "react";
import { useAllProjects } from "../hooks/useTauriCommand";

// @tauri-apps/api/core の invoke をモック
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

describe("useAllProjects", () => {
  it("enabled 未指定のとき list_all_projects を invoke する", async () => {
    const { result } = renderHook(() => useAllProjects(), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("list_all_projects");
  });

  it("enabled=true のとき list_all_projects を invoke する", async () => {
    const { result } = renderHook(() => useAllProjects({ enabled: true }), {
      wrapper: createWrapper(),
    });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("list_all_projects");
  });

  it("enabled=false のとき list_all_projects を invoke しない", async () => {
    const { result } = renderHook(() => useAllProjects({ enabled: false }), {
      wrapper: createWrapper(),
    });
    // enabled=false なのでクエリは pending のまま
    expect(result.current.isSuccess).toBe(false);
    expect(result.current.fetchStatus).toBe("idle");
    expect(invoke).not.toHaveBeenCalled();
  });
});
