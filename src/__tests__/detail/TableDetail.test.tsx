import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { TableDetail } from "../../components/detail/TableDetail";
import { makeFieldRow } from "../testFixtures";

vi.mock("../../hooks/table", () => ({
  useTableFields: vi.fn(),
  useTableList: vi.fn(() => ({ data: [], isLoading: false })),
}));

vi.mock("../../stores/appStore", () => ({
  useAppStore: vi.fn(() => ({
    setRightPanel: vi.fn(),
    rightPanel: null,
    diffContext: null,
  })),
}));

import { useTableFields, useTableList } from "../../hooks/table";
import { useAppStore } from "../../stores/appStore";

const mockFields = [
  makeFieldRow({ id: 1, fm_id: 1, name: "FirstName", data_type: "Text", comment: "First name" }),
  makeFieldRow({ id: 2, fm_id: 2, name: "GlobalFlag", data_type: "Number", is_global: true }),
];

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useAppStore).mockReturnValue({
    setRightPanel: vi.fn(),
    rightPanel: null,
    diffContext: null,
  } as unknown as ReturnType<typeof useAppStore>);
  vi.mocked(useTableList).mockReturnValue({
    data: [],
    isLoading: false,
  } as unknown as ReturnType<typeof useTableList>);
});

describe("TableDetail", () => {
  it("renders_field_list", () => {
    vi.mocked(useTableFields).mockReturnValue({
      data: mockFields,
      isLoading: false,
    } as unknown as ReturnType<typeof useTableFields>);
    render(<TableDetail projectId={1} tableId={1} name="Contact" />);
    expect(screen.getByText("FirstName")).toBeInTheDocument();
    expect(screen.getByText("Text")).toBeInTheDocument();
    expect(screen.getByText("GlobalFlag")).toBeInTheDocument();
  });

  it("renders_global_badge", () => {
    vi.mocked(useTableFields).mockReturnValue({
      data: mockFields,
      isLoading: false,
    } as unknown as ReturnType<typeof useTableFields>);
    render(<TableDetail projectId={1} tableId={1} name="Contact" />);
    const badges = screen.getAllByText("G");
    expect(badges.length).toBeGreaterThan(0);
  });

  it("renders_empty_when_no_fields", () => {
    vi.mocked(useTableFields).mockReturnValue({
      data: [],
      isLoading: false,
    } as unknown as ReturnType<typeof useTableFields>);
    render(<TableDetail projectId={1} tableId={1} name="Contact" />);
    expect(screen.getByText("フィールドなし")).toBeInTheDocument();
  });

  it("renders_spinner_when_loading", () => {
    vi.mocked(useTableFields).mockReturnValue({
      data: undefined,
      isLoading: true,
    } as unknown as ReturnType<typeof useTableFields>);
    render(<TableDetail projectId={1} tableId={1} name="Contact" />);
    expect(screen.getByText("読み込み中...")).toBeInTheDocument();
  });

  it("clicking_field_row_calls_setRightPanel", () => {
    const mockSetRightPanel = vi.fn();
    vi.mocked(useAppStore).mockReturnValue({
      setRightPanel: mockSetRightPanel,
      rightPanel: null,
      diffContext: null,
    } as unknown as ReturnType<typeof useAppStore>);
    vi.mocked(useTableFields).mockReturnValue({
      data: [makeFieldRow({ id: 10, name: "Email" })],
      isLoading: false,
    } as unknown as ReturnType<typeof useTableFields>);
    render(<TableDetail projectId={1} tableId={5} name="Contact" />);
    const row = screen.getByText("Email").closest("tr")!;
    fireEvent.click(row);
    expect(mockSetRightPanel).toHaveBeenCalledWith({
      kind: "field",
      fieldId: 10,
      tableId: 5,
      projectId: 1,
      tableName: "Contact",
    });
  });

  it("diff_context_shows_added_and_removed_badges", () => {
    // current has "FirstName" (new, not in compare) → Added
    // compare has "OldField" (not in current) → Removed
    const currentFields = [makeFieldRow({ id: 1, name: "FirstName", data_type: "Text" })];
    const compareFields = [makeFieldRow({ id: 99, name: "OldField", data_type: "Text" })];
    vi.mocked(useAppStore).mockReturnValue({
      setRightPanel: vi.fn(),
      rightPanel: null,
      diffContext: { compareProjectId: 2 },
    } as unknown as ReturnType<typeof useAppStore>);
    vi.mocked(useTableList).mockReturnValue({
      data: [{ id: 10, fm_id: 10, name: "Contact", field_count: 1 }],
      isLoading: false,
    } as unknown as ReturnType<typeof useTableList>);
    // first call: current project fields; second call: compare project fields
    vi.mocked(useTableFields)
      .mockReturnValueOnce({ data: currentFields, isLoading: false } as unknown as ReturnType<typeof useTableFields>)
      .mockReturnValueOnce({ data: compareFields, isLoading: false } as unknown as ReturnType<typeof useTableFields>);
    render(<TableDetail projectId={1} tableId={1} name="Contact" />);
    expect(screen.getByText("追加")).toBeInTheDocument();
    expect(screen.getByText("削除")).toBeInTheDocument();
  });

  it("diff_context_shows_modified_badge_when_data_type_changes", () => {
    const currentFields = [makeFieldRow({ id: 1, name: "Amount", data_type: "Number" })];
    const compareFields = [makeFieldRow({ id: 1, name: "Amount", data_type: "Text" })];
    vi.mocked(useAppStore).mockReturnValue({
      setRightPanel: vi.fn(),
      rightPanel: null,
      diffContext: { compareProjectId: 2 },
    } as unknown as ReturnType<typeof useAppStore>);
    vi.mocked(useTableList).mockReturnValue({
      data: [{ id: 10, fm_id: 10, name: "Contact", field_count: 1 }],
      isLoading: false,
    } as unknown as ReturnType<typeof useTableList>);
    vi.mocked(useTableFields)
      .mockReturnValueOnce({ data: currentFields, isLoading: false } as unknown as ReturnType<typeof useTableFields>)
      .mockReturnValueOnce({ data: compareFields, isLoading: false } as unknown as ReturnType<typeof useTableFields>);
    render(<TableDetail projectId={1} tableId={1} name="Contact" />);
    expect(screen.getByText("変更")).toBeInTheDocument();
  });
});
