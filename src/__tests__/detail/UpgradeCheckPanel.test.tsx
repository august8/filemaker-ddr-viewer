import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { UpgradeCheckPanel } from "../../components/detail/UpgradeCheckPanel";
import type { BrokenRefWithProject, UpgradeHit } from "../../types/ddr";

vi.mock("../../hooks/analysis", () => ({
  useUpgradeCheck: vi.fn(),
  useSolutionBrokenRefs: vi.fn(),
}));

vi.mock("../../stores/appStore", () => ({
  useAppStore: vi.fn(() => ({
    checkItems: [
      { id: "item1", label: "Perform Script", enabled: true, detectionType: "step_type_id", detectionValue: "89" },
    ],
    showBrokenRefsInUpgradeCheck: false,
    selectElement: vi.fn(),
    setRightPanel: vi.fn(),
  })),
}));

// @tauri-apps/plugin-dialog と @tauri-apps/api/core はテスト環境で使えないためモック
vi.mock("@tauri-apps/plugin-dialog", () => ({ save: vi.fn().mockResolvedValue(null) }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

import { useUpgradeCheck, useSolutionBrokenRefs } from "../../hooks/analysis";
import { useAppStore } from "../../stores/appStore";

beforeEach(() => {
  vi.mocked(useAppStore).mockImplementation(() => ({
    checkItems: [
      { id: "item1", label: "Perform Script", enabled: true, detectionType: "step_type_id", detectionValue: "89" },
    ],
    showBrokenRefsInUpgradeCheck: false,
    selectElement: vi.fn(),
    setRightPanel: vi.fn(),
  } as unknown as ReturnType<typeof useAppStore>));
  vi.mocked(useSolutionBrokenRefs).mockReturnValue(
    { data: [], isLoading: false } as unknown as ReturnType<typeof useSolutionBrokenRefs>
  );
});

const mockHits: UpgradeHit[] = [
  {
    item_id: "item1",
    project_id: 1,
    project_name: "DB_A",
    custom_function_name: null,
    script_id: 10,
    script_name: "Script A",
    step_id: 100,
    step_name: "Perform Script",
    step_text: "Perform Script [\"Sub\"]",
    field_id: null,
    field_name: null,
    table_id: null,
    table_name: null,
  },
  {
    item_id: "item1",
    project_id: 2,
    project_name: "DB_B",
    custom_function_name: null,
    script_id: 20,
    script_name: "Script B",
    step_id: 200,
    step_name: "Perform Script",
    step_text: "Perform Script [\"Other\"]",
    field_id: null,
    field_name: null,
    table_id: null,
    table_name: null,
  },
];

describe("UpgradeCheckPanel", () => {
  it("shows_group_header_with_count", () => {
    vi.mocked(useUpgradeCheck).mockReturnValue(
      { data: mockHits, isLoading: false } as unknown as ReturnType<typeof useUpgradeCheck>
    );
    render(<UpgradeCheckPanel solutionId={1} />);
    // アコーディオンヘッダーボタンにラベルと件数が表示される
    const btn = screen.getByRole("button", { name: /Perform Script/ });
    expect(btn).toBeInTheDocument();
    expect(screen.getByText("2 件")).toBeInTheDocument();
  });

  it("accordion_collapsed_by_default", () => {
    vi.mocked(useUpgradeCheck).mockReturnValue(
      { data: mockHits, isLoading: false } as unknown as ReturnType<typeof useUpgradeCheck>
    );
    render(<UpgradeCheckPanel solutionId={1} />);
    // デフォルトではヒット内容は非表示
    expect(screen.queryByText("Script A")).not.toBeInTheDocument();
  });

  it("accordion_expands_on_click", async () => {
    vi.mocked(useUpgradeCheck).mockReturnValue(
      { data: mockHits, isLoading: false } as unknown as ReturnType<typeof useUpgradeCheck>
    );
    const user = userEvent.setup();
    render(<UpgradeCheckPanel solutionId={1} />);
    await user.click(screen.getByRole("button", { name: /Perform Script/ }));
    expect(screen.getByText("Script A")).toBeInTheDocument();
    expect(screen.getByText("Script B")).toBeInTheDocument();
  });

  it("shows_project_name_after_expand", async () => {
    vi.mocked(useUpgradeCheck).mockReturnValue(
      { data: mockHits, isLoading: false } as unknown as ReturnType<typeof useUpgradeCheck>
    );
    const user = userEvent.setup();
    render(<UpgradeCheckPanel solutionId={1} />);
    await user.click(screen.getByRole("button", { name: /Perform Script/ }));
    // DB_A はサマリーグリッドヘッダーとヒット行の両方に現れる
    expect(screen.getAllByText("DB_A").length).toBeGreaterThan(0);
    expect(screen.getAllByText("DB_B").length).toBeGreaterThan(0);
  });

  it("shows_no_hits_message_when_empty", () => {
    vi.mocked(useUpgradeCheck).mockReturnValue(
      { data: [], isLoading: false } as unknown as ReturnType<typeof useUpgradeCheck>
    );
    render(<UpgradeCheckPanel solutionId={1} />);
    // hits が 0 件でも有効なチェック項目はアコーディオンに表示される
    expect(screen.getByText("0 件")).toBeInTheDocument();
  });

  it("csv_export_button_is_present_when_hits_exist", () => {
    vi.mocked(useUpgradeCheck).mockReturnValue(
      { data: mockHits, isLoading: false } as unknown as ReturnType<typeof useUpgradeCheck>
    );
    render(<UpgradeCheckPanel solutionId={1} />);
    expect(screen.getByRole("button", { name: /CSV/i })).toBeInTheDocument();
  });

  it("csv_export_button_is_disabled_when_no_hits", () => {
    vi.mocked(useUpgradeCheck).mockReturnValue(
      { data: [], isLoading: false } as unknown as ReturnType<typeof useUpgradeCheck>
    );
    render(<UpgradeCheckPanel solutionId={1} />);
    expect(screen.getByRole("button", { name: /CSV/i })).toBeDisabled();
  });

  it("click_script_hit_calls_selectElement", async () => {
    const selectElement = vi.fn();
    vi.mocked(useAppStore).mockImplementation(() => ({
      checkItems: [
        { id: "item1", label: "Perform Script", enabled: true, detectionType: "step_type_id", detectionValue: "89" },
      ],
      showBrokenRefsInUpgradeCheck: false,
      selectElement,
      setRightPanel: vi.fn(),
    } as unknown as ReturnType<typeof useAppStore>));
    vi.mocked(useUpgradeCheck).mockReturnValue(
      { data: mockHits, isLoading: false } as unknown as ReturnType<typeof useUpgradeCheck>
    );
    const user = userEvent.setup();
    render(<UpgradeCheckPanel solutionId={1} />);
    // アコーディオンを展開してからクリック
    await user.click(screen.getByRole("button", { name: /Perform Script/ }));
    await user.click(screen.getByText("Script A"));
    expect(selectElement).toHaveBeenCalledWith({
      kind: "script",
      projectId: 1,
      id: 10,
      name: "Script A",
    });
  });
});

const mockBrokenRefs: BrokenRefWithProject[] = [
  {
    kind: "performScript",
    source_name: "MainScript",
    target_script_name: "DeletedScript",
    project_id: 1,
    project_name: "DB_A",
  },
  {
    kind: "scriptTrigger",
    source_name: "Layout1",
    target_script_name: "MissingScript",
    project_id: 2,
    project_name: "DB_B",
  },
];

describe("壊れた参照セクション", () => {
  beforeEach(() => {
    vi.mocked(useSolutionBrokenRefs).mockReturnValue(
      { data: mockBrokenRefs, isLoading: false } as unknown as ReturnType<typeof useSolutionBrokenRefs>
    );
    vi.mocked(useAppStore).mockImplementation(() => ({
      checkItems: [],
      showBrokenRefsInUpgradeCheck: true,
      selectElement: vi.fn(),
      setRightPanel: vi.fn(),
    } as unknown as ReturnType<typeof useAppStore>));
    vi.mocked(useUpgradeCheck).mockReturnValue(
      { data: [], isLoading: false } as unknown as ReturnType<typeof useUpgradeCheck>
    );
  });

  it("broken_refs_section_shown_when_enabled", () => {
    render(<UpgradeCheckPanel solutionId={1} />);
    expect(screen.getByRole("button", { name: /壊れた参照/ })).toBeInTheDocument();
  });

  it("broken_refs_section_hidden_when_disabled", () => {
    vi.mocked(useAppStore).mockImplementation(() => ({
      checkItems: [],
      showBrokenRefsInUpgradeCheck: false,
      selectElement: vi.fn(),
      setRightPanel: vi.fn(),
    } as unknown as ReturnType<typeof useAppStore>));
    render(<UpgradeCheckPanel solutionId={1} />);
    expect(screen.queryByRole("button", { name: /壊れた参照/ })).not.toBeInTheDocument();
  });

  it("broken_refs_section_shows_count", () => {
    render(<UpgradeCheckPanel solutionId={1} />);
    expect(screen.getByText("2 件")).toBeInTheDocument();
  });

  it("broken_refs_section_expands_on_click", async () => {
    const user = userEvent.setup();
    render(<UpgradeCheckPanel solutionId={1} />);
    await user.click(screen.getByRole("button", { name: /壊れた参照/ }));
    expect(screen.getByText("MainScript")).toBeInTheDocument();
    expect(screen.getByText("DeletedScript")).toBeInTheDocument();
  });
});
