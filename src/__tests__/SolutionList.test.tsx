import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { SolutionList } from "../components/SolutionList";
import type { SolutionRow } from "../types/ddr";

vi.mock("../hooks/useTauriCommand", () => ({
  useSolutions: vi.fn(),
  useDeleteSolution: vi.fn(),
  useSolutionProjects: vi.fn(),
  useDeleteProject: vi.fn(() => ({ mutate: vi.fn() })),
}));

vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

import { useSolutions, useDeleteSolution, useSolutionProjects } from "../hooks/useTauriCommand";
import { useAppStore } from "../stores/appStore";

const mockSolutions: SolutionRow[] = [
  { id: 1, name: "Solution A", summary_path: null, imported_at: "2024-01-01T00:00:00Z" },
  { id: 2, name: "Solution B", summary_path: null, imported_at: "2024-06-15T00:00:00Z" },
];

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useAppStore).mockReturnValue({
    selectedSolution: null,
    selectedProject: null,
    selectSolution: vi.fn(),
    selectProject: vi.fn(),
    setRightPanel: vi.fn(),
  } as unknown as ReturnType<typeof useAppStore>);
  vi.mocked(useDeleteSolution).mockReturnValue(
    { mutate: vi.fn(), isPending: false } as unknown as ReturnType<typeof useDeleteSolution>
  );
  vi.mocked(useSolutionProjects).mockReturnValue(
    { data: [], isLoading: false } as unknown as ReturnType<typeof useSolutionProjects>
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
    // ×ボタンをクリック → 確認UIが出る
    const deleteButtons = screen.getAllByRole("button", { name: "削除" });
    fireEvent.click(deleteButtons[0]);
    // 確認ダイアログの「削除」テキストボタンをクリック（aria-labelなし、テキスト「削除」）
    const confirmBtn = screen.getByText("削除", { selector: "button:not([aria-label])" });
    fireEvent.click(confirmBtn);
    expect(mutateMock).toHaveBeenCalledWith(1, expect.objectContaining({ onSettled: expect.any(Function) }));
  });
});
