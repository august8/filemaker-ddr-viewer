// src/__tests__/CategoryTree.test.tsx
// Task E: CategoryTree スクロール安定化 (ADR-015)
// flushSync 後に直接 scrollIntoView を呼ぶことで requestAnimationFrame の競合を解消

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, act, screen } from "@testing-library/react";
import { CategoryTree } from "../components/navigation/CategoryTree";
import type { SelectedElement } from "../stores/appStore";

vi.mock("../hooks/table", () => ({
  useTableList: vi.fn(() => ({
    data: [{ id: 1, name: "TestTable", field_count: 3 }],
  })),
  useTableOccurrenceList: vi.fn(() => ({ data: [] })),
  useRelationshipList: vi.fn(() => ({ data: [] })),
}));
vi.mock("../hooks/script", () => ({
  useScriptList: vi.fn(() => ({
    data: [
      { id: 10, name: "ScriptA", folder_level: 0, is_folder: false, is_separator: false },
      { id: 11, name: "ScriptB", folder_level: 0, is_folder: false, is_separator: false },
    ],
  })),
}));
vi.mock("../hooks/layout", () => ({
  useLayoutList: vi.fn(() => ({ data: [] })),
}));
vi.mock("../hooks/catalog", () => ({
  useValueListList: vi.fn(() => ({ data: [] })),
  useCustomFunctionList: vi.fn(() => ({ data: [] })),
}));

vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

import { useAppStore } from "../stores/appStore";

const mockSelectElement = vi.fn();
const mockSetRightPanel = vi.fn();

function setupStore(selectedElement: SelectedElement | null) {
  vi.mocked(useAppStore).mockReturnValue({
    selectedElement,
    selectElement: mockSelectElement,
    setRightPanel: mockSetRightPanel,
  } as unknown as ReturnType<typeof useAppStore>);
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("CategoryTree スクロール", () => {
  it("selectedElement が null のとき scrollIntoView を呼ばない", async () => {
    setupStore(null);
    await act(async () => {
      render(<CategoryTree projectId={1} />);
    });
    expect(Element.prototype.scrollIntoView).not.toHaveBeenCalled();
  });

  it("selectedElement の kind が categorized でないとき scrollIntoView を呼ばない", async () => {
    // all_fields は kindToCategory に含まれていないため ref が設定されない
    setupStore({ kind: "all_fields", projectId: 1 });
    await act(async () => {
      render(<CategoryTree projectId={1} />);
    });
    expect(Element.prototype.scrollIntoView).not.toHaveBeenCalled();
  });

  it("requestAnimationFrame は使用していない（flushSync 後に直接スクロールする）", async () => {
    // 旧実装（requestAnimationFrame）との最重要な差異を検証
    const rafSpy = vi.spyOn(window, "requestAnimationFrame");

    setupStore({ kind: "script", projectId: 1, id: 10, name: "ScriptA" });
    await act(async () => {
      render(<CategoryTree projectId={1} />);
    });

    expect(rafSpy).not.toHaveBeenCalled();
    rafSpy.mockRestore();
  });

  it("selectedElement が script のとき scripts カテゴリが自動展開される", async () => {
    // flushSync によるカテゴリ展開の動作確認
    // 初期状態：スクリプトカテゴリ閉（items は DOM に存在しない）
    setupStore(null);
    const { rerender } = render(<CategoryTree projectId={1} />);
    expect(screen.queryByText("ScriptA")).not.toBeInTheDocument();

    // selectedElement を script に変更
    setupStore({ kind: "script", projectId: 1, id: 10, name: "ScriptA" });
    await act(async () => {
      rerender(<CategoryTree projectId={1} />);
    });

    // flushSync でカテゴリが展開されスクリプト一覧が表示される
    expect(screen.getByText("ScriptA")).toBeInTheDocument();
    expect(screen.getByText("ScriptB")).toBeInTheDocument();
  });

  it("selectedElement が table のとき tables カテゴリが自動展開される", async () => {
    setupStore(null);
    const { rerender } = render(<CategoryTree projectId={1} />);

    setupStore({ kind: "table", projectId: 1, id: 1, name: "TestTable" });
    await act(async () => {
      rerender(<CategoryTree projectId={1} />);
    });

    expect(screen.getByText("TestTable")).toBeInTheDocument();
  });
});
