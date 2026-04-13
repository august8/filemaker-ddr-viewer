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

import { useScriptSteps } from "../../hooks/useTauriCommand";

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

  it("renders_full_access_badge", () => {
    const fullAccessScript = { ...mockScript, run_with_full_access: true };
    vi.mocked(useScriptSteps).mockReturnValue({
      data: [],
      isLoading: false,
    } as unknown as ReturnType<typeof useScriptSteps>);
    render(<ScriptDetail script={fullAccessScript} projectId={1} />);
    expect(screen.getByText("完全アクセス権で実行")).toBeInTheDocument();
  });
});
