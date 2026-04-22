import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { MainContent } from "../components/MainContent";

vi.mock("../hooks/script", () => ({
  useScriptList: vi.fn(() => ({ data: [], isLoading: false })),
}));
vi.mock("../hooks/layout", () => ({
  useLayoutList: vi.fn(() => ({ data: [], isLoading: false })),
}));
vi.mock("../hooks/catalog", () => ({
  useValueListList: vi.fn(() => ({ data: [], isLoading: false })),
  useCustomFunctionList: vi.fn(() => ({ data: [], isLoading: false })),
}));
vi.mock("../hooks/solutions", () => ({
  useProjectSummary: vi.fn(() => ({ data: null, isLoading: false })),
}));
vi.mock("../hooks/analysis", () => ({
  useReportCard: vi.fn(() => ({ data: null, isLoading: false })),
  useBrokenRefs: vi.fn(() => ({ data: [], isLoading: false })),
  useOrphanScripts: vi.fn(() => ({ data: [], isLoading: false })),
  useUnusedFields: vi.fn(() => ({ data: [], isLoading: false })),
}));

vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(() => ({
    selectedProject: null,
    selectedElement: null,
    searchQuery: "",
    selectElement: vi.fn(),
  })),
}));

import { useAppStore } from "../stores/appStore";
import { useScriptList } from "../hooks/script";
import { useLayoutList } from "../hooks/layout";
import { useValueListList, useCustomFunctionList } from "../hooks/catalog";

describe("MainContent", () => {
  it("shows_not_found_when_script_missing", () => {
    vi.mocked(useScriptList).mockReturnValue({ data: [], isLoading: false } as unknown as ReturnType<typeof useScriptList>);
    vi.mocked(useAppStore).mockReturnValue({
      selectedProject: null,
      selectedElement: { kind: "script", id: 999, name: "Ghost Script", projectId: 10 },
      searchQuery: "",
      selectElement: vi.fn(),
    } as unknown as ReturnType<typeof useAppStore>);

    render(<MainContent />);
    expect(screen.getByText(/要素が見つかりません/)).toBeInTheDocument();
    expect(screen.getByText(/999/)).toBeInTheDocument();
  });

  it("shows_not_found_when_layout_missing", () => {
    vi.mocked(useLayoutList).mockReturnValue({ data: [], isLoading: false } as unknown as ReturnType<typeof useLayoutList>);
    vi.mocked(useAppStore).mockReturnValue({
      selectedProject: null,
      selectedElement: { kind: "layout", id: 888, name: "Ghost Layout", projectId: 10 },
      searchQuery: "",
      selectElement: vi.fn(),
    } as unknown as ReturnType<typeof useAppStore>);

    render(<MainContent />);
    expect(screen.getByText(/要素が見つかりません/)).toBeInTheDocument();
    expect(screen.getByText(/888/)).toBeInTheDocument();
  });

  it("shows_not_found_when_value_list_missing", () => {
    vi.mocked(useValueListList).mockReturnValue({ data: [], isLoading: false } as unknown as ReturnType<typeof useValueListList>);
    vi.mocked(useAppStore).mockReturnValue({
      selectedProject: null,
      selectedElement: { kind: "value_list", id: 777, name: "Ghost VL", projectId: 10 },
      searchQuery: "",
      selectElement: vi.fn(),
    } as unknown as ReturnType<typeof useAppStore>);

    render(<MainContent />);
    expect(screen.getByText(/要素が見つかりません/)).toBeInTheDocument();
    expect(screen.getByText(/777/)).toBeInTheDocument();
  });

  it("shows_not_found_when_custom_function_missing", () => {
    vi.mocked(useCustomFunctionList).mockReturnValue({ data: [], isLoading: false } as unknown as ReturnType<typeof useCustomFunctionList>);
    vi.mocked(useAppStore).mockReturnValue({
      selectedProject: null,
      selectedElement: { kind: "custom_function", id: 666, name: "Ghost CF", projectId: 10 },
      searchQuery: "",
      selectElement: vi.fn(),
    } as unknown as ReturnType<typeof useAppStore>);

    render(<MainContent />);
    expect(screen.getByText(/要素が見つかりません/)).toBeInTheDocument();
    expect(screen.getByText(/666/)).toBeInTheDocument();
  });
});
