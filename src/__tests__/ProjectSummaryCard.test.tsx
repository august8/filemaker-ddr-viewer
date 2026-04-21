import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ProjectSummaryCard } from "../components/ProjectSummaryCard";
import type { ProjectSummary } from "../types/ddr";

vi.mock("../hooks/solutions", () => ({
  useProjectSummary: vi.fn(),
}));

vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

import { useProjectSummary } from "../hooks/solutions";
import { useAppStore } from "../stores/appStore";

const mockSelectElement = vi.fn();

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
};

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useAppStore).mockReturnValue({
    selectElement: mockSelectElement,
  } as unknown as ReturnType<typeof useAppStore>);
});

describe("ProjectSummaryCard", () => {
  it("renders_nothing_when_no_project", () => {
    vi.mocked(useProjectSummary).mockReturnValue(
      { data: undefined, isLoading: false } as unknown as ReturnType<typeof useProjectSummary>
    );
    const { container } = render(<ProjectSummaryCard projectId={null} />);
    expect(container.firstChild).toBeNull();
  });

  it("renders_summary_counts", () => {
    vi.mocked(useProjectSummary).mockReturnValue(
      { data: mockSummary, isLoading: false } as unknown as ReturnType<typeof useProjectSummary>
    );
    render(<ProjectSummaryCard projectId={1} />);
    expect(screen.getByText("5")).toBeInTheDocument(); // table_count
    expect(screen.getByText("42")).toBeInTheDocument(); // field_count
    expect(screen.getByText("10")).toBeInTheDocument(); // script_count
    expect(screen.getByText("8")).toBeInTheDocument(); // layout_count
    expect(screen.getByText("My Project")).toBeInTheDocument();
  });

  it("table_count_click_navigates_to_all_tables", () => {
    vi.mocked(useProjectSummary).mockReturnValue(
      { data: mockSummary, isLoading: false } as unknown as ReturnType<typeof useProjectSummary>
    );
    render(<ProjectSummaryCard projectId={1} />);
    fireEvent.click(screen.getByRole("button", { name: "5" }));
    expect(mockSelectElement).toHaveBeenCalledWith({ kind: "all_tables", projectId: 1 });
  });

  it("script_count_click_navigates_to_all_scripts", () => {
    vi.mocked(useProjectSummary).mockReturnValue(
      { data: mockSummary, isLoading: false } as unknown as ReturnType<typeof useProjectSummary>
    );
    render(<ProjectSummaryCard projectId={1} />);
    fireEvent.click(screen.getByRole("button", { name: "10" }));
    expect(mockSelectElement).toHaveBeenCalledWith({ kind: "all_scripts", projectId: 1 });
  });

  it("layout_count_click_navigates_to_all_layouts", () => {
    vi.mocked(useProjectSummary).mockReturnValue(
      { data: mockSummary, isLoading: false } as unknown as ReturnType<typeof useProjectSummary>
    );
    render(<ProjectSummaryCard projectId={1} />);
    fireEvent.click(screen.getByRole("button", { name: "8" }));
    expect(mockSelectElement).toHaveBeenCalledWith({ kind: "all_layouts", projectId: 1 });
  });

  it("value_list_count_click_navigates_to_all_value_lists", () => {
    vi.mocked(useProjectSummary).mockReturnValue(
      { data: mockSummary, isLoading: false } as unknown as ReturnType<typeof useProjectSummary>
    );
    render(<ProjectSummaryCard projectId={1} />);
    fireEvent.click(screen.getByRole("button", { name: "3" }));
    expect(mockSelectElement).toHaveBeenCalledWith({ kind: "all_value_lists", projectId: 1 });
  });

  it("custom_function_count_click_navigates_to_all_custom_functions", () => {
    vi.mocked(useProjectSummary).mockReturnValue(
      { data: mockSummary, isLoading: false } as unknown as ReturnType<typeof useProjectSummary>
    );
    render(<ProjectSummaryCard projectId={1} />);
    fireEvent.click(screen.getByRole("button", { name: "2" }));
    expect(mockSelectElement).toHaveBeenCalledWith({ kind: "all_custom_functions", projectId: 1 });
  });

  it("field_count_has_no_click_handler", () => {
    vi.mocked(useProjectSummary).mockReturnValue(
      { data: mockSummary, isLoading: false } as unknown as ReturnType<typeof useProjectSummary>
    );
    render(<ProjectSummaryCard projectId={1} />);
    // フィールド数 42 は span（ボタンでない）
    expect(screen.queryByRole("button", { name: "42" })).toBeNull();
    expect(screen.getByText("42")).toBeInTheDocument();
  });
});
