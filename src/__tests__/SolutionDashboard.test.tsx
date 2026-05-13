import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { SolutionDashboard } from "../components/SolutionDashboard";

vi.mock("../hooks/solutions", () => ({
  useSolutionProjects: vi.fn(() => ({ data: [], isLoading: false })),
  useProjectSummary: vi.fn(() => ({ data: null, isLoading: false })),
}));

vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(() => ({ selectElement: vi.fn() })),
}));

import { useSolutionProjects } from "../hooks/solutions";

describe("SolutionDashboard", () => {
  it("shows_spinner_while_loading", () => {
    vi.mocked(useSolutionProjects).mockReturnValue({ data: undefined, isLoading: true } as unknown as ReturnType<typeof useSolutionProjects>);
    render(<SolutionDashboard solutionId={1} solutionName="My Solution" />);
    expect(screen.getByTestId("solution-dashboard-spinner")).toBeInTheDocument();
  });

  it("renders_project_summary_card_for_each_project", () => {
    vi.mocked(useSolutionProjects).mockReturnValue({
      data: [
        { id: 10, name: "File A", fm_version: "21", solution_id: 1, imported_at: "" },
        { id: 20, name: "File B", fm_version: "21", solution_id: 1, imported_at: "" },
      ],
      isLoading: false,
    } as unknown as ReturnType<typeof useSolutionProjects>);
    render(<SolutionDashboard solutionId={1} solutionName="My Solution" />);
    expect(screen.getByTestId("solution-project-card-10")).toBeInTheDocument();
    expect(screen.getByTestId("solution-project-card-20")).toBeInTheDocument();
  });

  it("shows_empty_state_when_no_projects", () => {
    vi.mocked(useSolutionProjects).mockReturnValue({ data: [], isLoading: false } as unknown as ReturnType<typeof useSolutionProjects>);
    render(<SolutionDashboard solutionId={1} solutionName="My Solution" />);
    expect(screen.getByTestId("solution-dashboard-empty")).toBeInTheDocument();
  });
});
