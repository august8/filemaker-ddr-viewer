import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { AllFieldsPanel } from "../../components/detail/AllFieldsPanel";
import { makeAllFieldRow } from "../testFixtures";

vi.mock("../../hooks/useTauriCommand", () => ({
  useAllFields: vi.fn(),
}));

vi.mock("../../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

import { useAllFields } from "../../hooks/useTauriCommand";
import { useAppStore } from "../../stores/appStore";

const mockSetRightPanel = vi.fn();

const mockFields = [
  makeAllFieldRow({ id: 1, name: "FirstName", table_name: "Contact", data_type: "Text", field_type: "normal", comment: "名前", is_global: false, table_id: 10 }),
  makeAllFieldRow({ id: 2, name: "TotalAmount", table_name: "Invoice", data_type: "Number", field_type: "summary", comment: "", is_global: false, table_id: 20 }),
  makeAllFieldRow({ id: 3, name: "FullName", table_name: "Contact", data_type: "Text", field_type: "calculated", comment: "", is_global: true, table_id: 10 }),
];

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useAppStore).mockReturnValue({
    setRightPanel: mockSetRightPanel,
  } as unknown as ReturnType<typeof useAppStore>);
});

describe("AllFieldsPanel", () => {
  it("shows_loading_state", () => {
    vi.mocked(useAllFields).mockReturnValue(
      { data: undefined, isLoading: true } as unknown as ReturnType<typeof useAllFields>
    );
    render(<AllFieldsPanel projectId={1} />);
    expect(screen.getByText("読み込み中...")).toBeInTheDocument();
  });

  it("shows_all_fields", () => {
    vi.mocked(useAllFields).mockReturnValue(
      { data: mockFields, isLoading: false } as unknown as ReturnType<typeof useAllFields>
    );
    render(<AllFieldsPanel projectId={1} />);
    expect(screen.getByText("FirstName")).toBeInTheDocument();
    expect(screen.getByText("TotalAmount")).toBeInTheDocument();
    expect(screen.getByText("FullName")).toBeInTheDocument();
  });

  it("shows_empty_state_when_filter_matches_nothing", () => {
    vi.mocked(useAllFields).mockReturnValue(
      { data: mockFields, isLoading: false } as unknown as ReturnType<typeof useAllFields>
    );
    render(<AllFieldsPanel projectId={1} />);
    const input = screen.getByPlaceholderText(/絞り込み/);
    fireEvent.change(input, { target: { value: "ZZZNOMATCH" } });
    expect(screen.getByText("該当するフィールドなし")).toBeInTheDocument();
  });

  it("filters_by_field_name", () => {
    vi.mocked(useAllFields).mockReturnValue(
      { data: mockFields, isLoading: false } as unknown as ReturnType<typeof useAllFields>
    );
    render(<AllFieldsPanel projectId={1} />);
    const input = screen.getByPlaceholderText(/絞り込み/);
    fireEvent.change(input, { target: { value: "total" } });
    expect(screen.queryByText("FirstName")).not.toBeInTheDocument();
    expect(screen.getByText("TotalAmount")).toBeInTheDocument();
  });

  it("filters_by_table_name", () => {
    vi.mocked(useAllFields).mockReturnValue(
      { data: mockFields, isLoading: false } as unknown as ReturnType<typeof useAllFields>
    );
    render(<AllFieldsPanel projectId={1} />);
    const input = screen.getByPlaceholderText(/絞り込み/);
    fireEvent.change(input, { target: { value: "invoice" } });
    expect(screen.getByText("TotalAmount")).toBeInTheDocument();
    expect(screen.queryByText("FirstName")).not.toBeInTheDocument();
  });

  it("filters_by_comment", () => {
    vi.mocked(useAllFields).mockReturnValue(
      { data: mockFields, isLoading: false } as unknown as ReturnType<typeof useAllFields>
    );
    render(<AllFieldsPanel projectId={1} />);
    const input = screen.getByPlaceholderText(/絞り込み/);
    fireEvent.change(input, { target: { value: "名前" } });
    expect(screen.getByText("FirstName")).toBeInTheDocument();
    expect(screen.queryByText("TotalAmount")).not.toBeInTheDocument();
  });

  it("maps_normal_field_type_to_empty_string", () => {
    vi.mocked(useAllFields).mockReturnValue(
      { data: [makeAllFieldRow({ id: 1, name: "F1", field_type: "normal" })], isLoading: false } as unknown as ReturnType<typeof useAllFields>
    );
    render(<AllFieldsPanel projectId={1} />);
    // normal → 空文字なので「計算」「集計」が表示されないことを確認
    expect(screen.queryByText("計算")).not.toBeInTheDocument();
    expect(screen.queryByText("集計")).not.toBeInTheDocument();
  });

  it("maps_calculated_field_type_to_keisan", () => {
    vi.mocked(useAllFields).mockReturnValue(
      { data: [makeAllFieldRow({ id: 1, name: "CalcField", field_type: "calculated" })], isLoading: false } as unknown as ReturnType<typeof useAllFields>
    );
    render(<AllFieldsPanel projectId={1} />);
    expect(screen.getByText("計算")).toBeInTheDocument();
  });

  it("maps_summary_field_type_to_shukei", () => {
    vi.mocked(useAllFields).mockReturnValue(
      { data: [makeAllFieldRow({ id: 1, name: "SumField", field_type: "summary" })], isLoading: false } as unknown as ReturnType<typeof useAllFields>
    );
    render(<AllFieldsPanel projectId={1} />);
    expect(screen.getByText("集計")).toBeInTheDocument();
  });

  it("shows_global_badge_for_global_field", () => {
    vi.mocked(useAllFields).mockReturnValue(
      { data: [makeAllFieldRow({ id: 1, name: "GlobalField", is_global: true })], isLoading: false } as unknown as ReturnType<typeof useAllFields>
    );
    render(<AllFieldsPanel projectId={1} />);
    // "G" はヘッダー（th）とデータ行（td）の両方に現れる
    const gCells = screen.getAllByText("G");
    expect(gCells.length).toBeGreaterThanOrEqual(2);
  });

  it("clicking_row_calls_setRightPanel_with_field_info", () => {
    vi.mocked(useAllFields).mockReturnValue(
      { data: [makeAllFieldRow({ id: 5, name: "Email", table_id: 10, table_name: "Contact" })], isLoading: false } as unknown as ReturnType<typeof useAllFields>
    );
    render(<AllFieldsPanel projectId={2} />);
    const row = screen.getByText("Email").closest("tr")!;
    fireEvent.click(row);
    expect(mockSetRightPanel).toHaveBeenCalledWith({
      kind: "field",
      projectId: 2,
      tableId: 10,
      fieldId: 5,
      tableName: "Contact",
    });
  });
});
