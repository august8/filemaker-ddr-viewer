import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { BrokenRefsList } from "../components/BrokenRefsList";
import type { BrokenRef } from "../types/ddr";

vi.mock("../hooks/analysis", () => ({
  useBrokenRefs: vi.fn(),
}));
vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

import { useBrokenRefs } from "../hooks/analysis";
import { useAppStore } from "../stores/appStore";

const mockSelectElement = vi.fn();

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useAppStore).mockReturnValue({
    selectElement: mockSelectElement,
  } as unknown as ReturnType<typeof useAppStore>);
});

describe("BrokenRefsList", () => {
  it("returns_null_when_project_id_is_null", () => {
    vi.mocked(useBrokenRefs).mockReturnValue(
      { data: [], isLoading: false } as unknown as ReturnType<typeof useBrokenRefs>
    );
    const { container } = render(<BrokenRefsList projectId={null} />);
    expect(container.firstChild).toBeNull();
  });

  it("shows_spinner_while_loading", () => {
    vi.mocked(useBrokenRefs).mockReturnValue(
      { data: undefined, isLoading: true } as unknown as ReturnType<typeof useBrokenRefs>
    );
    render(<BrokenRefsList projectId={1} />);
    expect(screen.getByText("読み込み中...")).toBeInTheDocument();
  });

  it("shows_empty_message_when_no_refs", () => {
    vi.mocked(useBrokenRefs).mockReturnValue(
      { data: [], isLoading: false } as unknown as ReturnType<typeof useBrokenRefs>
    );
    render(<BrokenRefsList projectId={1} />);
    expect(screen.getByText("壊れた参照はありません")).toBeInTheDocument();
  });

  it("shows_kind_label_for_each_ref", () => {
    const refs: BrokenRef[] = [
      { kind: "performScript",   source_name: "MainScript",   target_script_name: "Missing" },
      { kind: "scriptTrigger",   source_name: "CustomerList", target_script_name: "Missing" },
      { kind: "brokenFieldRef",  source_name: "MainScript",   target_script_name: "Set Field [...]" },
      { kind: "brokenLayoutRef", source_name: "NavScript",    target_script_name: "Go to Layout [...]" },
      { kind: "unknownRef",      source_name: "OtherScript",  target_script_name: "ファイルを開く [<不明>]" },
    ];
    vi.mocked(useBrokenRefs).mockReturnValue(
      { data: refs, isLoading: false } as unknown as ReturnType<typeof useBrokenRefs>
    );
    render(<BrokenRefsList projectId={1} />);
    expect(screen.getByText("Perform Script")).toBeInTheDocument();
    expect(screen.getByText("Script Trigger")).toBeInTheDocument();
    expect(screen.getByText("壊れたフィールド参照")).toBeInTheDocument();
    expect(screen.getByText("壊れたレイアウト参照")).toBeInTheDocument();
    expect(screen.getByText("参照先不明")).toBeInTheDocument();
  });

  it("perform_script_click_selects_script", () => {
    const refs: BrokenRef[] = [
      { kind: "performScript", source_name: "MainScript", source_id: 1, target_script_name: "Missing" },
    ];
    vi.mocked(useBrokenRefs).mockReturnValue(
      { data: refs, isLoading: false } as unknown as ReturnType<typeof useBrokenRefs>
    );
    render(<BrokenRefsList projectId={1} />);
    fireEvent.click(screen.getByTitle("MainScript を表示"));
    expect(mockSelectElement).toHaveBeenCalledWith({
      kind: "script", projectId: 1, id: 1, name: "MainScript",
    });
  });

  it("script_trigger_click_selects_layout", () => {
    const refs: BrokenRef[] = [
      { kind: "scriptTrigger", source_name: "CustomerList", source_id: 10, target_script_name: "Missing" },
    ];
    vi.mocked(useBrokenRefs).mockReturnValue(
      { data: refs, isLoading: false } as unknown as ReturnType<typeof useBrokenRefs>
    );
    render(<BrokenRefsList projectId={1} />);
    fireEvent.click(screen.getByTitle("CustomerList を表示"));
    expect(mockSelectElement).toHaveBeenCalledWith({
      kind: "layout", projectId: 1, id: 10, name: "CustomerList",
    });
  });

  it("broken_field_ref_click_selects_script", () => {
    const refs: BrokenRef[] = [
      { kind: "brokenFieldRef", source_name: "MainScript", source_id: 1, target_script_name: "Set Field [...]" },
    ];
    vi.mocked(useBrokenRefs).mockReturnValue(
      { data: refs, isLoading: false } as unknown as ReturnType<typeof useBrokenRefs>
    );
    render(<BrokenRefsList projectId={1} />);
    fireEvent.click(screen.getByTitle("MainScript を表示"));
    expect(mockSelectElement).toHaveBeenCalledWith({
      kind: "script", projectId: 1, id: 1, name: "MainScript",
    });
  });

  it("broken_layout_ref_click_selects_script", () => {
    const refs: BrokenRef[] = [
      { kind: "brokenLayoutRef", source_name: "NavScript", source_id: 2, target_script_name: "Go to Layout [...]" },
    ];
    vi.mocked(useBrokenRefs).mockReturnValue(
      { data: refs, isLoading: false } as unknown as ReturnType<typeof useBrokenRefs>
    );
    render(<BrokenRefsList projectId={1} />);
    fireEvent.click(screen.getByTitle("NavScript を表示"));
    expect(mockSelectElement).toHaveBeenCalledWith({
      kind: "script", projectId: 1, id: 2, name: "NavScript",
    });
  });

  it("unknown_ref_click_selects_script", () => {
    const refs: BrokenRef[] = [
      { kind: "unknownRef", source_name: "OtherScript", source_id: 5, target_script_name: "ファイルを開く [<不明>]" },
    ];
    vi.mocked(useBrokenRefs).mockReturnValue(
      { data: refs, isLoading: false } as unknown as ReturnType<typeof useBrokenRefs>
    );
    render(<BrokenRefsList projectId={1} />);
    fireEvent.click(screen.getByTitle("OtherScript を表示"));
    expect(mockSelectElement).toHaveBeenCalledWith({
      kind: "script", projectId: 1, id: 5, name: "OtherScript",
    });
  });

  it("broken_field_placement_shows_label", () => {
    const refs: BrokenRef[] = [
      { kind: "brokenFieldPlacement", source_name: "CustomerLayout", source_id: 20, target_script_name: "(フィールド削除済み) #7" },
    ];
    vi.mocked(useBrokenRefs).mockReturnValue(
      { data: refs as BrokenRef[], isLoading: false } as unknown as ReturnType<typeof useBrokenRefs>
    );
    render(<BrokenRefsList projectId={1} />);
    expect(screen.getByText("レイアウト上の削除フィールド")).toBeInTheDocument();
  });

  it("broken_field_placement_click_selects_layout", () => {
    const refs = [
      { kind: "brokenFieldPlacement" as const, source_name: "CustomerLayout", source_id: 20, target_script_name: "(フィールド削除済み) #7" },
    ] as BrokenRef[];
    vi.mocked(useBrokenRefs).mockReturnValue(
      { data: refs, isLoading: false } as unknown as ReturnType<typeof useBrokenRefs>
    );
    render(<BrokenRefsList projectId={1} />);
    fireEvent.click(screen.getByTitle("CustomerLayout を表示"));
    expect(mockSelectElement).toHaveBeenCalledWith({
      kind: "layout", projectId: 1, id: 20, name: "CustomerLayout",
    });
  });

  it("click_does_nothing_when_source_id_is_null", () => {
    const refs: BrokenRef[] = [
      { kind: "performScript", source_name: "MainScript", target_script_name: "Missing" },
    ];
    vi.mocked(useBrokenRefs).mockReturnValue(
      { data: refs, isLoading: false } as unknown as ReturnType<typeof useBrokenRefs>
    );
    render(<BrokenRefsList projectId={1} />);
    fireEvent.click(screen.getByRole("button", { name: /MainScript/ }));
    expect(mockSelectElement).not.toHaveBeenCalled();
  });

  it("button_is_disabled_when_source_id_is_null", () => {
    const refs: BrokenRef[] = [
      { kind: "performScript", source_name: "MainScript", target_script_name: "Missing" },
    ];
    vi.mocked(useBrokenRefs).mockReturnValue(
      { data: refs, isLoading: false } as unknown as ReturnType<typeof useBrokenRefs>
    );
    render(<BrokenRefsList projectId={1} />);
    const btn = screen.getByRole("button", { name: /MainScript/ });
    expect(btn).toBeDisabled();
  });
});
