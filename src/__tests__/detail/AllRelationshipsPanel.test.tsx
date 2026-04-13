import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { AllRelationshipsPanel } from "../../components/detail/AllRelationshipsPanel";
import { makeRelationshipRow, makePredicateRow } from "../testFixtures";

vi.mock("../../hooks/useTauriCommand", () => ({
  useRelationshipList: vi.fn(),
}));

import { useRelationshipList } from "../../hooks/useTauriCommand";

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
  it("shows_loading_state", () => {
    vi.mocked(useRelationshipList).mockReturnValue(
      { data: undefined, isLoading: true } as unknown as ReturnType<typeof useRelationshipList>
    );
    render(<AllRelationshipsPanel projectId={1} />);
    expect(screen.getByText("読み込み中...")).toBeInTheDocument();
  });

  it("shows_empty_state", () => {
    vi.mocked(useRelationshipList).mockReturnValue(
      { data: [], isLoading: false } as unknown as ReturnType<typeof useRelationshipList>
    );
    render(<AllRelationshipsPanel projectId={1} />);
    expect(screen.getByText(/該当するリレーションなし/)).toBeInTheDocument();
  });

  it("shows_left_and_right_tables", () => {
    vi.mocked(useRelationshipList).mockReturnValue(
      { data: mockRelationships, isLoading: false } as unknown as ReturnType<typeof useRelationshipList>
    );
    render(<AllRelationshipsPanel projectId={1} />);
    expect(screen.getByText("Customers")).toBeInTheDocument();
    expect(screen.getAllByText("Orders").length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText("LineItems")).toBeInTheDocument();
  });

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

  it("does_not_show_relationship_name_column", () => {
    vi.mocked(useRelationshipList).mockReturnValue(
      { data: mockRelationships, isLoading: false } as unknown as ReturnType<typeof useRelationshipList>
    );
    render(<AllRelationshipsPanel projectId={1} />);
    // リレーション名("Customers::Orders")はヘッダー列として表示されない
    expect(screen.queryByRole("columnheader", { name: /リレーション名/ })).not.toBeInTheDocument();
  });

  it("shows_header_with_count", () => {
    vi.mocked(useRelationshipList).mockReturnValue(
      { data: mockRelationships, isLoading: false } as unknown as ReturnType<typeof useRelationshipList>
    );
    render(<AllRelationshipsPanel projectId={1} />);
    expect(screen.getByText(/リレーション一覧/)).toBeInTheDocument();
    expect(screen.getByText(/2/)).toBeInTheDocument();
  });
});
