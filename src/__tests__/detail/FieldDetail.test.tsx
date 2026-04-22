import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { FieldDetail } from "../../components/detail/FieldDetail";
import { makeFieldRow } from "../testFixtures";

vi.mock("../../hooks/fieldRefs", () => ({
  useFieldRefs: vi.fn(() => ({ data: [], isLoading: false })),
  useFieldCalcRefs: vi.fn(() => ({ data: [], isLoading: false })),
  useFieldLayoutRefs: vi.fn(() => ({ data: [], isLoading: false })),
  useFieldRelationshipKeys: vi.fn(() => ({ data: [], isLoading: false })),
  useLayoutRefDebugInfo: vi.fn(() => ({ data: undefined })),
}));
vi.mock("../../hooks/script", () => ({
  useScriptList: vi.fn(() => ({ data: [] })),
}));
vi.mock("../../hooks/layout", () => ({
  useLayoutList: vi.fn(() => ({ data: [] })),
}));

vi.mock("../../stores/appStore", () => ({
  useAppStore: vi.fn(() => ({
    selectElement: vi.fn(),
    setRightPanel: vi.fn(),
  })),
}));

const defaultProps = {
  field: makeFieldRow(),
  tableName: "TestTable",
  projectId: 1,
};

describe("FieldDetail", () => {
  it("renders_basic_properties_section", () => {
    render(<FieldDetail {...defaultProps} />);
    expect(screen.getByText("フィールドプロパティ")).toBeInTheDocument();
    expect(screen.getByText("TestField")).toBeInTheDocument();
  });

  it("renders_all_reference_sections", () => {
    render(<FieldDetail {...defaultProps} />);
    expect(screen.getByText("計算フィールドとして使用されている箇所")).toBeInTheDocument();
    expect(screen.getByText("スクリプト使用箇所")).toBeInTheDocument();
    expect(screen.getByText("レイアウト使用箇所")).toBeInTheDocument();
    expect(screen.getByText("リレーションキー使用箇所")).toBeInTheDocument();
  });

  it("does_not_render_auto_enter_when_empty", () => {
    render(<FieldDetail {...defaultProps} field={makeFieldRow({ auto_enter_type: "" })} />);
    expect(screen.queryByText("自動入力")).not.toBeInTheDocument();
  });

  it("renders_auto_enter_when_set", () => {
    render(
      <FieldDetail
        {...defaultProps}
        field={makeFieldRow({ auto_enter_type: "Serial" })}
      />
    );
    expect(screen.getByText("自動入力")).toBeInTheDocument();
    expect(screen.getByText("シリアル番号")).toBeInTheDocument();
  });

  it("does_not_render_validation_when_none", () => {
    render(<FieldDetail {...defaultProps} />);
    expect(screen.queryByText("入力値の制限")).not.toBeInTheDocument();
  });

  it("renders_validation_when_not_empty", () => {
    render(
      <FieldDetail
        {...defaultProps}
        field={makeFieldRow({ val_not_empty: true })}
      />
    );
    expect(screen.getByText("入力値の制限")).toBeInTheDocument();
    expect(screen.getByText("入力必須")).toBeInTheDocument();
  });

  it("does_not_render_storage_when_none", () => {
    render(<FieldDetail {...defaultProps} field={makeFieldRow({ index_type: "", container_storage: null })} />);
    expect(screen.queryByText("ストレージ")).not.toBeInTheDocument();
  });

  it("renders_storage_when_index_type_set", () => {
    render(
      <FieldDetail
        {...defaultProps}
        field={makeFieldRow({ index_type: "All" })}
      />
    );
    expect(screen.getByText("ストレージ")).toBeInTheDocument();
    expect(screen.getByText("全て")).toBeInTheDocument();
  });

  it("shows_empty_message_for_script_refs", () => {
    render(<FieldDetail {...defaultProps} />);
    expect(
      screen.getByText("このフィールドを参照するスクリプトはありません")
    ).toBeInTheDocument();
  });

  it("shows_loading_spinner_for_calc_refs", async () => {
    const { useFieldCalcRefs } = await import("../../hooks/fieldRefs");
    vi.mocked(useFieldCalcRefs).mockReturnValue(
      { data: [], isLoading: true } as unknown as ReturnType<typeof useFieldCalcRefs>
    );
    render(<FieldDetail {...defaultProps} />);
    expect(screen.getAllByText("読み込み中...").length).toBeGreaterThan(0);
  });
});
