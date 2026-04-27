import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { AllLayoutsPanel } from "../../components/detail/AllLayoutsPanel";
import { makeLayoutRow } from "../testFixtures";
import { PAGE_SIZE } from "../../constants";

vi.mock("../../hooks/layout", () => ({
  useLayoutList: vi.fn(),
}));

vi.mock("../../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

import { useLayoutList } from "../../hooks/layout";
import { useAppStore } from "../../stores/appStore";

const mockLayouts = [
  makeLayoutRow({ id: 1, fm_id: 1, name: "Customer List", table_occurrence_name: "Customers", trigger_count: 2 }),
  makeLayoutRow({ id: 2, fm_id: 2, name: "Report" }),
];

const fullPage = Array.from({ length: PAGE_SIZE }, (_, i) =>
  makeLayoutRow({ id: i + 1, fm_id: i + 1, name: `Layout${i}` })
);

const mockSelectElement = vi.fn();

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useAppStore).mockReturnValue({
    selectElement: mockSelectElement,
  } as unknown as ReturnType<typeof useAppStore>);
});

describe("AllLayoutsPanel", () => {
  it("row_click_navigates_to_layout", () => {
    vi.mocked(useLayoutList).mockReturnValue(
      { data: mockLayouts, isLoading: false } as unknown as ReturnType<typeof useLayoutList>
    );
    render(<AllLayoutsPanel projectId={1} />);
    fireEvent.click(screen.getByText("Customer List"));
    expect(mockSelectElement).toHaveBeenCalledWith({
      kind: "layout",
      projectId: 1,
      id: 1,
      name: "Customer List",
    });
  });

  it("prev_button_disabled_on_first_page", () => {
    vi.mocked(useLayoutList).mockReturnValue(
      { data: mockLayouts, isLoading: false } as unknown as ReturnType<typeof useLayoutList>
    );
    render(<AllLayoutsPanel projectId={1} />);
    expect(screen.getByRole("button", { name: /前/ })).toBeDisabled();
  });

  it("next_button_disabled_when_last_page", () => {
    vi.mocked(useLayoutList).mockReturnValue(
      { data: mockLayouts, isLoading: false } as unknown as ReturnType<typeof useLayoutList>
    );
    render(<AllLayoutsPanel projectId={1} />);
    expect(screen.getByRole("button", { name: /次/ })).toBeDisabled();
  });

  it("next_click_increments_offset", () => {
    vi.mocked(useLayoutList)
      .mockReturnValueOnce({ data: fullPage, isLoading: false } as unknown as ReturnType<typeof useLayoutList>)
      .mockReturnValue({ data: mockLayouts, isLoading: false } as unknown as ReturnType<typeof useLayoutList>);
    render(<AllLayoutsPanel projectId={1} />);
    fireEvent.click(screen.getByRole("button", { name: /次/ }));
    expect(vi.mocked(useLayoutList).mock.lastCall?.[2]).toBe(PAGE_SIZE);
  });

  it("filter_resets_page_to_zero", () => {
    vi.mocked(useLayoutList).mockReturnValue(
      { data: fullPage, isLoading: false } as unknown as ReturnType<typeof useLayoutList>
    );
    render(<AllLayoutsPanel projectId={1} />);
    fireEvent.click(screen.getByRole("button", { name: /次/ }));
    fireEvent.change(screen.getByPlaceholderText(/絞り込み/), { target: { value: "x" } });
    const calls = vi.mocked(useLayoutList).mock.calls;
    expect(calls[calls.length - 1][2]).toBe(0);
  });
});
