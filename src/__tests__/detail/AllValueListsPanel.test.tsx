import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { AllValueListsPanel } from "../../components/detail/AllValueListsPanel";
import { makeValueListRow } from "../testFixtures";

vi.mock("../../hooks/useTauriCommand", () => ({
  useValueListList: vi.fn(),
}));

vi.mock("../../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

import { useValueListList } from "../../hooks/useTauriCommand";
import { useAppStore } from "../../stores/appStore";

const mockValueLists = [
  makeValueListRow({ id: 1, fm_id: 1, name: "Status", item_count: 3 }),
  makeValueListRow({ id: 2, fm_id: 2, name: "Category", item_count: 5 }),
];

const mockSelectElement = vi.fn();

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useAppStore).mockReturnValue({
    selectElement: mockSelectElement,
  } as unknown as ReturnType<typeof useAppStore>);
});

describe("AllValueListsPanel", () => {
  it("shows_loading_state", () => {
    vi.mocked(useValueListList).mockReturnValue(
      { data: undefined, isLoading: true } as unknown as ReturnType<typeof useValueListList>
    );
    render(<AllValueListsPanel projectId={1} />);
    expect(screen.getByText("読み込み中...")).toBeInTheDocument();
  });

  it("shows_empty_state", () => {
    vi.mocked(useValueListList).mockReturnValue(
      { data: [], isLoading: false } as unknown as ReturnType<typeof useValueListList>
    );
    render(<AllValueListsPanel projectId={1} />);
    expect(screen.getByText(/該当するバリューリストなし/)).toBeInTheDocument();
  });

  it("shows_value_list", () => {
    vi.mocked(useValueListList).mockReturnValue(
      { data: mockValueLists, isLoading: false } as unknown as ReturnType<typeof useValueListList>
    );
    render(<AllValueListsPanel projectId={1} />);
    expect(screen.getByText("Status")).toBeInTheDocument();
    expect(screen.getByText("Category")).toBeInTheDocument();
  });

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
});
