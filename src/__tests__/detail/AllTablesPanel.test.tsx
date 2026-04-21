import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { AllTablesPanel } from "../../components/detail/AllTablesPanel";
import { makeTableRow } from "../testFixtures";

vi.mock("../../hooks/table", () => ({
  useTableList: vi.fn(),
}));

vi.mock("../../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

import { useTableList } from "../../hooks/table";
import { useAppStore } from "../../stores/appStore";

const mockTables = [
  makeTableRow({ id: 1, fm_id: 1, name: "Customer", field_count: 10 }),
  makeTableRow({ id: 2, fm_id: 2, name: "Order", field_count: 5 }),
];

const mockSelectElement = vi.fn();

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useAppStore).mockReturnValue({
    selectElement: mockSelectElement,
  } as unknown as ReturnType<typeof useAppStore>);
});

describe("AllTablesPanel", () => {
  it("row_click_navigates_to_table", () => {
    vi.mocked(useTableList).mockReturnValue(
      { data: mockTables, isLoading: false } as unknown as ReturnType<typeof useTableList>
    );
    render(<AllTablesPanel projectId={1} />);
    fireEvent.click(screen.getByText("Customer"));
    expect(mockSelectElement).toHaveBeenCalledWith({
      kind: "table",
      projectId: 1,
      id: 1,
      name: "Customer",
    });
  });

});
