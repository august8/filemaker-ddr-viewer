import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { RelationshipGraphPanel } from "../../components/detail/RelationshipGraphPanel";
import { makeTableOccurrenceRow, makeRelationshipRow, makePredicateRow } from "../testFixtures";

// dagre は jsdom 上でレイアウト計算できないためモック
// new Graph() ごとに独立した nodes/edges コレクションを返すファクトリ方式
vi.mock("dagre", () => {
  return {
    default: {
      graphlib: {
        // アロー関数は new できないため通常の function を使用
        Graph: vi.fn().mockImplementation(function(this: unknown) {
          const nodeMap: Record<string, { x: number; y: number; width: number; height: number; label: string }> = {};
          const edgeList: Array<{ v: string; w: string; name?: string }> = [];
          return {
            setGraph: vi.fn(),
            setDefaultEdgeLabel: vi.fn(),
            setNode: vi.fn((id: string, attrs: { label: string; width: number; height: number }) => {
              nodeMap[id] = { x: 100, y: 100, width: attrs.width, height: attrs.height, label: attrs.label };
            }),
            setEdge: vi.fn((v: string, w: string, _label: object, name?: string) => {
              edgeList.push({ v, w, name });
            }),
            node: vi.fn((id: string) => nodeMap[id] ?? { x: 0, y: 0, width: 160, height: 48, label: id }),
            nodes: vi.fn(() => Object.keys(nodeMap)),
            edges: vi.fn(() => edgeList.map((e) => ({ v: e.v, w: e.w, name: e.name }))),
            edge: vi.fn(() => ({ points: [{ x: 0, y: 0 }, { x: 200, y: 0 }] })),
            graph: vi.fn(() => ({ width: 400, height: 300 })),
          };
        }),
      },
      layout: vi.fn(),
    },
  };
});

vi.mock("../../hooks/useTauriCommand", () => ({
  useTableOccurrenceList: vi.fn(),
  useRelationshipList: vi.fn(),
}));

import { useTableOccurrenceList, useRelationshipList } from "../../hooks/useTauriCommand";

const mockTOs = [
  makeTableOccurrenceRow({ id: 1, occurrence_name: "Customers", base_table_name: "Customer" }),
  makeTableOccurrenceRow({ id: 2, occurrence_name: "Orders", base_table_name: "Order" }),
];

const mockRelationships = [
  makeRelationshipRow({
    id: 1, fm_id: 10, name: "Customers::Orders",
    left_table: "Customers", right_table: "Orders",
    predicates: [makePredicateRow({ id: 1, left_field: "id", right_field: "customer_id" })],
  }),
];

beforeEach(() => {
  vi.clearAllMocks();
});

describe("RelationshipGraphPanel", () => {
  it("shows_loading_state_when_data_loading", () => {
    vi.mocked(useTableOccurrenceList).mockReturnValue(
      { data: undefined, isLoading: true } as unknown as ReturnType<typeof useTableOccurrenceList>
    );
    vi.mocked(useRelationshipList).mockReturnValue(
      { data: undefined, isLoading: true } as unknown as ReturnType<typeof useRelationshipList>
    );
    render(<RelationshipGraphPanel projectId={1} />);
    expect(screen.getByText(/読み込み中/)).toBeInTheDocument();
  });

  it("shows_empty_state_when_no_table_occurrences", () => {
    vi.mocked(useTableOccurrenceList).mockReturnValue(
      { data: [], isLoading: false } as unknown as ReturnType<typeof useTableOccurrenceList>
    );
    vi.mocked(useRelationshipList).mockReturnValue(
      { data: [], isLoading: false } as unknown as ReturnType<typeof useRelationshipList>
    );
    render(<RelationshipGraphPanel projectId={1} />);
    expect(screen.getByText(/テーブルオカレンスなし/)).toBeInTheDocument();
  });

  it("renders_one_node_per_table_occurrence", () => {
    vi.mocked(useTableOccurrenceList).mockReturnValue(
      { data: mockTOs, isLoading: false } as unknown as ReturnType<typeof useTableOccurrenceList>
    );
    vi.mocked(useRelationshipList).mockReturnValue(
      { data: mockRelationships, isLoading: false } as unknown as ReturnType<typeof useRelationshipList>
    );
    render(<RelationshipGraphPanel projectId={1} />);
    const nodes = screen.getAllByTestId("graph-node");
    expect(nodes).toHaveLength(mockTOs.length);
  });

  it("renders_occurrence_name_and_base_table_in_nodes", () => {
    vi.mocked(useTableOccurrenceList).mockReturnValue(
      { data: mockTOs, isLoading: false } as unknown as ReturnType<typeof useTableOccurrenceList>
    );
    vi.mocked(useRelationshipList).mockReturnValue(
      { data: mockRelationships, isLoading: false } as unknown as ReturnType<typeof useRelationshipList>
    );
    render(<RelationshipGraphPanel projectId={1} />);
    // SVG <text> と <title> の両方にテキストが現れるため getAllByText を使用
    expect(screen.getAllByText("Customers").length).toBeGreaterThan(0);
    expect(screen.getByText("Customer")).toBeInTheDocument();
    expect(screen.getAllByText("Orders").length).toBeGreaterThan(0);
    expect(screen.getByText("Order")).toBeInTheDocument();
  });

  it("renders_one_edge_per_relationship", () => {
    vi.mocked(useTableOccurrenceList).mockReturnValue(
      { data: mockTOs, isLoading: false } as unknown as ReturnType<typeof useTableOccurrenceList>
    );
    vi.mocked(useRelationshipList).mockReturnValue(
      { data: mockRelationships, isLoading: false } as unknown as ReturnType<typeof useRelationshipList>
    );
    render(<RelationshipGraphPanel projectId={1} />);
    const edges = screen.getAllByTestId("graph-edge");
    expect(edges).toHaveLength(mockRelationships.length);
  });

  it("renders_header_with_counts", () => {
    vi.mocked(useTableOccurrenceList).mockReturnValue(
      { data: mockTOs, isLoading: false } as unknown as ReturnType<typeof useTableOccurrenceList>
    );
    vi.mocked(useRelationshipList).mockReturnValue(
      { data: mockRelationships, isLoading: false } as unknown as ReturnType<typeof useRelationshipList>
    );
    render(<RelationshipGraphPanel projectId={1} />);
    expect(screen.getByText(/リレーショングラフ/)).toBeInTheDocument();
    expect(screen.getByText(/2 TO/)).toBeInTheDocument();
  });
});
