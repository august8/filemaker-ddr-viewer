import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { AllTableOccurrencesPanel } from "../../components/detail/AllTableOccurrencesPanel";
import { makeTableOccurrenceRow, makeTableRow, makeLayoutRow } from "../testFixtures";
import { PAGE_SIZE } from "../../constants";

vi.mock("../../hooks/table", () => ({
  useTableOccurrenceList: vi.fn(),
  useTableList: vi.fn(),
}));
vi.mock("../../hooks/layout", () => ({
  useLayoutList: vi.fn(),
}));

vi.mock("../../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

import { useTableOccurrenceList, useTableList } from "../../hooks/table";
import { useLayoutList } from "../../hooks/layout";
import { useAppStore } from "../../stores/appStore";

const mockTOs = [
  makeTableOccurrenceRow({ id: 1, occurrence_name: "Customers", base_table_name: "Customer" }),
  makeTableOccurrenceRow({ id: 2, occurrence_name: "Orders", base_table_name: "Order" }),
];

const fullPageTOs = Array.from({ length: PAGE_SIZE }, (_, i) =>
  makeTableOccurrenceRow({ id: i + 1, occurrence_name: `TO${i}`, base_table_name: "Customer" })
);

const mockTables = [
  makeTableRow({ id: 10, fm_id: 1, name: "Customer", field_count: 5 }),
  makeTableRow({ id: 11, fm_id: 2, name: "Order", field_count: 3 }),
];

const mockLayouts = [
  makeLayoutRow({ id: 100, fm_id: 1, name: "Customer List", table_occurrence_name: "Customers" }),
  makeLayoutRow({ id: 101, fm_id: 2, name: "Customer Detail", table_occurrence_name: "Customers" }),
  makeLayoutRow({ id: 102, fm_id: 3, name: "Order Entry", table_occurrence_name: "Orders" }),
];

const mockSelectElement = vi.fn();
const mockSetRightPanel = vi.fn();

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useAppStore).mockReturnValue({
    selectElement: mockSelectElement,
    setRightPanel: mockSetRightPanel,
  } as unknown as ReturnType<typeof useAppStore>);
  vi.mocked(useTableList).mockReturnValue(
    { data: mockTables, isLoading: false } as unknown as ReturnType<typeof useTableList>
  );
  vi.mocked(useLayoutList).mockReturnValue(
    { data: mockLayouts, isLoading: false } as unknown as ReturnType<typeof useLayoutList>
  );
});

describe("AllTableOccurrencesPanel", () => {
  it("prev_button_disabled_on_first_page", () => {
    vi.mocked(useTableOccurrenceList).mockReturnValue(
      { data: mockTOs, isLoading: false } as unknown as ReturnType<typeof useTableOccurrenceList>
    );
    render(<AllTableOccurrencesPanel projectId={1} />);
    expect(screen.getByRole("button", { name: /前/ })).toBeDisabled();
  });

  it("next_button_disabled_when_last_page", () => {
    vi.mocked(useTableOccurrenceList).mockReturnValue(
      { data: mockTOs, isLoading: false } as unknown as ReturnType<typeof useTableOccurrenceList>
    );
    render(<AllTableOccurrencesPanel projectId={1} />);
    expect(screen.getByRole("button", { name: /次/ })).toBeDisabled();
  });

  it("next_click_increments_offset", () => {
    vi.mocked(useTableOccurrenceList)
      .mockReturnValueOnce({ data: fullPageTOs, isLoading: false } as unknown as ReturnType<typeof useTableOccurrenceList>)
      .mockReturnValue({ data: mockTOs, isLoading: false } as unknown as ReturnType<typeof useTableOccurrenceList>);
    render(<AllTableOccurrencesPanel projectId={1} />);
    fireEvent.click(screen.getByRole("button", { name: /次/ }));
    expect(vi.mocked(useTableOccurrenceList).mock.lastCall?.[2]).toBe(PAGE_SIZE);
  });

  it("filter_resets_page_to_zero", () => {
    vi.mocked(useTableOccurrenceList).mockReturnValue(
      { data: fullPageTOs, isLoading: false } as unknown as ReturnType<typeof useTableOccurrenceList>
    );
    render(<AllTableOccurrencesPanel projectId={1} />);
    fireEvent.click(screen.getByRole("button", { name: /次/ }));
    fireEvent.change(screen.getByPlaceholderText(/絞り込み/), { target: { value: "x" } });
    const calls = vi.mocked(useTableOccurrenceList).mock.calls;
    expect(calls[calls.length - 1][2]).toBe(0);
  });

  it("base_table_click_navigates_to_table", () => {
    vi.mocked(useTableOccurrenceList).mockReturnValue(
      { data: mockTOs, isLoading: false } as unknown as ReturnType<typeof useTableOccurrenceList>
    );
    render(<AllTableOccurrencesPanel projectId={1} />);
    const customerBtn = screen.getByRole("button", { name: "Customer" });
    fireEvent.click(customerBtn);
    expect(mockSelectElement).toHaveBeenCalledWith({
      kind: "table",
      projectId: 1,
      id: 10,
      name: "Customer",
    });
  });

  it("shows_layout_badges_for_each_to", () => {
    vi.mocked(useTableOccurrenceList).mockReturnValue(
      { data: mockTOs, isLoading: false } as unknown as ReturnType<typeof useTableOccurrenceList>
    );
    render(<AllTableOccurrencesPanel projectId={1} />);
    // Customers に2レイアウト
    expect(screen.getByRole("button", { name: "Customer List" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Customer Detail" })).toBeInTheDocument();
    // Orders に1レイアウト
    expect(screen.getByRole("button", { name: "Order Entry" })).toBeInTheDocument();
  });

  it("layout_badge_click_navigates_to_layout", () => {
    vi.mocked(useTableOccurrenceList).mockReturnValue(
      { data: mockTOs, isLoading: false } as unknown as ReturnType<typeof useTableOccurrenceList>
    );
    render(<AllTableOccurrencesPanel projectId={1} />);
    fireEvent.click(screen.getByRole("button", { name: "Customer List" }));
    expect(mockSelectElement).toHaveBeenCalledWith({
      kind: "layout",
      projectId: 1,
      id: 100,
      name: "Customer List",
    });
  });

  it("shows_dash_when_no_layouts", () => {
    const tosWithNoLayout = [makeTableOccurrenceRow({ id: 3, occurrence_name: "Orphan", base_table_name: "Customer" })];
    vi.mocked(useTableOccurrenceList).mockReturnValue(
      { data: tosWithNoLayout, isLoading: false } as unknown as ReturnType<typeof useTableOccurrenceList>
    );
    render(<AllTableOccurrencesPanel projectId={1} />);
    expect(screen.getByText("—")).toBeInTheDocument();
  });
});
