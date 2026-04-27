import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { AllTablesPanel } from "../../components/detail/AllTablesPanel";
import { makeTableRow } from "../testFixtures";
import { PAGE_SIZE } from "../../constants";

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

const fullPage = Array.from({ length: PAGE_SIZE }, (_, i) =>
  makeTableRow({ id: i + 1, fm_id: i + 1, name: `Table${i}`, field_count: 0 })
);

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

  it("prev_button_disabled_on_first_page", () => {
    vi.mocked(useTableList).mockReturnValue(
      { data: mockTables, isLoading: false } as unknown as ReturnType<typeof useTableList>
    );
    render(<AllTablesPanel projectId={1} />);
    expect(screen.getByRole("button", { name: /前/ })).toBeDisabled();
  });

  it("next_button_disabled_when_last_page", () => {
    vi.mocked(useTableList).mockReturnValue(
      { data: mockTables, isLoading: false } as unknown as ReturnType<typeof useTableList>
    );
    render(<AllTablesPanel projectId={1} />);
    expect(screen.getByRole("button", { name: /次/ })).toBeDisabled();
  });

  it("next_click_increments_offset", () => {
    vi.mocked(useTableList)
      .mockReturnValueOnce({ data: fullPage, isLoading: false } as unknown as ReturnType<typeof useTableList>)
      .mockReturnValue({ data: mockTables, isLoading: false } as unknown as ReturnType<typeof useTableList>);
    render(<AllTablesPanel projectId={1} />);
    fireEvent.click(screen.getByRole("button", { name: /次/ }));
    expect(vi.mocked(useTableList).mock.lastCall?.[2]).toBe(PAGE_SIZE);
  });

  it("filter_resets_page_to_zero", () => {
    vi.mocked(useTableList).mockReturnValue(
      { data: fullPage, isLoading: false } as unknown as ReturnType<typeof useTableList>
    );
    render(<AllTablesPanel projectId={1} />);
    fireEvent.click(screen.getByRole("button", { name: /次/ }));
    fireEvent.change(screen.getByPlaceholderText(/絞り込み/), { target: { value: "x" } });
    const calls = vi.mocked(useTableList).mock.calls;
    expect(calls[calls.length - 1][2]).toBe(0);
  });
});
