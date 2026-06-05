import { describe, it, expect, beforeEach } from "vitest";
import { useAppStore } from "../../stores/appStore";
import type { SelectedElement } from "../../stores/appStore";

beforeEach(() => {
  useAppStore.setState({
    rightPanel: null,
    rightPanel2: null,
  } as Parameters<typeof useAppStore.setState>[0]);
});

const panel1 = { kind: "layout_object" as const, layoutObjectId: 10, layoutId: 1 };
const panel2 = { kind: "field" as const, fieldId: 5, tableId: 6, projectId: 7, tableName: "X" };
const mockProject = { id: 1, name: "P", file_path: null, fm_version: "19", imported_at: "" };
const mockSolution = { id: 1, name: "S", summary_path: null, imported_at: "" };

describe("appStore - パネル自動クリア（メイン遷移時）", () => {
  beforeEach(() => {
    useAppStore.setState({
      rightPanel: panel1,
      rightPanel2: panel2,
      navHistory: [] as SelectedElement[],
      navIndex: -1,
      selectedElement: null,
      searchQuery: "",
    } as Parameters<typeof useAppStore.setState>[0]);
  });

  it("selectElement: Panel 2 が開いているとき Panel 2 が Panel 1 に昇格し Panel 2 が null になる", () => {
    useAppStore.getState().selectElement({ kind: "script", projectId: 1, id: 1, name: "Script" });
    expect(useAppStore.getState().rightPanel).toEqual(panel2);
    expect(useAppStore.getState().rightPanel2).toBeNull();
  });

  it("selectElement: Panel 1 のみ開いているとき Panel 1 が null になる", () => {
    useAppStore.setState({ rightPanel2: null } as Parameters<typeof useAppStore.setState>[0]);
    useAppStore.getState().selectElement({ kind: "script", projectId: 1, id: 1, name: "Script" });
    expect(useAppStore.getState().rightPanel).toBeNull();
    expect(useAppStore.getState().rightPanel2).toBeNull();
  });

  it("selectElement: パネルなしのとき rightPanel は null のまま", () => {
    useAppStore.setState({ rightPanel: null, rightPanel2: null } as Parameters<typeof useAppStore.setState>[0]);
    useAppStore.getState().selectElement({ kind: "script", projectId: 1, id: 1, name: "Script" });
    expect(useAppStore.getState().rightPanel).toBeNull();
    expect(useAppStore.getState().rightPanel2).toBeNull();
  });

  it("navigateFromDiff: Panel 2 が開いているとき Panel 2 が Panel 1 に昇格する", () => {
    useAppStore.getState().navigateFromDiff({ kind: "script", projectId: 1, id: 1, name: "Script" }, 2);
    expect(useAppStore.getState().rightPanel).toEqual(panel2);
    expect(useAppStore.getState().rightPanel2).toBeNull();
  });

  it("selectProject: 両パネルが閉じる", () => {
    useAppStore.getState().selectProject(mockProject);
    expect(useAppStore.getState().rightPanel).toBeNull();
    expect(useAppStore.getState().rightPanel2).toBeNull();
  });

  it("selectSolution: 両パネルが閉じる", () => {
    useAppStore.getState().selectSolution(mockSolution);
    expect(useAppStore.getState().rightPanel).toBeNull();
    expect(useAppStore.getState().rightPanel2).toBeNull();
  });

  it("navigateToProject: 両パネルが閉じる", () => {
    useAppStore.getState().navigateToProject(mockProject);
    expect(useAppStore.getState().rightPanel).toBeNull();
    expect(useAppStore.getState().rightPanel2).toBeNull();
  });
});

describe("appStore - setRightPanel", () => {
  it("setRightPanel が rightPanel2 を null にクリアする", () => {
    useAppStore.setState({
      rightPanel2: { kind: "field", fieldId: 1, tableId: 2, projectId: 3, tableName: "T" },
    } as Parameters<typeof useAppStore.setState>[0]);

    useAppStore.getState().setRightPanel({ kind: "layout_object", layoutObjectId: 10, layoutId: 1 });

    expect(useAppStore.getState().rightPanel2).toBeNull();
  });

  it("setRightPanel(null) でパネル1・パネル2 両方がクリアされる", () => {
    useAppStore.setState({
      rightPanel: { kind: "layout_object", layoutObjectId: 10, layoutId: 1 },
      rightPanel2: { kind: "field", fieldId: 1, tableId: 2, projectId: 3, tableName: "T" },
    } as Parameters<typeof useAppStore.setState>[0]);

    useAppStore.getState().setRightPanel(null);

    expect(useAppStore.getState().rightPanel).toBeNull();
    expect(useAppStore.getState().rightPanel2).toBeNull();
  });

  it("setRightPanel2 が rightPanel2 のみ更新する", () => {
    const panel1 = { kind: "layout_object" as const, layoutObjectId: 10, layoutId: 1 };
    useAppStore.setState({ rightPanel: panel1 } as Parameters<typeof useAppStore.setState>[0]);

    useAppStore.getState().setRightPanel2({ kind: "field", fieldId: 5, tableId: 6, projectId: 7, tableName: "X" });

    expect(useAppStore.getState().rightPanel).toEqual(panel1);
    expect(useAppStore.getState().rightPanel2).toEqual(
      expect.objectContaining({ kind: "field", fieldId: 5 })
    );
  });
});
