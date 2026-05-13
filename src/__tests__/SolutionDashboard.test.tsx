import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { SolutionDashboard } from "../components/SolutionDashboard";

vi.mock("../hooks/solutions", () => ({
  useSolutionProjectSummaries: vi.fn(() => ({ data: [], isLoading: false })),
}));

vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(() => ({ selectElement: vi.fn() })),
}));

import { useSolutionProjectSummaries } from "../hooks/solutions";
import type { ProjectSummary } from "../types/ddr";

const makeSummary = (id: number, name: string, n: number): ProjectSummary => ({
  project: { id, name, fm_version: "21", file_path: null, imported_at: "" },
  table_count: n,
  field_count: n * 2,
  script_count: n * 3,
  layout_count: n * 4,
  table_occurrence_count: n * 5,
  relationship_count: n * 6,
  value_list_count: n,
  custom_function_count: n,
  account_count: n,
  privilege_set_count: n,
  external_data_source_count: n,
});

describe("SolutionDashboard", () => {
  it("shows_spinner_while_loading", () => {
    vi.mocked(useSolutionProjectSummaries).mockReturnValue({ data: undefined, isLoading: true } as unknown as ReturnType<typeof useSolutionProjectSummaries>);
    render(<SolutionDashboard solutionId={1} solutionName="My Solution" />);
    expect(screen.getByTestId("solution-dashboard-spinner")).toBeInTheDocument();
  });

  it("shows_empty_state_when_no_projects", () => {
    vi.mocked(useSolutionProjectSummaries).mockReturnValue({ data: [], isLoading: false } as unknown as ReturnType<typeof useSolutionProjectSummaries>);
    render(<SolutionDashboard solutionId={1} solutionName="My Solution" />);
    expect(screen.getByTestId("solution-dashboard-empty")).toBeInTheDocument();
  });

  it("renders_a_row_per_project", () => {
    vi.mocked(useSolutionProjectSummaries).mockReturnValue({
      data: [makeSummary(10, "File A", 1), makeSummary(20, "File B", 2)],
      isLoading: false,
    } as unknown as ReturnType<typeof useSolutionProjectSummaries>);
    render(<SolutionDashboard solutionId={1} solutionName="My Solution" />);
    expect(screen.getByText("File A")).toBeInTheDocument();
    expect(screen.getByText("File B")).toBeInTheDocument();
  });

  it("shows_total_and_average_rows", () => {
    vi.mocked(useSolutionProjectSummaries).mockReturnValue({
      data: [makeSummary(10, "File A", 2), makeSummary(20, "File B", 4)],
      isLoading: false,
    } as unknown as ReturnType<typeof useSolutionProjectSummaries>);
    render(<SolutionDashboard solutionId={1} solutionName="My Solution" />);
    expect(screen.getByTestId("solution-total-row")).toBeInTheDocument();
    expect(screen.getByTestId("solution-average-row")).toBeInTheDocument();
  });

  it("calculates_correct_total_for_table_count", () => {
    vi.mocked(useSolutionProjectSummaries).mockReturnValue({
      data: [makeSummary(10, "File A", 3), makeSummary(20, "File B", 7)],
      isLoading: false,
    } as unknown as ReturnType<typeof useSolutionProjectSummaries>);
    render(<SolutionDashboard solutionId={1} solutionName="My Solution" />);
    const totalRow = screen.getByTestId("solution-total-row");
    expect(totalRow).toHaveTextContent("10"); // table_count: 3+7=10
  });
});
