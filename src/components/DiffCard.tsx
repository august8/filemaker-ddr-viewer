import { useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAllProjects, useCompareSolutions } from "../hooks/useTauriCommand";
import { useAppStore } from "../stores/appStore";
import type { DiffItem, DiffKind } from "../types/ddr";
import { Spinner } from "./Spinner";

const KIND_COLOR: Record<DiffKind, string> = {
  Added: "text-green-700",
  Removed: "text-red-600",
  Modified: "text-yellow-700",
};

const KIND_PREFIX: Record<DiffKind, string> = {
  Added: "+",
  Removed: "-",
  Modified: "~",
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
]);

export function DiffCard() {
  const { selectElement, navigateFromDiff, diffState, setDiffState } = useAppStore();
  const { solA, solB, committedA, committedB, expandedTypes: expandedTypesArr } = diffState;
  const expandedTypes = useMemo(() => new Set(expandedTypesArr), [expandedTypesArr]);

  const { data: allProjects = [] } = useAllProjects();
  const { data: diffResult, isLoading } = useCompareSolutions(
    committedA ?? null,
    committedB ?? null
  );

  // solution_id の重複なしリスト（インポート日時付き）
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

    type ElementRef = { id: number; name: string };
    const ref = await invoke<ElementRef | null>("resolve_element_by_name", {
      projectId: item.project_id,
      elementType: item.element_type,
      name: item.name,
    }).catch(() => null);
    if (!ref) return;

    const projectId = item.project_id;
    const el = { projectId, id: ref.id, name: ref.name };

    const navigate = (element: Parameters<typeof selectElement>[0]) => {
      if (item.compare_project_id) {
        navigateFromDiff(element, item.compare_project_id);
      } else {
        selectElement(element);
      }
    };

    switch (item.element_type) {
      case "script":   navigate({ kind: "script",          ...el }); break;
      case "layout":   navigate({ kind: "layout",          ...el }); break;
      case "table":    navigate({ kind: "table",           ...el }); break;
      case "value_list":     navigate({ kind: "value_list",     ...el }); break;
      case "custom_function": navigate({ kind: "custom_function", ...el }); break;
    }
  }

  const canCompare =
    solA !== null &&
    solB !== null &&
    solA !== solB;

  return (
    <div className="bg-white rounded-lg border shadow-sm p-4">
      {/* ヘッダー */}
      <h2 className="text-sm font-semibold text-gray-700 mb-3">変更差分</h2>

      {/* Primary / Target 選択エリア */}
      <div className="space-y-2 mb-3">
        {/* Primary（ドロップダウン） */}
        <div className="flex items-center gap-2 text-xs">
          <span className="w-14 shrink-0 text-gray-500 font-medium">Primary</span>
          <select
            className="flex-1 border rounded px-2 py-1 text-gray-700 bg-white focus:outline-none focus:ring-1 focus:ring-blue-400 text-xs"
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
        </div>

        {/* Target（ドロップダウン） */}
        <div className="flex items-center gap-2 text-xs">
          <span className="w-14 shrink-0 text-gray-500 font-medium">Target</span>
          <select
            className="flex-1 border rounded px-2 py-1 text-gray-700 bg-white focus:outline-none focus:ring-1 focus:ring-blue-400 text-xs"
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
      </div>

      {/* 比較ボタン */}
      <button
        className="w-full text-xs py-1.5 rounded border font-medium transition-colors
          disabled:opacity-40 disabled:cursor-not-allowed
          enabled:bg-blue-50 enabled:border-blue-300 enabled:text-blue-700 enabled:hover:bg-blue-100"
        disabled={!canCompare}
        onClick={handleCompare}
      >
        比較する
      </button>

      {/* コンテンツ */}
      {!committedA && (
        <p className="text-xs text-gray-400 text-center py-4">
          Primary と Target を選択して「比較する」を押してください
        </p>
      )}

      {committedA && isLoading && (
        <div className="flex justify-center py-4"><Spinner className="w-4 h-4" /></div>
      )}

      {diffResult && !isLoading && (
        <>
          {/* サマリーバッジ */}
          <div className="flex gap-3 text-xs mt-3 mb-2">
            <span className="font-medium text-green-700">追加 +{diffResult.added_count}</span>
            <span className="font-medium text-red-600">削除 -{diffResult.removed_count}</span>
            <span className="font-medium text-yellow-700">変更 ~{diffResult.modified_count}</span>
          </div>

          {diffResult.items.length === 0 ? (
            <p className="text-xs text-gray-400 text-center py-2">差分なし</p>
          ) : (
            <div className="space-y-1">
              {Object.entries(groups).map(([type, items]) => (
                <div key={type}>
                  <button
                    className="w-full flex items-center gap-1 text-xs font-medium text-gray-600 hover:text-gray-900 py-0.5"
                    onClick={() => toggleType(type)}
                  >
                    <span>{expandedTypes.has(type) ? "▼" : "▶"}</span>
                    <span>{type}</span>
                    <span className="text-gray-400">({items.length})</span>
                  </button>
                  {expandedTypes.has(type) && (
                    <ul className="ml-4 space-y-0.5">
                      {items.map((item, i) => {
                        const isNavigable =
                          !!item.project_id && NAVIGABLE_TYPES.has(item.element_type);
                        return (
                          <li key={i}>
                            <button
                              className={`text-xs flex gap-1 w-full text-left ${KIND_COLOR[item.kind]} ${
                                isNavigable
                                  ? "hover:underline cursor-pointer"
                                  : "cursor-default"
                              }`}
                              onClick={() => handleItemClick(item)}
                              disabled={!isNavigable}
                            >
                              <span className="font-mono w-3 shrink-0">
                                {KIND_PREFIX[item.kind]}
                              </span>
                              <span className="truncate">{item.name}</span>
                              {item.detail && (
                                <span className="text-gray-400 truncate ml-1">
                                  — {item.detail}
                                </span>
                              )}
                            </button>
                          </li>
                        );
                      })}
                    </ul>
                  )}
                </div>
              ))}
            </div>
          )}
        </>
      )}
    </div>
  );
}
