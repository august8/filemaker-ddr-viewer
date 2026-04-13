import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { AllScriptsPanel } from "../../components/detail/AllScriptsPanel";
import { makeScriptRow } from "../testFixtures";

vi.mock("../../hooks/useTauriCommand", () => ({
  useScriptList: vi.fn(),
}));

vi.mock("../../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

import { useScriptList } from "../../hooks/useTauriCommand";
import { useAppStore } from "../../stores/appStore";

const mockScripts = [
  makeScriptRow({ id: 1, fm_id: 1, name: "Save Record", step_count: 5 }),
  makeScriptRow({ id: 2, fm_id: 2, name: "Print Report", run_with_full_access: true, step_count: 12 }),
];

const mockSelectElement = vi.fn();

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useAppStore).mockReturnValue({
    selectElement: mockSelectElement,
  } as unknown as ReturnType<typeof useAppStore>);
});

describe("AllScriptsPanel", () => {
  it("shows_loading_state", () => {
    vi.mocked(useScriptList).mockReturnValue(
      { data: undefined, isLoading: true } as unknown as ReturnType<typeof useScriptList>
    );
    render(<AllScriptsPanel projectId={1} />);
    expect(screen.getByText("読み込み中...")).toBeInTheDocument();
  });

  it("shows_empty_state", () => {
    vi.mocked(useScriptList).mockReturnValue(
      { data: [], isLoading: false } as unknown as ReturnType<typeof useScriptList>
    );
    render(<AllScriptsPanel projectId={1} />);
    expect(screen.getByText(/該当するスクリプトなし/)).toBeInTheDocument();
  });

  it("shows_script_list", () => {
    vi.mocked(useScriptList).mockReturnValue(
      { data: mockScripts, isLoading: false } as unknown as ReturnType<typeof useScriptList>
    );
    render(<AllScriptsPanel projectId={1} />);
    expect(screen.getByText("Save Record")).toBeInTheDocument();
    expect(screen.getByText("Print Report")).toBeInTheDocument();
    expect(screen.getByText("5")).toBeInTheDocument();
    expect(screen.getByText("12")).toBeInTheDocument();
  });

  it("row_click_navigates_to_script", () => {
    vi.mocked(useScriptList).mockReturnValue(
      { data: mockScripts, isLoading: false } as unknown as ReturnType<typeof useScriptList>
    );
    render(<AllScriptsPanel projectId={1} />);
    fireEvent.click(screen.getByText("Save Record"));
    expect(mockSelectElement).toHaveBeenCalledWith({
      kind: "script",
      projectId: 1,
      id: 1,
      name: "Save Record",
    });
  });
});
