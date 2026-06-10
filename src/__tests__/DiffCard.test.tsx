import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { DiffCard } from "../components/DiffCard";
import type { DiffResult } from "../types/ddr";
import type { DiffStateData } from "../stores/appStore";

vi.mock("../hooks/solutions", () => ({ useAllProjects: vi.fn() }));
vi.mock("../hooks/diff", () => ({ useCompareSolutions: vi.fn() }));
vi.mock("../stores/appStore", () => ({ useAppStore: vi.fn() }));
vi.mock("../hooks/analysis", () => ({ useResolveElementByName: vi.fn() }));

import { useAllProjects } from "../hooks/solutions";
import { useCompareSolutions } from "../hooks/diff";
import { useAppStore } from "../stores/appStore";
import { useResolveElementByName } from "../hooks/analysis";

const mockDiffResult: DiffResult = {
  items: [
    {
      kind: "Added",
      element_type: "script",
      name: "NewScript",
      detail: null,
      project_id: 2,
      compare_project_id: null,
    },
  ],
  added_count: 1,
  removed_count: 0,
  modified_count: 0,
};

const mockSetDiffState = vi.fn();
const mockSelectElement = vi.fn();
const mockNavigateFromDiff = vi.fn();
const mockResolve = vi.fn();

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useAllProjects).mockReturnValue({
    data: [],
    isLoading: false,
  } as unknown as ReturnType<typeof useAllProjects>);
  vi.mocked(useCompareSolutions).mockReturnValue({
    data: mockDiffResult,
    isLoading: false,
  } as unknown as ReturnType<typeof useCompareSolutions>);
  mockResolve.mockResolvedValue({ id: 1, name: "NewScript" });
  vi.mocked(useResolveElementByName).mockReturnValue(mockResolve);
});

describe("DiffCard", () => {
  it("resolve_element_called_when_navigable_item_clicked", async () => {
    vi.mocked(useAppStore).mockReturnValue({
      diffState: {
        solA: 1,
        solB: 2,
        committedA: 1,
        committedB: 2,
        expandedTypes: ["script"],
      } as DiffStateData,
      setDiffState: mockSetDiffState,
      selectElement: mockSelectElement,
      navigateFromDiff: mockNavigateFromDiff,
    } as unknown as ReturnType<typeof useAppStore>);

    render(<DiffCard />);
    fireEvent.click(screen.getByText("NewScript"));

    await waitFor(() => {
      expect(mockResolve).toHaveBeenCalledWith(2, "script", "NewScript");
    });
  });
});
