import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { AllRelationshipsPanel } from "../../components/detail/AllRelationshipsPanel";
import { makeRelationshipRow, makePredicateRow } from "../testFixtures";
import { PAGE_SIZE } from "../../constants";

vi.mock("../../hooks/table", () => ({
  useRelationshipList: vi.fn(),
}));

import { useRelationshipList } from "../../hooks/table";

const fullPageRels = Array.from({ length: PAGE_SIZE }, (_, i) =>
  makeRelationshipRow({
    id: i + 1, fm_id: i + 1, name: `Rel${i}`,
    left_table: "A", right_table: "B", predicates: [],
  })
);

const mockRelationships = [
  makeRelationshipRow({
    id: 1, fm_id: 10, name: "Customers::Orders",
    left_table: "Customers", right_table: "Orders",
    predicates: [makePredicateRow({ id: 1, left_field: "id", right_field: "customer_id" })],
  }),
  makeRelationshipRow({
    id: 2, fm_id: 11, name: "Orders::LineItems",
    left_table: "Orders", right_table: "LineItems",
    predicates: [
      makePredicateRow({ id: 2, left_field: "id", right_field: "order_id" }),
      makePredicateRow({ id: 3, left_field: "status", right_field: "status", position: 1 }),
    ],
  }),
];

beforeEach(() => {
  vi.clearAllMocks();
});

describe("AllRelationshipsPanel", () => {
  it("shows_predicate_in_full_format", () => {
    vi.mocked(useRelationshipList).mockReturnValue(
      { data: mockRelationships, isLoading: false } as unknown as ReturnType<typeof useRelationshipList>
    );
    render(<AllRelationshipsPanel projectId={1} />);
    // 左テーブル::左フィールド operator 右テーブル::右フィールド 形式
    expect(screen.getByText("Customers::id = Orders::customer_id")).toBeInTheDocument();
  });

  it("shows_all_predicates_for_multi_predicate_relationship", () => {
    vi.mocked(useRelationshipList).mockReturnValue(
      { data: mockRelationships, isLoading: false } as unknown as ReturnType<typeof useRelationshipList>
    );
    render(<AllRelationshipsPanel projectId={1} />);
    // 2件目のリレーション: 述語が2件とも表示される
    expect(screen.getByText("Orders::id = LineItems::order_id")).toBeInTheDocument();
    expect(screen.getByText("Orders::status = LineItems::status")).toBeInTheDocument();
  });

  it("prev_button_disabled_on_first_page", () => {
    vi.mocked(useRelationshipList).mockReturnValue(
      { data: mockRelationships, isLoading: false } as unknown as ReturnType<typeof useRelationshipList>
    );
    render(<AllRelationshipsPanel projectId={1} />);
    expect(screen.getByRole("button", { name: /前/ })).toBeDisabled();
  });

  it("next_button_disabled_when_last_page", () => {
    vi.mocked(useRelationshipList).mockReturnValue(
      { data: mockRelationships, isLoading: false } as unknown as ReturnType<typeof useRelationshipList>
    );
    render(<AllRelationshipsPanel projectId={1} />);
    expect(screen.getByRole("button", { name: /次/ })).toBeDisabled();
  });

  it("next_click_increments_offset", () => {
    vi.mocked(useRelationshipList)
      .mockReturnValueOnce({ data: fullPageRels, isLoading: false } as unknown as ReturnType<typeof useRelationshipList>)
      .mockReturnValue({ data: mockRelationships, isLoading: false } as unknown as ReturnType<typeof useRelationshipList>);
    render(<AllRelationshipsPanel projectId={1} />);
    fireEvent.click(screen.getByRole("button", { name: /次/ }));
    expect(vi.mocked(useRelationshipList).mock.lastCall?.[2]).toBe(PAGE_SIZE);
  });

  it("filter_resets_page_to_zero", () => {
    vi.mocked(useRelationshipList).mockReturnValue(
      { data: fullPageRels, isLoading: false } as unknown as ReturnType<typeof useRelationshipList>
    );
    render(<AllRelationshipsPanel projectId={1} />);
    fireEvent.click(screen.getByRole("button", { name: /次/ }));
    fireEvent.change(screen.getByPlaceholderText(/絞り込み/), { target: { value: "x" } });
    const calls = vi.mocked(useRelationshipList).mock.calls;
    expect(calls[calls.length - 1][2]).toBe(0);
  });

});
