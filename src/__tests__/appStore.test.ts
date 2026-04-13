import { beforeEach, describe, expect, it } from "vitest";
import { useAppStore } from "../stores/appStore";
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

  it("selectSolution_resets_project_and_element", () => {
    // setup with project and element selected
    useAppStore.setState({
      selectedProject: mockProject,
      selectedElement: { kind: "dashboard" },
    });
    useAppStore.getState().selectSolution(mockSolution);
    const state = useAppStore.getState();
    expect(state.selectedSolution).toEqual(mockSolution);
    expect(state.selectedProject).toBeNull();
    expect(state.selectedElement).toBeNull();
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
});
