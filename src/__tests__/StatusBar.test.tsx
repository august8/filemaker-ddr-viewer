import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { StatusBar } from "../components/StatusBar";
import type { ProjectSummary } from "../types/ddr";

vi.mock("../hooks/solutions", () => ({
  useProjectSummary: vi.fn(),
}));

vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

import { useProjectSummary } from "../hooks/solutions";
import { useAppStore } from "../stores/appStore";

const mockSummary: ProjectSummary = {
  project: { id: 1, name: "My Project", file_path: null, fm_version: "19", imported_at: "2024-01-01" },
  table_count: 5,
  field_count: 42,
  script_count: 10,
  layout_count: 8,
  table_occurrence_count: 6,
  relationship_count: 4,
  value_list_count: 3,
  custom_function_count: 2,
  account_count: 1,
  privilege_set_count: 2,
  external_data_source_count: 1,
};

beforeEach(() => {
  vi.clearAllMocks();
});

describe("StatusBar", () => {
  it("renders_empty_when_no_project_selected", () => {
    vi.mocked(useAppStore).mockReturnValue({
      selectedProject: null,
      searchQuery: "",
      searchDuration: null,
    } as unknown as ReturnType<typeof useAppStore>);
    vi.mocked(useProjectSummary).mockReturnValue(
      { data: undefined, isLoading: false } as unknown as ReturnType<typeof useProjectSummary>
    );
    const { container } = render(<StatusBar />);
    // プロジェクト未選択・検索なしは空表示
    expect(container.firstChild).toBeEmptyDOMElement();
  });

  it("renders_element_counts_when_project_selected", () => {
    vi.mocked(useAppStore).mockReturnValue({
      selectedProject: { id: 1, name: "My Project" },
      searchQuery: "",
      searchDuration: null,
    } as unknown as ReturnType<typeof useAppStore>);
    vi.mocked(useProjectSummary).mockReturnValue(
      { data: mockSummary, isLoading: false } as unknown as ReturnType<typeof useProjectSummary>
    );
    render(<StatusBar />);
    expect(screen.getByText(/テーブル.*5/)).toBeInTheDocument();
    expect(screen.getByText(/スクリプト.*10/)).toBeInTheDocument();
    expect(screen.getByText(/レイアウト.*8/)).toBeInTheDocument();
  });

  it("renders_search_results_count_with_duration", () => {
    vi.mocked(useAppStore).mockReturnValue({
      selectedProject: { id: 1, name: "My Project" },
      searchQuery: "Contact",
      searchDuration: 42,
    } as unknown as ReturnType<typeof useAppStore>);
    vi.mocked(useProjectSummary).mockReturnValue(
      { data: mockSummary, isLoading: false } as unknown as ReturnType<typeof useProjectSummary>
    );
    render(<StatusBar />);
    expect(screen.getByText(/Contact/)).toBeInTheDocument();
    expect(screen.getByText(/42ms/)).toBeInTheDocument();
  });

  it("renders_searching_indicator_when_duration_null_and_query_set", () => {
    vi.mocked(useAppStore).mockReturnValue({
      selectedProject: { id: 1, name: "My Project" },
      searchQuery: "test",
      searchDuration: null,
    } as unknown as ReturnType<typeof useAppStore>);
    vi.mocked(useProjectSummary).mockReturnValue(
      { data: mockSummary, isLoading: false } as unknown as ReturnType<typeof useProjectSummary>
    );
    render(<StatusBar />);
    // 検索クエリはあるが duration がまだない（検索中）
    expect(screen.getByText(/test/)).toBeInTheDocument();
  });
});
