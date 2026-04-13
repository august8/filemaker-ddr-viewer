// src/components/DiffView.tsx
import { useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAllProjects, useCompareSolutions } from "../hooks/useTauriCommand";
import { useAppStore } from "../stores/appStore";
import type { DiffItem, DiffKind } from "../types/ddr";
import { DIFF_BADGE_VARIANTS, CARD, SELECT_INPUT } from "../styles/tokens";
import { Spinner } from "./Spinner";

const KIND_COLOR: Record<DiffKind, string> = {
  Added: "text-green-700",
  Removed: "text-red-600",
  Modified: "text-yellow-700",
};

const KIND_LABEL: Record<DiffKind, string> = {
  Added: "追加",
  Removed: "削除",
  Modified: "変更",
};

function groupByType(items: DiffItem[]): Record<string, DiffItem[]> {
  const groups: Record<string, DiffItem[]> = {};
  for (const item of items) {
    if (!groups[item.element_type]) groups[item.element_type] = [];
    groups[item.element_type].push(item);
  }
  return groups;
}

const NAVIGABLE_TYPES = new Set([
  "script",
  "layout",
  "table",
  "value_list",
  "custom_function",
  "table_occurrence",
  "relationship",
]);

const TYPE_LABEL: Record<string, string> = {
  script: "スクリプト",
  layout: "レイアウト",
  table: "テーブル",
  value_list: "バリューリスト",
  custom_function: "カスタム関数",
  field: "フィールド",
  project: "プロジェクト",
  table_occurrence: "テーブルオカレンス",
  relationship: "リレーション",
};

export function DiffView() {
  const { selectElement, navigateFromDiff, diffState, setDiffState, selectedElement } = useAppStore();
  const { solA, solB, committedA, committedB, expandedTypes: expandedTypesArr } = diffState;
  const expandedTypes = useMemo(() => new Set(expandedTypesArr), [expandedTypesArr]);

  const { data: allProjects = [] } = useAllProjects({ enabled: selectedElement?.kind === "diff" });
  const { data: diffResult, isLoading } = useCompareSolutions(
    committedA ?? null,
    committedB ?? null
  );

  // solution_id の重複なしリスト
  const solutions = useMemo(() => {
    const seen = new Set<number>();
    return allProjects.filter((p) => {
      if (seen.has(p.solution_id)) return false;
      seen.add(p.solution_id);
      return true;
    });
  }, [allProjects]);

  function formatImportedAt(raw: string): string {
    const m = raw.match(/(\d{4})-(\d{2})-(\d{2})\s+(\d{2}:\d{2})/);
    if (m) return `${m[2]}/${m[3]} ${m[4]}`;
    return raw;
  }

  const groups = useMemo(
    () => (diffResult ? groupByType(diffResult.items) : {}),
    [diffResult]
  );

  function toggleType(type: string) {
    const newArr = expandedTypesArr.includes(type)
      ? expandedTypesArr.filter((t) => t !== type)
      : [...expandedTypesArr, type];
    setDiffState({ ...diffState, expandedTypes: newArr });
  }

  function handleCompare() {
    if (solA && solB) {
      setDiffState({ ...diffState, committedA: solA, committedB: solB, expandedTypes: [] });
    }
  }

  async function handleItemClick(item: DiffItem) {
    if (!item.project_id) return;
    if (!NAVIGABLE_TYPES.has(item.element_type)) return;

    const projectId = item.project_id;

    const navigate = (element: Parameters<typeof selectElement>[0]) => {
      if (item.compare_project_id) {
        navigateFromDiff(element, item.compare_project_id);
      } else {
        selectElement(element);
      }
    };

    // パネル遷移（resolve_element_by_name 不要）
    if (item.element_type === "table_occurrence") {
      navigate({ kind: "all_table_occurrences", projectId });
      return;
    }
    if (item.element_type === "relationship") {
      navigate({ kind: "all_relationships", projectId });
      return;
    }

    type ElementRef = { id: number; name: string };
    const ref = await invoke<ElementRef | null>("resolve_element_by_name", {
      projectId,
      elementType: item.element_type,
      name: item.name,
    }).catch(() => null);
    if (!ref) return;

    const el = { projectId, id: ref.id, name: ref.name };

    switch (item.element_type) {
      case "script":          navigate({ kind: "script",          ...el }); break;
      case "layout":          navigate({ kind: "layout",          ...el }); break;
      case "table":           navigate({ kind: "table",           ...el }); break;
      case "value_list":      navigate({ kind: "value_list",      ...el }); break;
      case "custom_function": navigate({ kind: "custom_function", ...el }); break;
    }
  }

  const canCompare = solA !== null && solB !== null && solA !== solB;

  return (
    <div className="flex-1 overflow-auto p-6">
      {/* ヘッダー */}
      <h2 className="text-lg font-bold text-gray-800 mb-4">差分比較</h2>

      {/* 選択カード */}
      <div className={`${CARD} mb-4`}>
        <div className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 items-center text-sm">
          <span className="font-semibold text-gray-600 text-right">Primary</span>
          <select
            className={SELECT_INPUT}
            value={solA ?? ""}
            onChange={(e) => {
              const val = e.target.value;
              setDiffState({ ...diffState, solA: val ? Number(val) : null, committedA: null, committedB: null, expandedTypes: [] });
            }}
          >
            <option value="">ソリューションを選択...</option>
            {solutions.map((p) => (
              <option key={p.solution_id} value={p.solution_id}>
                {p.solution_name}（{formatImportedAt(p.solution_imported_at)}）
              </option>
            ))}
          </select>

          <span className="font-semibold text-gray-600 text-right">Target</span>
          <select
            className={SELECT_INPUT}
            value={solB ?? ""}
            onChange={(e) => {
              const val = e.target.value;
              setDiffState({ ...diffState, solB: val ? Number(val) : null, committedA: null, committedB: null, expandedTypes: [] });
            }}
          >
            <option value="">比較先を選択...</option>
            {solutions.map((p) => (
              <option key={p.solution_id} value={p.solution_id}>
                {p.solution_name}（{formatImportedAt(p.solution_imported_at)}）
              </option>
            ))}
          </select>
        </div>

        <div className="mt-3">
          <button
            className="px-6 py-1.5 rounded border font-medium text-sm transition-colors
              disabled:opacity-40 disabled:cursor-not-allowed
              bg-blue-50 border-blue-300 text-blue-700 hover:bg-blue-100 disabled:hover:bg-blue-50"
            disabled={!canCompare}
            onClick={handleCompare}
          >
            比較する
          </button>
        </div>
      </div>

      {/* コンテンツ */}
      {!committedA && (
        <p className="text-sm text-gray-400 text-center py-8">
          Primary と Target を選択して「比較する」を押してください
        </p>
      )}

      {committedA && isLoading && (
        <div className="flex justify-center py-8"><Spinner className="w-5 h-5" /></div>
      )}

      {diffResult && !isLoading && (
        <div className={CARD}>
          {/* サマリーバッジ */}
          <div className="flex gap-3 text-sm mb-4">
            <span className="font-semibold text-green-700">追加 +{diffResult.added_count}</span>
            <span className="font-semibold text-red-600">削除 -{diffResult.removed_count}</span>
            <span className="font-semibold text-yellow-700">変更 ~{diffResult.modified_count}</span>
          </div>

          {diffResult.items.length === 0 ? (
            <p className="text-sm text-gray-400 text-center py-4">差分なし</p>
          ) : (
            <div className="space-y-2">
              {Object.entries(groups).map(([type, items]) => (
                <div key={type} className="border border-gray-200 rounded overflow-hidden">
                  <button
                    className="w-full flex items-center justify-between px-4 py-2 bg-gray-50 hover:bg-gray-100 text-sm font-semibold text-gray-700 transition-colors"
                    onClick={() => toggleType(type)}
                  >
                    <span>{TYPE_LABEL[type] ?? type}</span>
                    <div className="flex items-center gap-2">
                      <span className="text-xs font-normal text-gray-400">{items.length} 件</span>
                      <span className="text-xs text-gray-400">{expandedTypes.has(type) ? "▲" : "▼"}</span>
                    </div>
                  </button>
                  {expandedTypes.has(type) && (
                    <div className="divide-y divide-gray-100">
                      {items.map((item, i) => {
                        const isNavigable =
                          !!item.project_id && NAVIGABLE_TYPES.has(item.element_type);
                        return (
                          <div key={i} className="px-4 py-2 flex items-center gap-3 hover:bg-blue-50">
                            <span
                              className={`shrink-0 px-2 py-0.5 text-xs font-medium border rounded ${DIFF_BADGE_VARIANTS[item.kind]}`}
                            >
                              {KIND_LABEL[item.kind]}
                            </span>
                            <button
                              className={`flex-1 text-left text-sm ${KIND_COLOR[item.kind]} ${
                                isNavigable ? "hover:underline cursor-pointer" : "cursor-default"
                              }`}
                              onClick={() => handleItemClick(item)}
                              disabled={!isNavigable}
                            >
                              {item.name}
                            </button>
                            {item.detail && (
                              <span className="text-xs text-gray-400 shrink-0">{item.detail}</span>
                            )}
                          </div>
                        );
                      })}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
