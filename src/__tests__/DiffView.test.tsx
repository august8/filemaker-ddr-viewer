import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { DiffView } from "../components/DiffView";
import type { ProjectWithSolution, DiffResult } from "../types/ddr";

vi.mock("../hooks/solutions", () => ({
  useAllProjects: vi.fn(),
}));
vi.mock("../hooks/diff", () => ({
  useCompareSolutions: vi.fn(),
}));

vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

vi.mock("../hooks/analysis", () => ({
  useResolveElementByName: vi.fn(),
}));

import { useAllProjects } from "../hooks/solutions";
import { useCompareSolutions } from "../hooks/diff";
import { useAppStore } from "../stores/appStore";
import { useResolveElementByName } from "../hooks/analysis";
import type { DiffStateData } from "../stores/appStore";

const mockProjects: ProjectWithSolution[] = [
  {
    project_id: 1,
    project_name: "Project A",
    solution_id: 1,
    solution_name: "Solution A",
    solution_imported_at: "2024-01-01 10:00:00",
  },
  {
    project_id: 2,
    project_name: "Project B",
    solution_id: 2,
    solution_name: "Solution B",
    solution_imported_at: "2024-02-01 10:00:00",
  },
];

const mockDiffResult: DiffResult = {
  items: [
    { kind: "Added", element_type: "script", name: "NewScript", detail: null, project_id: 2, compare_project_id: null },
    { kind: "Removed", element_type: "script", name: "OldScript", detail: null, project_id: 1, compare_project_id: null },
  ],
  added_count: 1,
  removed_count: 1,
  modified_count: 0,
};

const INITIAL_DIFF_STATE: DiffStateData = {
  solA: null,
  solB: null,
  committedA: null,
  committedB: null,
  expandedTypes: [],
};

const mockSetDiffState = vi.fn();
const mockSelectElement = vi.fn();
const mockNavigateFromDiff = vi.fn();
const mockResolve = vi.fn();

beforeEach(() => {
  vi.clearAllMocks();
  mockResolve.mockResolvedValue({ id: 1, name: "NewScript" });
  vi.mocked(useResolveElementByName).mockReturnValue(mockResolve);
  vi.mocked(useAllProjects).mockReturnValue({
    data: mockProjects,
    isLoading: false,
  } as unknown as ReturnType<typeof useAllProjects>);
  vi.mocked(useCompareSolutions).mockReturnValue({
    data: undefined,
    isLoading: false,
  } as unknown as ReturnType<typeof useCompareSolutions>);
  vi.mocked(useAppStore).mockReturnValue({
    diffState: INITIAL_DIFF_STATE,
    setDiffState: mockSetDiffState,
    selectElement: mockSelectElement,
    navigateFromDiff: mockNavigateFromDiff,
  } as unknown as ReturnType<typeof useAppStore>);
});

describe("DiffView", () => {
  it("renders_primary_and_target_dropdowns", () => {
    render(<DiffView />);
    expect(screen.getByText("Primary")).toBeInTheDocument();
    expect(screen.getByText("Target")).toBeInTheDocument();
  });

  it("renders_compare_button", () => {
    render(<DiffView />);
    expect(screen.getByRole("button", { name: "比較する" })).toBeInTheDocument();
  });

  it("compare_button_disabled_when_nothing_selected", () => {
    render(<DiffView />);
    expect(screen.getByRole("button", { name: "比較する" })).toBeDisabled();
  });

  it("compare_button_disabled_when_same_solution_selected", () => {
    vi.mocked(useAppStore).mockReturnValue({
      diffState: { ...INITIAL_DIFF_STATE, solA: 1, solB: 1 },
      setDiffState: mockSetDiffState,
      selectElement: mockSelectElement,
      navigateFromDiff: mockNavigateFromDiff,
    } as unknown as ReturnType<typeof useAppStore>);
    render(<DiffView />);
    expect(screen.getByRole("button", { name: "比較する" })).toBeDisabled();
  });

  it("compare_button_enabled_when_different_solutions_selected", () => {
    vi.mocked(useAppStore).mockReturnValue({
      diffState: { ...INITIAL_DIFF_STATE, solA: 1, solB: 2 },
      setDiffState: mockSetDiffState,
      selectElement: mockSelectElement,
      navigateFromDiff: mockNavigateFromDiff,
    } as unknown as ReturnType<typeof useAppStore>);
    render(<DiffView />);
    expect(screen.getByRole("button", { name: "比較する" })).not.toBeDisabled();
  });

  it("shows_solution_names_in_dropdowns", () => {
    render(<DiffView />);
    // Both dropdowns should list solution names
    const solutionA = screen.getAllByText(/Solution A/);
    const solutionB = screen.getAllByText(/Solution B/);
    expect(solutionA.length).toBeGreaterThan(0);
    expect(solutionB.length).toBeGreaterThan(0);
  });

  it("shows_diff_summary_badges_after_compare", () => {
    vi.mocked(useAppStore).mockReturnValue({
      diffState: { ...INITIAL_DIFF_STATE, solA: 1, solB: 2, committedA: 1, committedB: 2 },
      setDiffState: mockSetDiffState,
      selectElement: mockSelectElement,
      navigateFromDiff: mockNavigateFromDiff,
    } as unknown as ReturnType<typeof useAppStore>);
    vi.mocked(useCompareSolutions).mockReturnValue({
      data: mockDiffResult,
      isLoading: false,
    } as unknown as ReturnType<typeof useCompareSolutions>);
    render(<DiffView />);
    expect(screen.getByText(/追加.*\+1/)).toBeInTheDocument();
    expect(screen.getByText(/削除.*-1/)).toBeInTheDocument();
  });

  it("shows_placeholder_when_no_compare_committed", () => {
    render(<DiffView />);
    expect(screen.getByText(/Primary.*Target.*選択/)).toBeInTheDocument();
  });

  it("resolve_element_called_when_navigable_item_clicked", async () => {
    vi.mocked(useAppStore).mockReturnValue({
      diffState: {
        solA: 1,
        solB: 2,
        committedA: 1,
        committedB: 2,
        expandedTypes: ["script"],
      } as DiffStateData,
      setDiffState: mockSetDiffState,
      selectElement: mockSelectElement,
      navigateFromDiff: mockNavigateFromDiff,
      selectedElement: { kind: "diff" },
    } as unknown as ReturnType<typeof useAppStore>);
    vi.mocked(useCompareSolutions).mockReturnValue({
      data: mockDiffResult,
      isLoading: false,
    } as unknown as ReturnType<typeof useCompareSolutions>);

    render(<DiffView />);
    fireEvent.click(screen.getByText("NewScript"));

    await waitFor(() => {
      expect(mockResolve).toHaveBeenCalledWith(2, "script", "NewScript");
    });
  });
});
