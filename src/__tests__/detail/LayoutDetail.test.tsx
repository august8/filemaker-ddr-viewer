import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { LayoutDetail } from "../../components/detail/LayoutDetail";
import { makeLayoutRow, makeTriggerRow } from "../testFixtures";

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

import { useLayoutTriggers, useLayoutObjects, useScriptList } from "../../hooks/useTauriCommand";

const mockLayout = makeLayoutRow({ id: 1, name: "Contact Layout", table_occurrence_name: "Contact", trigger_count: 1 });
const mockTriggers = [makeTriggerRow({ id: 1, event: "OnRecordLoad", script_name: "Load Data", file_name: "MyDB" })];

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useLayoutObjects).mockReturnValue({
    data: [],
    isLoading: false,
  } as unknown as ReturnType<typeof useLayoutObjects>);
  vi.mocked(useScriptList).mockReturnValue({
    data: [],
    isLoading: false,
  } as unknown as ReturnType<typeof useScriptList>);
});

describe("LayoutDetail", () => {
  it("renders_table_occurrence", () => {
    vi.mocked(useLayoutTriggers).mockReturnValue({
      data: mockTriggers,
      isLoading: false,
    } as unknown as ReturnType<typeof useLayoutTriggers>);
    render(<LayoutDetail layout={mockLayout} projectId={1} />);
    expect(screen.getByText("Contact")).toBeInTheDocument();
  });

  it("renders_trigger_list", () => {
    vi.mocked(useLayoutTriggers).mockReturnValue({
      data: mockTriggers,
      isLoading: false,
    } as unknown as ReturnType<typeof useLayoutTriggers>);
    render(<LayoutDetail layout={mockLayout} projectId={1} />);
    expect(screen.getByText("OnRecordLoad")).toBeInTheDocument();
    expect(screen.getByText("Load Data")).toBeInTheDocument();
  });

  it("renders_no_triggers_message", () => {
    vi.mocked(useLayoutTriggers).mockReturnValue({
      data: [],
      isLoading: false,
    } as unknown as ReturnType<typeof useLayoutTriggers>);
    const layoutNoTriggers = { ...mockLayout, trigger_count: 0 };
    render(<LayoutDetail layout={layoutNoTriggers} projectId={1} />);
    expect(screen.getByText("トリガーなし")).toBeInTheDocument();
  });
});
