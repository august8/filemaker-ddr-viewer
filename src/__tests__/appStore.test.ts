import { beforeEach, describe, expect, it, vi } from "vitest";
import { useAppStore } from "../stores/appStore";
// elementKey は内部関数のため、selectElement の挙動を通じて間接テストする
import type { SolutionRow, ProjectRow } from "../types/ddr";

const mockSolution: SolutionRow = {
  id: 1,
  name: "Test Solution",
  summary_path: "/path/to/summary.xml",
  imported_at: "2024-01-01T00:00:00Z",
};

const mockProject: ProjectRow = {
  id: 10,
  name: "Test Project",
  file_path: "/path/to/file.fmp12",
  fm_version: "19",
  imported_at: "2024-01-01T00:00:00Z",
};

beforeEach(() => {
  useAppStore.setState({
    solutions: [],
    selectedSolution: null,
    selectedProject: null,
    selectedElement: null,
    searchQuery: "",
  });
});

describe("appStore", () => {
  it("initial_state_is_empty", () => {
    const state = useAppStore.getState();
    expect(state.solutions).toEqual([]);
    expect(state.selectedSolution).toBeNull();
    expect(state.selectedProject).toBeNull();
    expect(state.selectedElement).toBeNull();
    expect(state.searchQuery).toBe("");
  });

  it("setSolutions_updates_state", () => {
    useAppStore.getState().setSolutions([mockSolution]);
    expect(useAppStore.getState().solutions).toEqual([mockSolution]);
  });

  it("selectSolution_resets_project_and_sets_solution_dashboard_element", () => {
    // setup with project and element selected
    useAppStore.setState({
      selectedProject: mockProject,
      selectedElement: { kind: "dashboard" },
    });
    useAppStore.getState().selectSolution(mockSolution);
    const state = useAppStore.getState();
    expect(state.selectedSolution).toEqual(mockSolution);
    expect(state.selectedProject).toBeNull();
    expect(state.selectedElement).toEqual({ kind: "solution_dashboard", solutionId: mockSolution.id });
    expect(state.navHistory).toEqual([{ kind: "solution_dashboard", solutionId: mockSolution.id }]);
    expect(state.navIndex).toBe(0);
  });

  it("selectProject_resets_element", () => {
    useAppStore.setState({
      selectedElement: { kind: "dashboard" },
    });
    useAppStore.getState().selectProject(mockProject);
    const state = useAppStore.getState();
    expect(state.selectedProject).toEqual(mockProject);
    expect(state.selectedElement).toBeNull();
  });

  it("selectProject_preserves_navHistory", () => {
    useAppStore.setState({
      navHistory: [{ kind: "solution_dashboard", solutionId: 1 }],
      navIndex: 0,
    });
    useAppStore.getState().selectProject(mockProject);
    const state = useAppStore.getState();
    expect(state.selectedProject).toEqual(mockProject);
    expect(state.navHistory).toHaveLength(1); // 履歴は保持される
  });

  it("navigateToProject_pushes_dashboard_entry_to_history", () => {
    useAppStore.setState({
      navHistory: [{ kind: "solution_dashboard", solutionId: 1 }],
      navIndex: 0,
      selectedElement: { kind: "solution_dashboard", solutionId: 1 },
    });
    useAppStore.getState().navigateToProject(mockProject);
    const state = useAppStore.getState();
    expect(state.selectedProject).toEqual(mockProject);
    expect(state.selectedElement).toBeNull();
    expect(state.navHistory).toEqual([
      { kind: "solution_dashboard", solutionId: 1 },
      { kind: "dashboard" },
    ]);
    expect(state.navIndex).toBe(1);
  });

  it("navigateToProject_allows_navigateBack_to_solution_dashboard", () => {
    useAppStore.setState({
      navHistory: [{ kind: "solution_dashboard", solutionId: 1 }],
      navIndex: 0,
      selectedElement: { kind: "solution_dashboard", solutionId: 1 },
    });
    useAppStore.getState().navigateToProject(mockProject);
    useAppStore.getState().navigateBack();
    const state = useAppStore.getState();
    expect(state.selectedElement).toEqual({ kind: "solution_dashboard", solutionId: 1 });
  });

  it("navigateBack_resets_selectedProject_when_landing_on_solution_dashboard", () => {
    useAppStore.setState({
      navHistory: [{ kind: "solution_dashboard", solutionId: 1 }, { kind: "dashboard" }],
      navIndex: 1,
      selectedElement: null,
      selectedProject: mockProject,
    });
    useAppStore.getState().navigateBack();
    const state = useAppStore.getState();
    expect(state.selectedElement).toEqual({ kind: "solution_dashboard", solutionId: 1 });
    expect(state.selectedProject).toBeNull();
  });

  it("navigateForward_resets_selectedProject_when_landing_on_solution_dashboard", () => {
    useAppStore.setState({
      navHistory: [{ kind: "dashboard" }, { kind: "solution_dashboard", solutionId: 3 }],
      navIndex: 0,
      selectedElement: null,
      selectedProject: mockProject,
    });
    useAppStore.getState().navigateForward();
    const state = useAppStore.getState();
    expect(state.selectedElement).toEqual({ kind: "solution_dashboard", solutionId: 3 });
    expect(state.selectedProject).toBeNull();
  });

  it("purgeProjectFromHistory_removes_related_entries", () => {
    useAppStore.setState({
      navHistory: [
        { kind: "solution_dashboard", solutionId: 1 },
        { kind: "all_scripts", projectId: 10 },
        { kind: "all_tables", projectId: 20 },
      ],
      navIndex: 2,
    });
    useAppStore.getState().purgeProjectFromHistory(10);
    const state = useAppStore.getState();
    expect(state.navHistory).toEqual([
      { kind: "solution_dashboard", solutionId: 1 },
      { kind: "all_tables", projectId: 20 },
    ]);
    expect(state.navIndex).toBe(1);
  });

  it("purgeProjectFromHistory_resets_index_when_current_entry_removed", () => {
    useAppStore.setState({
      navHistory: [
        { kind: "solution_dashboard", solutionId: 1 },
        { kind: "all_scripts", projectId: 10 },
      ],
      navIndex: 1,
      selectedElement: { kind: "all_scripts", projectId: 10 },
    });
    useAppStore.getState().purgeProjectFromHistory(10);
    const state = useAppStore.getState();
    expect(state.navHistory).toEqual([{ kind: "solution_dashboard", solutionId: 1 }]);
    expect(state.navIndex).toBe(0);
  });

  it("setSearchQuery_updates_query", () => {
    useAppStore.getState().setSearchQuery("hello");
    expect(useAppStore.getState().searchQuery).toBe("hello");
  });

  it("selectSolution_resets_searchScope", () => {
    useAppStore.setState({ searchScope: "project" });
    useAppStore.getState().selectSolution(mockSolution);
    expect(useAppStore.getState().searchScope).toBe("all");
  });

  it("selectElement_same_element_clears_searchQuery", () => {
    const element = { kind: "script" as const, id: 1, name: "MyScript", projectId: 10 };
    // navHistory に element が積まれており searchQuery が非空の状態を再現
    useAppStore.setState({
      searchQuery: "test",
      navHistory: [element],
      navIndex: 0,
    });
    // 同じ要素を再度 selectElement → early return が発動
    useAppStore.getState().selectElement(element);
    expect(useAppStore.getState().searchQuery).toBe("");
  });

  it("selectElement_works_when_nav_history_is_empty", () => {
    // navHistory が空（navIndex === -1）の初期状態から selectElement が正常に動作すること
    useAppStore.setState({ navHistory: [], navIndex: -1, selectedElement: null });
    const el = { kind: "script" as const, id: 1, name: "S", projectId: 10 };
    useAppStore.getState().selectElement(el);
    expect(useAppStore.getState().selectedElement).toEqual(el);
    expect(useAppStore.getState().navHistory.length).toBeGreaterThan(0);
  });

  it("selectElement_same_kind_different_id_adds_to_history", () => {
    const el1 = { kind: "script" as const, id: 1, name: "Script1", projectId: 10 };
    const el2 = { kind: "script" as const, id: 2, name: "Script2", projectId: 10 };
    // selectedElement も設定して「el1 を表示中」の状態を再現
    useAppStore.setState({ selectedElement: el1, navHistory: [el1], navIndex: 0 });
    useAppStore.getState().selectElement(el2);
    // 別の id → 新規エントリとして履歴に積まれる
    expect(useAppStore.getState().navHistory.length).toBe(2);
    expect(useAppStore.getState().selectedElement).toEqual(el2);
  });

  it("selectElement_same_element_does_not_grow_history", () => {
    const el = { kind: "table" as const, id: 5, name: "Contacts", projectId: 10 };
    useAppStore.setState({ navHistory: [el], navIndex: 0 });
    const before = useAppStore.getState().navHistory.length;
    useAppStore.getState().selectElement(el);
    // 同じ要素 → 履歴は増えない
    expect(useAppStore.getState().navHistory.length).toBe(before);
  });

  it("selectElement_null_dashboard_entry_is_not_duplicated", () => {
    // selectedElement === null（ダッシュボード）の状態から別要素を選択
    useAppStore.setState({ selectedElement: null, navHistory: [{ kind: "dashboard" }], navIndex: 0 });
    const el = { kind: "script" as const, id: 1, name: "S", projectId: 10 };
    useAppStore.getState().selectElement(el);
    const history = useAppStore.getState().navHistory;
    // dashboard が重複して挿入されていないこと
    expect(history.filter((h) => h?.kind === "dashboard").length).toBe(1);
  });
});

// ---------------------------------------------------------------------------
// navigateBack / navigateForward
// ---------------------------------------------------------------------------
describe("navigation", () => {
  const el1 = { kind: "script" as const, id: 1, name: "S1", projectId: 10 };
  const el2 = { kind: "script" as const, id: 2, name: "S2", projectId: 10 };
  const el3 = { kind: "table" as const, id: 3, name: "T1", projectId: 10 };

  beforeEach(() => {
    useAppStore.setState({ navHistory: [], navIndex: -1, selectedElement: null, searchQuery: "" });
  });

  it("navigateBack_does_nothing_when_history_is_empty", () => {
    useAppStore.getState().navigateBack();
    expect(useAppStore.getState().navIndex).toBe(-1);
  });

  it("navigateBack_does_nothing_when_at_first_entry", () => {
    useAppStore.setState({ navHistory: [el1], navIndex: 0, selectedElement: el1 });
    useAppStore.getState().navigateBack();
    expect(useAppStore.getState().navIndex).toBe(0);
    expect(useAppStore.getState().selectedElement).toEqual(el1);
  });

  it("navigateBack_moves_to_previous_entry", () => {
    useAppStore.setState({ navHistory: [el1, el2, el3], navIndex: 2, selectedElement: el3 });
    useAppStore.getState().navigateBack();
    expect(useAppStore.getState().navIndex).toBe(1);
    expect(useAppStore.getState().selectedElement).toEqual(el2);
  });

  it("navigateBack_restores_searchQuery_for_search_entry", () => {
    const searchEl = { kind: "search" as const, query: "hello" };
    useAppStore.setState({ navHistory: [searchEl, el1], navIndex: 1, selectedElement: el1 });
    useAppStore.getState().navigateBack();
    expect(useAppStore.getState().searchQuery).toBe("hello");
    expect(useAppStore.getState().selectedElement).toEqual(searchEl);
  });

  it("navigateForward_does_nothing_when_at_last_entry", () => {
    useAppStore.setState({ navHistory: [el1, el2], navIndex: 1, selectedElement: el2 });
    useAppStore.getState().navigateForward();
    expect(useAppStore.getState().navIndex).toBe(1);
  });

  it("navigateForward_moves_to_next_entry", () => {
    useAppStore.setState({ navHistory: [el1, el2, el3], navIndex: 0, selectedElement: el1 });
    useAppStore.getState().navigateForward();
    expect(useAppStore.getState().navIndex).toBe(1);
    expect(useAppStore.getState().selectedElement).toEqual(el2);
  });

  it("navigateBack_clears_diffContext", () => {
    useAppStore.setState({
      navHistory: [el1, el2],
      navIndex: 1,
      selectedElement: el2,
      diffContext: { compareProjectId: 5 },
    });
    useAppStore.getState().navigateBack();
    expect(useAppStore.getState().diffContext).toBeNull();
  });
});

// ---------------------------------------------------------------------------
// font size
// ---------------------------------------------------------------------------
describe("fontSize", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    useAppStore.setState({ fontSize: 14 });
  });

  it("stepFontSize_increases_by_step", () => {
    useAppStore.getState().stepFontSize(2);
    expect(useAppStore.getState().fontSize).toBe(16);
  });

  it("stepFontSize_decreases_by_step", () => {
    useAppStore.getState().stepFontSize(-2);
    expect(useAppStore.getState().fontSize).toBe(12);
  });

  it("stepFontSize_zero_resets_to_default", () => {
    useAppStore.setState({ fontSize: 20 });
    useAppStore.getState().stepFontSize(0);
    expect(useAppStore.getState().fontSize).toBe(14);
  });

  it("stepFontSize_clamps_at_max", () => {
    useAppStore.setState({ fontSize: 23 });
    useAppStore.getState().stepFontSize(5);
    expect(useAppStore.getState().fontSize).toBe(24);
  });

  it("stepFontSize_clamps_at_min", () => {
    useAppStore.setState({ fontSize: 11 });
    useAppStore.getState().stepFontSize(-5);
    expect(useAppStore.getState().fontSize).toBe(10);
  });

  it("setFontSize_clamps_above_max", () => {
    useAppStore.getState().setFontSize(99);
    expect(useAppStore.getState().fontSize).toBe(24);
  });

  it("setFontSize_clamps_below_min", () => {
    useAppStore.getState().setFontSize(1);
    expect(useAppStore.getState().fontSize).toBe(10);
  });

  it("setFontSize_saves_to_localStorage", () => {
    const spy = vi.spyOn(Storage.prototype, "setItem");
    useAppStore.getState().setFontSize(16);
    expect(spy).toHaveBeenCalledWith("fm-ddr-font-size", "16");
  });
});

// ---------------------------------------------------------------------------
// navigateFromDiff
// ---------------------------------------------------------------------------
describe("navigateFromDiff", () => {
  beforeEach(() => {
    useAppStore.setState({ navHistory: [], navIndex: -1, selectedElement: null, diffContext: null });
  });

  it("sets_diffContext_with_compareProjectId", () => {
    const el = { kind: "table" as const, id: 1, name: "T", projectId: 10 };
    useAppStore.getState().navigateFromDiff(el, 99);
    expect(useAppStore.getState().diffContext).toEqual({ compareProjectId: 99 });
    expect(useAppStore.getState().selectedElement).toEqual(el);
  });

  it("same_element_updates_diffContext_without_growing_history", () => {
    const el = { kind: "table" as const, id: 1, name: "T", projectId: 10 };
    useAppStore.setState({ navHistory: [el], navIndex: 0, selectedElement: el });
    const before = useAppStore.getState().navHistory.length;
    useAppStore.getState().navigateFromDiff(el, 42);
    expect(useAppStore.getState().navHistory.length).toBe(before);
    expect(useAppStore.getState().diffContext).toEqual({ compareProjectId: 42 });
  });
});

// ---------------------------------------------------------------------------
// setSearchQuery special cases
// ---------------------------------------------------------------------------
describe("setSearchQuery", () => {
  beforeEach(() => {
    useAppStore.setState({ searchQuery: "", selectedElement: null });
  });

  it("clears_selectedElement_when_it_is_search_kind", () => {
    useAppStore.setState({ selectedElement: { kind: "search", query: "old" } });
    useAppStore.getState().setSearchQuery("new");
    expect(useAppStore.getState().selectedElement).toBeNull();
    expect(useAppStore.getState().searchQuery).toBe("new");
  });

  it("clears_selectedElement_when_it_is_dashboard_kind", () => {
    useAppStore.setState({ selectedElement: { kind: "dashboard" } });
    useAppStore.getState().setSearchQuery("test");
    expect(useAppStore.getState().selectedElement).toBeNull();
  });

  it("does_not_clear_selectedElement_for_other_kinds", () => {
    const el = { kind: "script" as const, id: 1, name: "S", projectId: 10 };
    useAppStore.setState({ selectedElement: el });
    useAppStore.getState().setSearchQuery("test");
    expect(useAppStore.getState().selectedElement).toEqual(el);
  });
});
