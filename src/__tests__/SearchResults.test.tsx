import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { SearchResults } from "../components/SearchResults";
import type { SearchResult } from "../types/ddr";

vi.mock("../hooks/useTauriCommand", () => ({
  useSearch: vi.fn(),
}));

vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(() => ({
    selectElement: vi.fn(),
    setRightPanel: vi.fn(),
    searchContains: false,
    searchScope: "all" as const,
    selectedProject: null,
    selectedSolution: null,
    setSearchDuration: vi.fn(),
  })),
}));

import { useSearch } from "../hooks/useTauriCommand";

const mockResults: SearchResult[] = [
  { project_id: 1, element_type: "script", element_id: 1, name: "My Script", snippet: "some snippet", rank: 1.0, parent_id: null, parent_name: null },
  { project_id: 1, element_type: "field", element_id: 2, name: "My Field", snippet: "", rank: 0.9, parent_id: 10, parent_name: "Contact" },
  { project_id: 1, element_type: "layout", element_id: 3, name: "My Layout", snippet: "", rank: 0.8, parent_id: null, parent_name: null },
];

describe("SearchResults", () => {
  it("renders_nothing_when_query_empty", () => {
    vi.mocked(useSearch).mockReturnValue(
      { data: undefined, isLoading: false } as unknown as ReturnType<typeof useSearch>
    );
    const { container } = render(<SearchResults query="" />);
    expect(container.firstChild).toBeNull();
  });

  it("renders_results", () => {
    vi.mocked(useSearch).mockReturnValue(
      { data: mockResults, isLoading: false } as unknown as ReturnType<typeof useSearch>
    );
    render(<SearchResults query="test" />);
    expect(screen.getByText("My Script")).toBeInTheDocument();
    expect(screen.getByText("My Field")).toBeInTheDocument();
    expect(screen.getByText("My Layout")).toBeInTheDocument();
  });

  it("renders_no_results_message", () => {
    vi.mocked(useSearch).mockReturnValue(
      { data: [], isLoading: false } as unknown as ReturnType<typeof useSearch>
    );
    render(<SearchResults query="notfound" />);
    expect(screen.getByText("見つかりませんでした")).toBeInTheDocument();
  });

  it("filter_shows_only_scripts_when_script_filter_clicked", async () => {
    vi.mocked(useSearch).mockReturnValue(
      { data: mockResults, isLoading: false } as unknown as ReturnType<typeof useSearch>
    );
    const user = userEvent.setup();
    render(<SearchResults query="test" />);

    // フィルター前: 全件表示
    expect(screen.getByText("My Script")).toBeInTheDocument();
    expect(screen.getByText("My Field")).toBeInTheDocument();
    expect(screen.getByText("My Layout")).toBeInTheDocument();

    // スクリプトフィルターボタンをクリック（exact matchで結果行バッジと区別）
    const scriptFilterBtn = screen.getByRole("button", { name: "スクリプト (1)" });
    await user.click(scriptFilterBtn);

    // スクリプトのみ表示
    expect(screen.getByText("My Script")).toBeInTheDocument();
    // フィールドとレイアウトは非表示
    expect(screen.queryByText("My Field")).not.toBeInTheDocument();
    expect(screen.queryByText("My Layout")).not.toBeInTheDocument();
  });

  it("filter_resets_to_all_when_same_filter_clicked_twice", async () => {
    vi.mocked(useSearch).mockReturnValue(
      { data: mockResults, isLoading: false } as unknown as ReturnType<typeof useSearch>
    );
    const user = userEvent.setup();
    render(<SearchResults query="test" />);

    const scriptFilterBtn = screen.getByRole("button", { name: "スクリプト (1)" });
    await user.click(scriptFilterBtn); // 絞り込み
    await user.click(scriptFilterBtn); // 解除

    // 全件に戻る
    expect(screen.getByText("My Script")).toBeInTheDocument();
    expect(screen.getByText("My Field")).toBeInTheDocument();
    expect(screen.getByText("My Layout")).toBeInTheDocument();
  });

  it("click_script_result_calls_selectElement", async () => {
    const selectElement = vi.fn();
    vi.mocked(useSearch).mockReturnValue(
      { data: mockResults, isLoading: false } as unknown as ReturnType<typeof useSearch>
    );
    const { useAppStore } = await import("../stores/appStore");
    vi.mocked(useAppStore).mockReturnValue({
      selectElement,
      setRightPanel: vi.fn(),
    } as unknown as ReturnType<typeof useAppStore>);

    const user = userEvent.setup();
    render(<SearchResults query="test" />);

    await user.click(screen.getByText("My Script"));
    expect(selectElement).toHaveBeenCalledWith({
      kind: "script",
      projectId: 1,
      id: 1,
      name: "My Script",
    });
  });

  it("highlight_marks_query_word_in_name", () => {
    vi.mocked(useSearch).mockReturnValue(
      { data: [{ project_id: 1, element_type: "script", element_id: 1, name: "My Script", snippet: "", rank: 1.0, parent_id: null, parent_name: null }], isLoading: false } as unknown as ReturnType<typeof useSearch>
    );
    const { container } = render(<SearchResults query="Script" />);
    const marks = container.querySelectorAll("mark");
    expect(marks.length).toBeGreaterThan(0);
    expect(Array.from(marks).some(m => /script/i.test(m.textContent ?? ""))).toBe(true);
  });

  it("highlight_is_case_insensitive", () => {
    vi.mocked(useSearch).mockReturnValue(
      { data: [{ project_id: 1, element_type: "script", element_id: 1, name: "My Script", snippet: "", rank: 1.0, parent_id: null, parent_name: null }], isLoading: false } as unknown as ReturnType<typeof useSearch>
    );
    const { container } = render(<SearchResults query="script" />);
    const marks = container.querySelectorAll("mark");
    expect(marks.length).toBeGreaterThan(0);
    expect(marks[0].textContent?.toLowerCase()).toBe("script");
  });

  it("highlight_marks_snippet", () => {
    vi.mocked(useSearch).mockReturnValue(
      { data: [{ project_id: 1, element_type: "script", element_id: 1, name: "Script", snippet: 'スクリプト実行 ["Sub Script"]', rank: 1.0, parent_id: null, parent_name: null }], isLoading: false } as unknown as ReturnType<typeof useSearch>
    );
    const { container } = render(<SearchResults query="Sub" />);
    const marks = container.querySelectorAll("mark");
    expect(marks.length).toBeGreaterThan(0);
    expect(Array.from(marks).some(m => /sub/i.test(m.textContent ?? ""))).toBe(true);
  });

  it("highlight_handles_special_regex_chars_without_error", () => {
    vi.mocked(useSearch).mockReturnValue(
      { data: [{ project_id: 1, element_type: "field", element_id: 1, name: "C++ Field", snippet: "", rank: 1.0, parent_id: null, parent_name: null }], isLoading: false } as unknown as ReturnType<typeof useSearch>
    );
    expect(() => render(<SearchResults query="C++" />)).not.toThrow();
    expect(screen.getByText(/C/)).toBeInTheDocument();
  });

  it("click_field_result_calls_selectElement_and_setRightPanel", async () => {
    const selectElement = vi.fn();
    const setRightPanel = vi.fn();
    vi.mocked(useSearch).mockReturnValue(
      { data: mockResults, isLoading: false } as unknown as ReturnType<typeof useSearch>
    );
    const { useAppStore } = await import("../stores/appStore");
    vi.mocked(useAppStore).mockReturnValue({
      selectElement,
      setRightPanel,
    } as unknown as ReturnType<typeof useAppStore>);

    const user = userEvent.setup();
    render(<SearchResults query="test" />);

    await user.click(screen.getByText("My Field"));
    expect(selectElement).toHaveBeenCalledWith({
      kind: "table",
      projectId: 1,
      id: 10,
      name: "Contact",
    });
    expect(setRightPanel).toHaveBeenCalledWith({
      kind: "field",
      projectId: 1,
      tableId: 10,
      fieldId: 2,
      tableName: "Contact",
    });
  });
});
