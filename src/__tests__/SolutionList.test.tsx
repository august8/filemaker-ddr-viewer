import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { SolutionList } from "../components/SolutionList";
import type { SolutionRow } from "../types/ddr";

vi.mock("../hooks/solutions", () => ({
  useSolutions: vi.fn(),
  useDeleteSolution: vi.fn(),
  useSolutionProjects: vi.fn(),
  useDeleteProject: vi.fn(() => ({ mutate: vi.fn(), isPending: false })),
}));

vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

// CategoryTree は別テストで担保。ここでは stub
vi.mock("../components/navigation/CategoryTree", () => ({
  CategoryTree: () => null,
}));

import { useSolutions, useDeleteSolution, useSolutionProjects, useDeleteProject } from "../hooks/solutions";
import { useAppStore } from "../stores/appStore";

const mockSolutions: SolutionRow[] = [
  { id: 1, name: "Solution A", summary_path: null, imported_at: "2024-01-01T00:00:00Z" },
  { id: 2, name: "Solution B", summary_path: null, imported_at: "2024-06-15T00:00:00Z" },
];

const mockSelectSolution = vi.fn();
const mockSelectProject = vi.fn();
const mockSelectElement = vi.fn();
const mockSetRightPanel = vi.fn();

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useAppStore).mockReturnValue({
    selectedSolution: null,
    selectedProject: null,
    selectSolution: mockSelectSolution,
    selectProject: mockSelectProject,
    selectElement: mockSelectElement,
    setRightPanel: mockSetRightPanel,
  } as unknown as ReturnType<typeof useAppStore>);
  vi.mocked(useDeleteSolution).mockReturnValue(
    { mutate: vi.fn(), isPending: false } as unknown as ReturnType<typeof useDeleteSolution>
  );
  vi.mocked(useSolutionProjects).mockReturnValue(
    { data: [], isLoading: false } as unknown as ReturnType<typeof useSolutionProjects>
  );
  vi.mocked(useDeleteProject).mockReturnValue(
    { mutate: vi.fn(), isPending: false } as unknown as ReturnType<typeof useDeleteProject>
  );
});

describe("SolutionList", () => {
  it("shows_loading_state", () => {
    vi.mocked(useSolutions).mockReturnValue(
      { data: undefined, isLoading: true, isError: false } as unknown as ReturnType<typeof useSolutions>
    );
    render(<SolutionList />);
    expect(screen.getByText("読み込み中...")).toBeInTheDocument();
  });

  it("shows_empty_state", () => {
    vi.mocked(useSolutions).mockReturnValue(
      { data: [], isLoading: false, isError: false } as unknown as ReturnType<typeof useSolutions>
    );
    render(<SolutionList />);
    expect(screen.getByText("DDR をインポートしてください")).toBeInTheDocument();
  });

  it("shows_solution_names", () => {
    vi.mocked(useSolutions).mockReturnValue(
      { data: mockSolutions, isLoading: false, isError: false } as unknown as ReturnType<typeof useSolutions>
    );
    render(<SolutionList />);
    expect(screen.getByText(/Solution A/)).toBeInTheDocument();
    expect(screen.getByText(/Solution B/)).toBeInTheDocument();
  });

  it("delete_button_shows_inline_confirmation", () => {
    vi.mocked(useSolutions).mockReturnValue(
      { data: mockSolutions, isLoading: false, isError: false } as unknown as ReturnType<typeof useSolutions>
    );
    render(<SolutionList />);
    // ×ボタン押下 → 確認UIが表示される
    const deleteButtons = screen.getAllByRole("button", { name: "削除" });
    fireEvent.click(deleteButtons[0]);
    expect(screen.getByText(/「Solution A」を削除しますか？/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "キャンセル" })).toBeInTheDocument();
  });

  it("delete_cancel_hides_confirmation", () => {
    vi.mocked(useSolutions).mockReturnValue(
      { data: mockSolutions, isLoading: false, isError: false } as unknown as ReturnType<typeof useSolutions>
    );
    render(<SolutionList />);
    const deleteButtons = screen.getAllByRole("button", { name: "削除" });
    fireEvent.click(deleteButtons[0]);
    expect(screen.getByText(/「Solution A」を削除しますか？/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "キャンセル" }));
    expect(screen.queryByText(/「Solution A」を削除しますか？/)).not.toBeInTheDocument();
  });

  it("delete_confirm_calls_mutation", () => {
    const mutateMock = vi.fn();
    vi.mocked(useDeleteSolution).mockReturnValue(
      { mutate: mutateMock, isPending: false } as unknown as ReturnType<typeof useDeleteSolution>
    );
    vi.mocked(useSolutions).mockReturnValue(
      { data: mockSolutions, isLoading: false, isError: false } as unknown as ReturnType<typeof useSolutions>
    );
    render(<SolutionList />);
    const deleteButtons = screen.getAllByRole("button", { name: "削除" });
    fireEvent.click(deleteButtons[0]);
    const confirmBtn = screen.getByText("削除", { selector: "button:not([aria-label])" });
    fireEvent.click(confirmBtn);
    expect(mutateMock).toHaveBeenCalledWith(1, expect.objectContaining({ onSettled: expect.any(Function) }));
  });

  it("shows_error_state", () => {
    vi.mocked(useSolutions).mockReturnValue(
      { data: undefined, isLoading: false, isError: true } as unknown as ReturnType<typeof useSolutions>
    );
    render(<SolutionList />);
    expect(screen.getByText("読み込みエラー")).toBeInTheDocument();
  });

  it("clicking_solution_row_calls_selectSolution", () => {
    vi.mocked(useSolutions).mockReturnValue(
      { data: mockSolutions, isLoading: false, isError: false } as unknown as ReturnType<typeof useSolutions>
    );
    render(<SolutionList />);
    const row = screen.getByText(/Solution A/).closest("div[class*='cursor-pointer']")!;
    fireEvent.click(row);
    expect(mockSelectSolution).toHaveBeenCalledWith(mockSolutions[0]);
    expect(mockSelectProject).toHaveBeenCalledWith(null);
    expect(mockSetRightPanel).toHaveBeenCalledWith(null);
  });

  it("selected_solution_shows_upgrade_check_button", () => {
    vi.mocked(useAppStore).mockReturnValue({
      selectedSolution: mockSolutions[0],
      selectedProject: null,
      selectSolution: mockSelectSolution,
      selectProject: mockSelectProject,
      selectElement: mockSelectElement,
      setRightPanel: mockSetRightPanel,
    } as unknown as ReturnType<typeof useAppStore>);
    vi.mocked(useSolutions).mockReturnValue(
      { data: mockSolutions, isLoading: false, isError: false } as unknown as ReturnType<typeof useSolutions>
    );
    render(<SolutionList />);
    expect(screen.getByText("アップグレードチェック")).toBeInTheDocument();
  });

  it("upgrade_check_button_calls_selectElement", () => {
    vi.mocked(useAppStore).mockReturnValue({
      selectedSolution: mockSolutions[0],
      selectedProject: null,
      selectSolution: mockSelectSolution,
      selectProject: mockSelectProject,
      selectElement: mockSelectElement,
      setRightPanel: mockSetRightPanel,
    } as unknown as ReturnType<typeof useAppStore>);
    vi.mocked(useSolutions).mockReturnValue(
      { data: mockSolutions, isLoading: false, isError: false } as unknown as ReturnType<typeof useSolutions>
    );
    render(<SolutionList />);
    const upgradeBtn = screen.getByText("アップグレードチェック");
    fireEvent.click(upgradeBtn);
    expect(mockSelectElement).toHaveBeenCalledWith({ kind: "upgrade_check", solutionId: 1 });
  });

  it("delete_selected_solution_clears_selection", () => {
    const mutateMock = vi.fn();
    vi.mocked(useDeleteSolution).mockReturnValue(
      { mutate: mutateMock, isPending: false } as unknown as ReturnType<typeof useDeleteSolution>
    );
    vi.mocked(useAppStore).mockReturnValue({
      selectedSolution: mockSolutions[0],
      selectedProject: null,
      selectSolution: mockSelectSolution,
      selectProject: mockSelectProject,
      selectElement: mockSelectElement,
      setRightPanel: mockSetRightPanel,
    } as unknown as ReturnType<typeof useAppStore>);
    vi.mocked(useSolutions).mockReturnValue(
      { data: mockSolutions, isLoading: false, isError: false } as unknown as ReturnType<typeof useSolutions>
    );
    render(<SolutionList />);
    const deleteButtons = screen.getAllByRole("button", { name: "削除" });
    fireEvent.click(deleteButtons[0]);
    const confirmBtn = screen.getByText("削除", { selector: "button:not([aria-label])" });
    fireEvent.click(confirmBtn);
    expect(mockSelectSolution).toHaveBeenCalledWith(null);
    expect(mockSelectProject).toHaveBeenCalledWith(null);
    expect(mockSetRightPanel).toHaveBeenCalledWith(null);
  });

  it("project_items_shows_loading", () => {
    vi.mocked(useAppStore).mockReturnValue({
      selectedSolution: mockSolutions[0],
      selectedProject: null,
      selectSolution: mockSelectSolution,
      selectProject: mockSelectProject,
      selectElement: mockSelectElement,
      setRightPanel: mockSetRightPanel,
    } as unknown as ReturnType<typeof useAppStore>);
    vi.mocked(useSolutions).mockReturnValue(
      { data: mockSolutions, isLoading: false, isError: false } as unknown as ReturnType<typeof useSolutions>
    );
    vi.mocked(useSolutionProjects).mockReturnValue(
      { data: undefined, isLoading: true } as unknown as ReturnType<typeof useSolutionProjects>
    );
    render(<SolutionList />);
    // ProjectItems の loading スピナー（"読み込み中..." は複数ある可能性）
    const loadingTexts = screen.getAllByText("読み込み中...");
    expect(loadingTexts.length).toBeGreaterThan(0);
  });

  it("project_items_shows_empty_when_no_projects", () => {
    vi.mocked(useAppStore).mockReturnValue({
      selectedSolution: mockSolutions[0],
      selectedProject: null,
      selectSolution: mockSelectSolution,
      selectProject: mockSelectProject,
      selectElement: mockSelectElement,
      setRightPanel: mockSetRightPanel,
    } as unknown as ReturnType<typeof useAppStore>);
    vi.mocked(useSolutions).mockReturnValue(
      { data: mockSolutions, isLoading: false, isError: false } as unknown as ReturnType<typeof useSolutions>
    );
    vi.mocked(useSolutionProjects).mockReturnValue(
      { data: [], isLoading: false } as unknown as ReturnType<typeof useSolutionProjects>
    );
    render(<SolutionList />);
    expect(screen.getByText("ファイルなし")).toBeInTheDocument();
  });

  it("project_items_shows_projects_and_click_selects", () => {
    const mockProject = { id: 10, name: "MyDB", fm_version: "19", file_path: "", imported_at: "" };
    vi.mocked(useAppStore).mockReturnValue({
      selectedSolution: mockSolutions[0],
      selectedProject: null,
      selectSolution: mockSelectSolution,
      selectProject: mockSelectProject,
      selectElement: mockSelectElement,
      setRightPanel: mockSetRightPanel,
    } as unknown as ReturnType<typeof useAppStore>);
    vi.mocked(useSolutions).mockReturnValue(
      { data: mockSolutions, isLoading: false, isError: false } as unknown as ReturnType<typeof useSolutions>
    );
    vi.mocked(useSolutionProjects).mockReturnValue(
      { data: [mockProject], isLoading: false } as unknown as ReturnType<typeof useSolutionProjects>
    );
    render(<SolutionList />);
    expect(screen.getByText(/MyDB/)).toBeInTheDocument();
    const projectRow = screen.getByText(/MyDB/).closest("div[class*='cursor-pointer']")!;
    fireEvent.click(projectRow);
    expect(mockSelectProject).toHaveBeenCalledWith(mockProject);
  });
});
