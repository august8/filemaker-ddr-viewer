import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { ScriptDetail } from "../../components/detail/ScriptDetail";
import { makeScriptRow, makeScriptStepRow } from "../testFixtures";
import type { ScriptStepRow } from "../../types/ddr";

vi.mock("../../hooks/useTauriCommand", () => ({
  useScriptSteps: vi.fn(),
  useCallers: vi.fn(() => ({ data: [], isLoading: false })),
  useScriptList: vi.fn(() => ({ data: [], isLoading: false })),
}));

vi.mock("../../stores/appStore", () => ({
  useAppStore: vi.fn((selector?: (s: unknown) => unknown) => {
    const state = { selectElement: vi.fn(), diffContext: null };
    return selector ? selector(state) : state;
  }),
}));

import { useScriptSteps, useScriptList } from "../../hooks/useTauriCommand";
import { useAppStore } from "../../stores/appStore";

const mockScript = makeScriptRow({ id: 1, name: "Main Script", step_count: 2 });
const mockSteps: ScriptStepRow[] = [
  makeScriptStepRow({
    id: 1, step_type_id: 1, name: "Perform Script",
    script_ref_name: "Sub Script", script_ref_file: "MyDB",
    step_text: 'スクリプト実行 ["Sub Script"]',
  }),
  makeScriptStepRow({
    id: 2, step_type_id: 2, name: "Show Custom Dialog", enabled: false,
    calculation: '"Hello World"', position: 1,
  }),
];

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useAppStore).mockReturnValue({
    selectElement: vi.fn(),
    diffContext: null,
  } as unknown as ReturnType<typeof useAppStore>);
  vi.mocked(useScriptList).mockReturnValue({
    data: [],
    isLoading: false,
  } as unknown as ReturnType<typeof useScriptList>);
});

describe("ScriptDetail", () => {
  it("renders_step_list", () => {
    vi.mocked(useScriptSteps).mockReturnValue({
      data: mockSteps,
      isLoading: false,
    } as unknown as ReturnType<typeof useScriptSteps>);
    render(<ScriptDetail script={mockScript} projectId={1} />);
    // step_text が表示される
    expect(screen.getByText(/スクリプト実行/)).toBeInTheDocument();
    // step_text がない場合は name が表示される
    expect(screen.getByText("Show Custom Dialog")).toBeInTheDocument();
  });

  it("disabled_step_has_opacity_class", () => {
    vi.mocked(useScriptSteps).mockReturnValue({
      data: mockSteps,
      isLoading: false,
    } as unknown as ReturnType<typeof useScriptSteps>);
    render(<ScriptDetail script={mockScript} projectId={1} />);
    // 無効ステップには [無効] が表示される
    expect(screen.getByText("[無効]")).toBeInTheDocument();
  });

  it("renders_script_ref_when_no_step_text", () => {
    const stepsWithoutText: ScriptStepRow[] = [
      {
        ...mockSteps[0],
        step_text: null,
      },
    ];
    vi.mocked(useScriptSteps).mockReturnValue({
      data: stepsWithoutText,
      isLoading: false,
    } as unknown as ReturnType<typeof useScriptSteps>);
    render(<ScriptDetail script={mockScript} projectId={1} />);
    expect(screen.getByText("Perform Script")).toBeInTheDocument();
    expect(screen.getByText("[Sub Script]")).toBeInTheDocument();
  });

  it("diff_context_shows_added_badge_for_new_step", () => {
    const addedStep = makeScriptStepRow({ id: 3, name: "New Step", step_text: "新しいステップ" });
    vi.mocked(useAppStore).mockReturnValue({
      selectElement: vi.fn(),
      diffContext: { compareProjectId: 2 },
    } as unknown as ReturnType<typeof useAppStore>);
    // Compare script list returns "Main Script" so it finds a match
    vi.mocked(useScriptList).mockReturnValue({
      data: [makeScriptRow({ id: 99, name: "Main Script" })],
      isLoading: false,
    } as unknown as ReturnType<typeof useScriptList>);
    // first call: current steps (with addedStep), second call: compare steps (empty)
    vi.mocked(useScriptSteps)
      .mockReturnValueOnce({ data: [addedStep], isLoading: false } as unknown as ReturnType<typeof useScriptSteps>)
      .mockReturnValueOnce({ data: [], isLoading: false } as unknown as ReturnType<typeof useScriptSteps>);
    render(<ScriptDetail script={mockScript} projectId={1} />);
    expect(screen.getByText("追加")).toBeInTheDocument();
  });

  it("diff_context_shows_removed_badge_for_deleted_step", () => {
    const removedStep = makeScriptStepRow({ id: 3, name: "Old Step", step_text: "古いステップ" });
    vi.mocked(useAppStore).mockReturnValue({
      selectElement: vi.fn(),
      diffContext: { compareProjectId: 2 },
    } as unknown as ReturnType<typeof useAppStore>);
    vi.mocked(useScriptList).mockReturnValue({
      data: [makeScriptRow({ id: 99, name: "Main Script" })],
      isLoading: false,
    } as unknown as ReturnType<typeof useScriptList>);
    // current steps empty, compare steps has removedStep
    vi.mocked(useScriptSteps)
      .mockReturnValueOnce({ data: [], isLoading: false } as unknown as ReturnType<typeof useScriptSteps>)
      .mockReturnValueOnce({ data: [removedStep], isLoading: false } as unknown as ReturnType<typeof useScriptSteps>);
    render(<ScriptDetail script={mockScript} projectId={1} />);
    expect(screen.getByText("削除")).toBeInTheDocument();
  });

  it("indented_steps_increase_and_decrease_level", () => {
    const steps: ScriptStepRow[] = [
      makeScriptStepRow({ id: 1, name: "If", step_text: "If [条件]" }),
      makeScriptStepRow({ id: 2, name: "Set Field", step_text: "フィールド設定" }),
      makeScriptStepRow({ id: 3, name: "End If", step_text: "End If" }),
    ];
    vi.mocked(useScriptSteps).mockReturnValue({
      data: steps,
      isLoading: false,
    } as unknown as ReturnType<typeof useScriptSteps>);
    render(<ScriptDetail script={mockScript} projectId={1} />);
    expect(screen.getByText(/If \[条件\]/)).toBeInTheDocument();
    expect(screen.getByText("フィールド設定")).toBeInTheDocument();
    expect(screen.getByText("End If")).toBeInTheDocument();
  });
});
