import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { LayoutObjectDetail } from "../../components/detail/LayoutObjectDetail";
import { makeLayoutObjectRow, makeLayoutRow } from "../testFixtures";

vi.mock("../../hooks/layout", () => ({
  useLayoutObjects: vi.fn(),
  useLayoutObjectConditions: vi.fn(() => ({ data: [] })),
  useLayoutList: vi.fn(),
}));
vi.mock("../../hooks/fieldRefs", () => ({
  useResolveLayoutField: vi.fn(() => ({ data: null })),
}));

vi.mock("../../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

import { useLayoutObjects, useLayoutObjectConditions, useLayoutList } from "../../hooks/layout";
import { useResolveLayoutField } from "../../hooks/fieldRefs";
import { useAppStore } from "../../stores/appStore";

const baseObj = makeLayoutObjectRow({
  id: 1, object_key: 100,
  field_table_occurrence: "Contact", field_name: "Name",
  bound_top: 100, bound_left: 50, bound_bottom: 120, bound_right: 200,
});

const compareObj = {
  ...baseObj,
  id: 99,  // 別 DB のオブジェクトなので id は異なる
};

const mockLayout = makeLayoutRow({ id: 10, fm_id: 1, name: "ContactLayout", table_occurrence_name: "Contact" });

const mockSetRightPanel = vi.fn();

function setupStore(diffContext: { compareProjectId: number } | null) {
  vi.mocked(useAppStore).mockReturnValue({
    selectedProject: { id: 1, name: "ProjectA", file_path: null, fm_version: "19", imported_at: "" },
    selectedElement: { kind: "layout" as const, projectId: 1, id: 10, name: "ContactLayout" },
    diffContext,
    setRightPanel: mockSetRightPanel,
  } as unknown as ReturnType<typeof useAppStore>);
}

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useLayoutObjects).mockImplementation((layoutId) => {
    if (layoutId === 10) {
      return { data: [baseObj], isLoading: false } as unknown as ReturnType<typeof useLayoutObjects>;
    }
    if (layoutId === 20) {
      return { data: [compareObj], isLoading: false } as unknown as ReturnType<typeof useLayoutObjects>;
    }
    return { data: [], isLoading: false } as unknown as ReturnType<typeof useLayoutObjects>;
  });
  vi.mocked(useLayoutObjectConditions).mockReturnValue(
    { data: [] } as unknown as ReturnType<typeof useLayoutObjectConditions>
  );
  vi.mocked(useLayoutList).mockReturnValue(
    { data: [] } as unknown as ReturnType<typeof useLayoutList>
  );
});

describe("LayoutObjectDetail", () => {
  it("shows_no_diff_section_without_diff_context", () => {
    setupStore(null);
    render(<LayoutObjectDetail layoutObjectId={1} layoutId={10} />);
    expect(screen.queryByText("変更点")).not.toBeInTheDocument();
  });

  it("shows_no_compare_object_message_when_object_not_in_compare", () => {
    setupStore({ compareProjectId: 2 });
    // 比較プロジェクトのレイアウト一覧: 同名レイアウトあり、でも object_key が異なるオブジェクト
    vi.mocked(useLayoutList).mockReturnValue(
      { data: [{ ...mockLayout, id: 20 }] } as unknown as ReturnType<typeof useLayoutList>
    );
    vi.mocked(useLayoutObjects).mockImplementation((layoutId) => {
      if (layoutId === 10) return { data: [baseObj], isLoading: false } as unknown as ReturnType<typeof useLayoutObjects>;
      if (layoutId === 20) return { data: [{ ...compareObj, object_key: 999 }], isLoading: false } as unknown as ReturnType<typeof useLayoutObjects>;
      return { data: [], isLoading: false } as unknown as ReturnType<typeof useLayoutObjects>;
    });
    render(<LayoutObjectDetail layoutObjectId={1} layoutId={10} />);
    expect(screen.getByText("変更点")).toBeInTheDocument();
    expect(screen.getByText(/比較対象なし/)).toBeInTheDocument();
  });

  it("shows_no_changes_message_when_object_identical", () => {
    setupStore({ compareProjectId: 2 });
    vi.mocked(useLayoutList).mockReturnValue(
      { data: [{ ...mockLayout, id: 20 }] } as unknown as ReturnType<typeof useLayoutList>
    );
    // 全フィールド同一の compareObj
    render(<LayoutObjectDetail layoutObjectId={1} layoutId={10} />);
    expect(screen.getByText("変更点")).toBeInTheDocument();
    expect(screen.getByText("変更点なし")).toBeInTheDocument();
  });

  it("shows_position_change_when_bounds_differ", () => {
    setupStore({ compareProjectId: 2 });
    vi.mocked(useLayoutList).mockReturnValue(
      { data: [{ ...mockLayout, id: 20 }] } as unknown as ReturnType<typeof useLayoutList>
    );
    vi.mocked(useLayoutObjects).mockImplementation((layoutId) => {
      if (layoutId === 10) return { data: [baseObj], isLoading: false } as unknown as ReturnType<typeof useLayoutObjects>;
      if (layoutId === 20) return {
        data: [{ ...compareObj, bound_top: 200, bound_left: 100 }],
        isLoading: false,
      } as unknown as ReturnType<typeof useLayoutObjects>;
      return { data: [], isLoading: false } as unknown as ReturnType<typeof useLayoutObjects>;
    });
    render(<LayoutObjectDetail layoutObjectId={1} layoutId={10} />);
    expect(screen.getByText("変更点")).toBeInTheDocument();
    expect(screen.getByText(/位置/)).toBeInTheDocument();
  });

  it("field_click_passes_field_project_id_to_set_right_panel", async () => {
    const { fireEvent } = await import("@testing-library/react");
    setupStore(null);
    vi.mocked(useResolveLayoutField).mockReturnValue({
      data: { table_id: 5, field_id: 42, table_name: "Customer", field_project_id: 99 },
    } as unknown as ReturnType<typeof useResolveLayoutField>);
    render(<LayoutObjectDetail layoutObjectId={1} layoutId={10} />);
    const btn = screen.getByRole("button", { name: /Contact::Name/ });
    fireEvent.click(btn);
    expect(mockSetRightPanel).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: "field",
        fieldProjectId: 99,
        tableId: 5,
        fieldId: 42,
      })
    );
  });

  it("shows_tooltip_change_when_tooltip_differs", () => {
    setupStore({ compareProjectId: 2 });
    vi.mocked(useLayoutList).mockReturnValue(
      { data: [{ ...mockLayout, id: 20 }] } as unknown as ReturnType<typeof useLayoutList>
    );
    vi.mocked(useLayoutObjects).mockImplementation((layoutId) => {
      if (layoutId === 10) return { data: [{ ...baseObj, tooltip: "新しいツールチップ" }], isLoading: false } as unknown as ReturnType<typeof useLayoutObjects>;
      if (layoutId === 20) return { data: [{ ...compareObj, tooltip: "古いツールチップ" }], isLoading: false } as unknown as ReturnType<typeof useLayoutObjects>;
      return { data: [], isLoading: false } as unknown as ReturnType<typeof useLayoutObjects>;
    });
    render(<LayoutObjectDetail layoutObjectId={1} layoutId={10} />);
    expect(screen.getByText("変更点")).toBeInTheDocument();
    // 変更点セクション内に「ツールチップ」ラベルが複数現れる（元のセクション + diff 行）
    expect(screen.getAllByText(/ツールチップ/).length).toBeGreaterThanOrEqual(2);
  });
});
