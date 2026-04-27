import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { AllValueListsPanel } from "../../components/detail/AllValueListsPanel";
import { makeValueListRow } from "../testFixtures";

vi.mock("../../hooks/catalog", () => ({
  useValueListList: vi.fn(),
}));

vi.mock("../../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

import { useValueListList } from "../../hooks/catalog";
import { useAppStore } from "../../stores/appStore";

const PAGE_SIZE = 500;

const mockValueLists = [
  makeValueListRow({ id: 1, fm_id: 1, name: "Status", item_count: 3 }),
  makeValueListRow({ id: 2, fm_id: 2, name: "Category", item_count: 5 }),
];

const fullPage = Array.from({ length: PAGE_SIZE }, (_, i) =>
  makeValueListRow({ id: i + 1, fm_id: i + 1, name: `VL${i}`, item_count: 0 })
);

const mockSelectElement = vi.fn();

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useAppStore).mockReturnValue({
    selectElement: mockSelectElement,
  } as unknown as ReturnType<typeof useAppStore>);
});

describe("AllValueListsPanel", () => {
  it("row_click_navigates_to_value_list", () => {
    vi.mocked(useValueListList).mockReturnValue(
      { data: mockValueLists, isLoading: false } as unknown as ReturnType<typeof useValueListList>
    );
    render(<AllValueListsPanel projectId={1} />);
    fireEvent.click(screen.getByText("Status"));
    expect(mockSelectElement).toHaveBeenCalledWith({
      kind: "value_list",
      projectId: 1,
      id: 1,
      name: "Status",
    });
  });

  it("prev_button_disabled_on_first_page", () => {
    vi.mocked(useValueListList).mockReturnValue(
      { data: mockValueLists, isLoading: false } as unknown as ReturnType<typeof useValueListList>
    );
    render(<AllValueListsPanel projectId={1} />);
    expect(screen.getByRole("button", { name: /前/ })).toBeDisabled();
  });

  it("next_button_disabled_when_last_page", () => {
    vi.mocked(useValueListList).mockReturnValue(
      { data: mockValueLists, isLoading: false } as unknown as ReturnType<typeof useValueListList>
    );
    render(<AllValueListsPanel projectId={1} />);
    expect(screen.getByRole("button", { name: /次/ })).toBeDisabled();
  });

  it("next_button_enabled_when_full_page", () => {
    vi.mocked(useValueListList).mockReturnValue(
      { data: fullPage, isLoading: false } as unknown as ReturnType<typeof useValueListList>
    );
    render(<AllValueListsPanel projectId={1} />);
    expect(screen.getByRole("button", { name: /次/ })).not.toBeDisabled();
  });

  it("filter_resets_page_to_zero", () => {
    vi.mocked(useValueListList).mockReturnValue(
      { data: fullPage, isLoading: false } as unknown as ReturnType<typeof useValueListList>
    );
    render(<AllValueListsPanel projectId={1} />);
    fireEvent.click(screen.getByRole("button", { name: /次/ }));
    fireEvent.change(screen.getByPlaceholderText(/絞り込み/), { target: { value: "x" } });
    const calls = vi.mocked(useValueListList).mock.calls;
    expect(calls[calls.length - 1][2]).toBe(0);
  });
});
