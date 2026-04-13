import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { AllLayoutsPanel } from "../../components/detail/AllLayoutsPanel";
import { makeLayoutRow } from "../testFixtures";

vi.mock("../../hooks/useTauriCommand", () => ({
  useLayoutList: vi.fn(),
}));

vi.mock("../../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

import { useLayoutList } from "../../hooks/useTauriCommand";
import { useAppStore } from "../../stores/appStore";

const mockLayouts = [
  makeLayoutRow({ id: 1, fm_id: 1, name: "Customer List", table_occurrence_name: "Customers", trigger_count: 2 }),
  makeLayoutRow({ id: 2, fm_id: 2, name: "Report" }),
];

const mockSelectElement = vi.fn();

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useAppStore).mockReturnValue({
    selectElement: mockSelectElement,
  } as unknown as ReturnType<typeof useAppStore>);
});

describe("AllLayoutsPanel", () => {
  it("shows_loading_state", () => {
    vi.mocked(useLayoutList).mockReturnValue(
      { data: undefined, isLoading: true } as unknown as ReturnType<typeof useLayoutList>
    );
    render(<AllLayoutsPanel projectId={1} />);
    expect(screen.getByText("読み込み中...")).toBeInTheDocument();
  });

  it("shows_empty_state", () => {
    vi.mocked(useLayoutList).mockReturnValue(
      { data: [], isLoading: false } as unknown as ReturnType<typeof useLayoutList>
    );
    render(<AllLayoutsPanel projectId={1} />);
    expect(screen.getByText(/該当するレイアウトなし/)).toBeInTheDocument();
  });

  it("shows_layout_list", () => {
    vi.mocked(useLayoutList).mockReturnValue(
      { data: mockLayouts, isLoading: false } as unknown as ReturnType<typeof useLayoutList>
    );
    render(<AllLayoutsPanel projectId={1} />);
    expect(screen.getByText("Customer List")).toBeInTheDocument();
    expect(screen.getByText("Report")).toBeInTheDocument();
    expect(screen.getByText("Customers")).toBeInTheDocument();
  });

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
});
