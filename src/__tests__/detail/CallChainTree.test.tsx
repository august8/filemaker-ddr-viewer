import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { CallChainTree } from "../../components/detail/CallChainTree";
import type { CallChainNode, ScriptRow } from "../../types/ddr";

vi.mock("../../hooks/script", () => ({
  useCallChain: vi.fn(),
  useScriptList: vi.fn(),
}));

vi.mock("../../stores/appStore", () => ({
  useAppStore: vi.fn((selector: (s: { selectElement: ReturnType<typeof vi.fn> }) => unknown) =>
    selector({ selectElement: mockSelectElement })
  ),
}));

const mockSelectElement = vi.fn();

import { useCallChain, useScriptList } from "../../hooks/script";

const mockScripts: ScriptRow[] = [
  { id: 10, fm_id: 1, name: "Root Script", run_with_full_access: false, step_count: 2 },
  { id: 11, fm_id: 2, name: "Child Script", run_with_full_access: false, step_count: 1 },
];

const mockChain: CallChainNode = {
  script_id: 1,
  script_name: "Root Script",
  depth: 0,
  is_cycle: false,
  children: [
    {
      script_id: 2,
      script_name: "Child Script",
      depth: 1,
      is_cycle: false,
      children: [],
    },
  ],
};

const cycleChain: CallChainNode = {
  script_id: 1,
  script_name: "Root Script",
  depth: 0,
  is_cycle: false,
  children: [
    {
      script_id: 1,
      script_name: "Root Script",
      depth: 1,
      is_cycle: true,
      children: [],
    },
  ],
};

beforeEach(() => {
  vi.clearAllMocks();
});

describe("CallChainTree", () => {
  it("renders_root_and_child_nodes", () => {
    vi.mocked(useCallChain).mockReturnValue(
      { data: mockChain, isLoading: false, isError: false } as unknown as ReturnType<typeof useCallChain>
    );
    vi.mocked(useScriptList).mockReturnValue(
      { data: mockScripts } as unknown as ReturnType<typeof useScriptList>
    );
    render(<CallChainTree projectId={1} scriptFmId={1} />);
    expect(screen.getAllByText("Root Script").length).toBeGreaterThan(0);
    expect(screen.getByText("Child Script")).toBeInTheDocument();
  });

  it("shows_cycle_indicator", () => {
    vi.mocked(useCallChain).mockReturnValue(
      { data: cycleChain, isLoading: false, isError: false } as unknown as ReturnType<typeof useCallChain>
    );
    vi.mocked(useScriptList).mockReturnValue(
      { data: mockScripts } as unknown as ReturnType<typeof useScriptList>
    );
    render(<CallChainTree projectId={1} scriptFmId={1} />);
    expect(screen.getByText(/循環/)).toBeInTheDocument();
  });

  it("navigates_to_script_on_click", () => {
    vi.mocked(useCallChain).mockReturnValue(
      { data: mockChain, isLoading: false, isError: false } as unknown as ReturnType<typeof useCallChain>
    );
    vi.mocked(useScriptList).mockReturnValue(
      { data: mockScripts } as unknown as ReturnType<typeof useScriptList>
    );
    render(<CallChainTree projectId={1} scriptFmId={1} />);
    fireEvent.click(screen.getByText("Child Script"));
    expect(mockSelectElement).toHaveBeenCalledWith({
      kind: "script",
      projectId: 1,
      id: 11,
      name: "Child Script",
    });
  });

  it("shows_loading_state", () => {
    vi.mocked(useCallChain).mockReturnValue(
      { data: undefined, isLoading: true, isError: false } as unknown as ReturnType<typeof useCallChain>
    );
    vi.mocked(useScriptList).mockReturnValue(
      { data: [] } as unknown as ReturnType<typeof useScriptList>
    );
    render(<CallChainTree projectId={1} scriptFmId={1} />);
    expect(screen.getByText("読み込み中...")).toBeInTheDocument();
  });

  it("collapses_children_on_toggle", () => {
    vi.mocked(useCallChain).mockReturnValue(
      { data: mockChain, isLoading: false, isError: false } as unknown as ReturnType<typeof useCallChain>
    );
    vi.mocked(useScriptList).mockReturnValue(
      { data: mockScripts } as unknown as ReturnType<typeof useScriptList>
    );
    render(<CallChainTree projectId={1} scriptFmId={1} />);
    // 子が最初は表示されている
    expect(screen.getByText("Child Script")).toBeInTheDocument();
    // 展開ボタンをクリックして折りたたむ
    fireEvent.click(screen.getByText("▾"));
    expect(screen.queryByText("Child Script")).not.toBeInTheDocument();
  });
});
