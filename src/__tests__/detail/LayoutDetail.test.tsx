import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { LayoutDetail } from "../../components/detail/LayoutDetail";
import { makeLayoutRow, makeTriggerRow, makeLayoutObjectRow } from "../testFixtures";

vi.mock("../../hooks/useTauriCommand", () => ({
  useLayoutTriggers: vi.fn(),
  useLayoutObjects: vi.fn(),
  useScriptList: vi.fn(),
  useLayoutList: vi.fn(() => ({ data: [], isLoading: false })),
}));

vi.mock("../../stores/appStore", () => ({
  useAppStore: vi.fn(() => ({
    setRightPanel: vi.fn(),
    rightPanel: null,
    selectElement: vi.fn(),
    diffContext: null,
  })),
}));

import {
  useLayoutTriggers,
  useLayoutObjects,
  useScriptList,
  useLayoutList,
} from "../../hooks/useTauriCommand";
import { useAppStore } from "../../stores/appStore";

const mockLayout = makeLayoutRow({
  id: 1,
  name: "Contact Layout",
  table_occurrence_name: "Contact",
  trigger_count: 1,
});
beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useAppStore).mockReturnValue({
    setRightPanel: vi.fn(),
    rightPanel: null,
    selectElement: vi.fn(),
    diffContext: null,
  } as unknown as ReturnType<typeof useAppStore>);
  vi.mocked(useLayoutObjects).mockReturnValue({
    data: [],
    isLoading: false,
  } as unknown as ReturnType<typeof useLayoutObjects>);
  vi.mocked(useScriptList).mockReturnValue({
    data: [],
    isLoading: false,
  } as unknown as ReturnType<typeof useScriptList>);
  vi.mocked(useLayoutList).mockReturnValue({
    data: [],
    isLoading: false,
  } as unknown as ReturnType<typeof useLayoutList>);
});

describe("LayoutDetail", () => {
  it("clicking_object_row_calls_setRightPanel", () => {
    const mockSetRightPanel = vi.fn();
    vi.mocked(useAppStore).mockReturnValue({
      setRightPanel: mockSetRightPanel,
      rightPanel: null,
      selectElement: vi.fn(),
      diffContext: null,
    } as unknown as ReturnType<typeof useAppStore>);
    vi.mocked(useLayoutTriggers).mockReturnValue({
      data: [],
      isLoading: false,
    } as unknown as ReturnType<typeof useLayoutTriggers>);
    vi.mocked(useLayoutObjects).mockReturnValue({
      data: [makeLayoutObjectRow({ id: 5, object_type: "Button", object_key: 5, object_name: "SubmitBtn" })],
      isLoading: false,
    } as unknown as ReturnType<typeof useLayoutObjects>);
    render(<LayoutDetail layout={mockLayout} projectId={1} />);
    const row = screen.getByText("Button").closest("tr")!;
    fireEvent.click(row);
    expect(mockSetRightPanel).toHaveBeenCalledWith({
      kind: "layout_object",
      layoutObjectId: 5,
      layoutId: 1,
    });
  });

  it("trigger_with_matching_script_renders_button", () => {
    const mockSelectElement = vi.fn();
    vi.mocked(useAppStore).mockReturnValue({
      setRightPanel: vi.fn(),
      rightPanel: null,
      selectElement: mockSelectElement,
      diffContext: null,
    } as unknown as ReturnType<typeof useAppStore>);
    vi.mocked(useLayoutTriggers).mockReturnValue({
      data: [makeTriggerRow({ id: 1, event: "OnRecordLoad", script_name: "Load Data", file_name: "" })],
      isLoading: false,
    } as unknown as ReturnType<typeof useLayoutTriggers>);
    vi.mocked(useScriptList).mockReturnValue({
      data: [{ id: 10, fm_id: 10, name: "Load Data", run_with_full_access: false, step_count: 3 }],
      isLoading: false,
    } as unknown as ReturnType<typeof useScriptList>);
    render(<LayoutDetail layout={mockLayout} projectId={1} />);
    const btn = screen.getByRole("button", { name: "Load Data" });
    fireEvent.click(btn);
    expect(mockSelectElement).toHaveBeenCalledWith({
      kind: "script",
      projectId: 1,
      id: 10,
      name: "Load Data",
    });
  });

  it("diff_context_shows_added_badge_for_new_object", () => {
    const newObj = makeLayoutObjectRow({ id: 1, object_key: 1, object_type: "Field" });
    vi.mocked(useAppStore).mockReturnValue({
      setRightPanel: vi.fn(),
      rightPanel: null,
      selectElement: vi.fn(),
      diffContext: { compareProjectId: 2 },
    } as unknown as ReturnType<typeof useAppStore>);
    vi.mocked(useLayoutTriggers).mockReturnValue({
      data: [],
      isLoading: false,
    } as unknown as ReturnType<typeof useLayoutTriggers>);
    vi.mocked(useLayoutList).mockReturnValue({
      data: [{ id: 20, fm_id: 20, name: "Contact Layout", table_occurrence_name: null, trigger_count: 0 }],
      isLoading: false,
    } as unknown as ReturnType<typeof useLayoutList>);
    // first call: current project objects; second call: compare project objects (empty)
    vi.mocked(useLayoutObjects)
      .mockReturnValueOnce({ data: [newObj], isLoading: false } as unknown as ReturnType<typeof useLayoutObjects>)
      .mockReturnValueOnce({ data: [], isLoading: false } as unknown as ReturnType<typeof useLayoutObjects>);
    render(<LayoutDetail layout={mockLayout} projectId={1} />);
    expect(screen.getByText("追加")).toBeInTheDocument();
  });
});
