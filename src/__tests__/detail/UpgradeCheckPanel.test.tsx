import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { UpgradeCheckPanel } from "../../components/detail/UpgradeCheckPanel";
import type { UpgradeHit } from "../../types/ddr";

vi.mock("../../hooks/useTauriCommand", () => ({
  useUpgradeCheck: vi.fn(),
}));

vi.mock("../../stores/appStore", () => ({
  useAppStore: vi.fn(() => ({
    checkItems: [
      { id: "item1", label: "Perform Script", enabled: true, detectionType: "step_type_id", detectionValue: "89" },
    ],
    selectElement: vi.fn(),
    setRightPanel: vi.fn(),
  })),
}));

// @tauri-apps/plugin-dialog と @tauri-apps/api/core はテスト環境で使えないためモック
vi.mock("@tauri-apps/plugin-dialog", () => ({ save: vi.fn().mockResolvedValue(null) }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

import { useUpgradeCheck } from "../../hooks/useTauriCommand";

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
    const { useAppStore } = await import("../../stores/appStore");
    vi.mocked(useAppStore).mockReturnValue({
      checkItems: [
        { id: "item1", label: "Perform Script", enabled: true, detectionType: "step_type_id", detectionValue: "89" },
      ],
      selectElement,
      setRightPanel: vi.fn(),
    } as unknown as ReturnType<typeof useAppStore>);
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
