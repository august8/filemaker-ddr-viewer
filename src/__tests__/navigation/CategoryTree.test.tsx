import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { CategoryTree } from "../../components/navigation/CategoryTree";
import { makeTableRow, makeScriptRow, makeLayoutRow, makeValueListRow, makeCustomFunctionRow } from "../testFixtures";

vi.mock("../../hooks/table", () => ({
  useTableList: vi.fn(),
  useTableOccurrenceList: vi.fn(() => ({ data: [] })),
  useRelationshipList: vi.fn(() => ({ data: [] })),
}));
vi.mock("../../hooks/script", () => ({
  useScriptList: vi.fn(),
}));
vi.mock("../../hooks/layout", () => ({
  useLayoutList: vi.fn(),
}));
vi.mock("../../hooks/catalog", () => ({
  useValueListList: vi.fn(),
  useCustomFunctionList: vi.fn(),
}));

vi.mock("../../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

import { useTableList } from "../../hooks/table";
import { useScriptList } from "../../hooks/script";
import { useLayoutList } from "../../hooks/layout";
import { useValueListList, useCustomFunctionList } from "../../hooks/catalog";
import { useAppStore } from "../../stores/appStore";

const mockTables = [
  makeTableRow({ id: 1, name: "Contact", field_count: 5 }),
  makeTableRow({ id: 2, fm_id: 2, name: "Project", field_count: 3 }),
];

const mockScripts = [
  makeScriptRow({ id: 1, name: "Main Script", step_count: 10 }),
];

const mockLayouts = [
  makeLayoutRow({ id: 1, name: "Contact Layout", table_occurrence_name: "Contact" }),
];

const mockValueLists = [
  makeValueListRow({ id: 1, name: "Status", item_count: 3 }),
];

const mockCustomFunctions = [
  makeCustomFunctionRow({ id: 1, name: "MyFunc", parameters: "p1; p2", calculation: "p1 + p2" }),
];

const mockSelectElement = vi.fn();
const mockSetRightPanel = vi.fn();

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useAppStore).mockReturnValue({
    selectedElement: null,
    selectElement: mockSelectElement,
    setRightPanel: mockSetRightPanel,
  } as unknown as ReturnType<typeof useAppStore>);
  vi.mocked(useTableList).mockReturnValue({
    data: mockTables,
    isLoading: false,
  } as unknown as ReturnType<typeof useTableList>);
  vi.mocked(useScriptList).mockReturnValue({
    data: mockScripts,
    isLoading: false,
  } as unknown as ReturnType<typeof useScriptList>);
  vi.mocked(useLayoutList).mockReturnValue({
    data: mockLayouts,
    isLoading: false,
  } as unknown as ReturnType<typeof useLayoutList>);
  vi.mocked(useValueListList).mockReturnValue({
    data: mockValueLists,
    isLoading: false,
  } as unknown as ReturnType<typeof useValueListList>);
  vi.mocked(useCustomFunctionList).mockReturnValue({
    data: mockCustomFunctions,
    isLoading: false,
  } as unknown as ReturnType<typeof useCustomFunctionList>);
});

describe("CategoryTree", () => {
  it("renders_category_headers", () => {
    render(<CategoryTree projectId={1} />);
    expect(screen.getByText(/^テーブル \(/)).toBeInTheDocument();
    expect(screen.getByText(/スクリプト/)).toBeInTheDocument();
    expect(screen.getByText(/レイアウト/)).toBeInTheDocument();
    expect(screen.getByText(/バリューリスト/)).toBeInTheDocument();
    expect(screen.getByText(/カスタム関数/)).toBeInTheDocument();
    expect(screen.getByText(/テーブルオカレンス/)).toBeInTheDocument();
    expect(screen.getByText(/リレーション \(/)).toBeInTheDocument();
  });

  it("category_click_toggles_items", () => {
    render(<CategoryTree projectId={1} />);
    // 最初は折りたたまれているので要素が見えない
    expect(screen.queryByText("Contact")).not.toBeInTheDocument();

    // テーブルカテゴリをクリック
    const tableHeader = screen.getByText(/^テーブル \(/);
    fireEvent.click(tableHeader);

    // 展開後は要素が見える
    expect(screen.getByText("Contact")).toBeInTheDocument();
    expect(screen.getByText("Project")).toBeInTheDocument();

    // もう一度クリックで折りたたむ
    fireEvent.click(tableHeader);
    expect(screen.queryByText("Contact")).not.toBeInTheDocument();
  });

  it("table_occurrence_button_click_selects_all_table_occurrences", () => {
    render(<CategoryTree projectId={1} />);
    const toButton = screen.getByText(/テーブルオカレンス/);
    fireEvent.click(toButton);
    expect(mockSelectElement).toHaveBeenCalledWith({
      kind: "all_table_occurrences",
      projectId: 1,
    });
  });

  it("relationship_button_click_selects_all_relationships", () => {
    render(<CategoryTree projectId={1} />);
    const relButton = screen.getByText(/^リレーション \(/);
    fireEvent.click(relButton);
    expect(mockSelectElement).toHaveBeenCalledWith({
      kind: "all_relationships",
      projectId: 1,
    });
  });

  it("closed_category_calls_hook_with_null_projectId", () => {
    render(<CategoryTree projectId={1} />);
    // 初期状態ではすべてのカテゴリが閉じている → useTableList に null が渡る
    expect(vi.mocked(useTableList).mock.calls[0]?.[0]).toBeNull();
  });

  it("opening_category_calls_hook_with_real_projectId", () => {
    render(<CategoryTree projectId={1} />);
    vi.mocked(useTableList).mockClear();
    fireEvent.click(screen.getByText(/^テーブル \(/));
    // 展開後は projectId=1 でフックが呼ばれる
    expect(vi.mocked(useTableList)).toHaveBeenCalledWith(1);
  });

  it("element_click_calls_selectElement", () => {
    render(<CategoryTree projectId={1} />);

    // テーブルカテゴリを展開
    const tableHeader = screen.getByText(/^テーブル \(/);
    fireEvent.click(tableHeader);

    // Contact をクリック
    fireEvent.click(screen.getByText("Contact"));
    expect(mockSelectElement).toHaveBeenCalledWith({
      kind: "table",
      projectId: 1,
      id: 1,
      name: "Contact",
    });
  });
});
