import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { FieldCalcReferences } from "../../../components/detail/field/FieldCalcReferences";

vi.mock("../../../hooks/fieldRefs", () => ({
  useFieldCalcRefs: vi.fn(),
}));
vi.mock("../../../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

import { useFieldCalcRefs } from "../../../hooks/fieldRefs";
import { useAppStore } from "../../../stores/appStore";

const mockSetRightPanel2 = vi.fn();

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useAppStore).mockReturnValue({
    setRightPanel2: mockSetRightPanel2,
  } as unknown as ReturnType<typeof useAppStore>);
});

describe("FieldCalcReferences", () => {
  it("shows_empty_message_when_no_refs", () => {
    vi.mocked(useFieldCalcRefs).mockReturnValue({
      data: [],
      isLoading: false,
    } as unknown as ReturnType<typeof useFieldCalcRefs>);
    render(<FieldCalcReferences projectId={1} tableName="T" fieldName="F" />);
    expect(screen.getByText(/他のフィールドの計算式で参照されていません/)).toBeInTheDocument();
  });

  it("shows_loading_state", () => {
    vi.mocked(useFieldCalcRefs).mockReturnValue({
      data: [],
      isLoading: true,
    } as unknown as ReturnType<typeof useFieldCalcRefs>);
    render(<FieldCalcReferences projectId={1} tableName="T" fieldName="F" />);
    expect(screen.getByText(/読み込み中/)).toBeInTheDocument();
  });

  it("click_calls_setRightPanel2_with_correct_args", () => {
    vi.mocked(useFieldCalcRefs).mockReturnValue({
      data: [{ field_id: 42, field_name: "CalcField", table_name: "Customer", table_id: 5, project_id: 99 }],
      isLoading: false,
    } as unknown as ReturnType<typeof useFieldCalcRefs>);
    render(<FieldCalcReferences projectId={1} tableName="T" fieldName="F" />);
    fireEvent.click(screen.getByRole("button", { name: "Customer::CalcField" }));
    expect(mockSetRightPanel2).toHaveBeenCalledWith({
      kind: "field",
      projectId: 99,
      fieldProjectId: 99,
      tableId: 5,
      fieldId: 42,
      tableName: "Customer",
    });
  });
});
