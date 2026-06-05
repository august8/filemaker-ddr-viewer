import { describe, it, expect, beforeEach } from "vitest";
import { useAppStore } from "../../stores/appStore";

beforeEach(() => {
  useAppStore.setState({
    rightPanel: null,
    rightPanel2: null,
  } as Parameters<typeof useAppStore.setState>[0]);
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
