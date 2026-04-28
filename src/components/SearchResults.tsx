import { useEffect, useRef, type ReactNode } from "react";
import { Spinner } from "./Spinner";
import { useSearch } from "../hooks/search";
import { useAppStore } from "../stores/appStore";
import { useSearchFiltering } from "../hooks/useSearchFiltering";
import type { SearchResult } from "../types/ddr";

function escapeRegex(str: string): string {
  return str.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function highlightText(text: string, query: string): ReactNode {
  const words = query.trim().split(/\s+/).filter(Boolean).map(escapeRegex);
  if (words.length === 0) return text;
  const pattern = new RegExp(`(${words.join("|")})`, "gi");
  const parts = text.split(pattern);
  if (parts.length <= 1) return text;
  const matchRe = new RegExp(`^(${words.join("|")})$`, "i");
  return (
    <>
      {parts.map((part, i) =>
        part && matchRe.test(part) ? (
          <mark key={i} className="bg-yellow-200 text-gray-900 rounded-sm">{part}</mark>
        ) : (
          part
        )
      )}
    </>
  );
}

const TYPE_LABELS: Record<string, string> = {
  script: "スクリプト",
  field: "フィールド",
  layout: "レイアウト",
  table: "テーブル",
  value_list: "バリューリスト",
  custom_function: "カスタム関数",
  table_occurrence: "テーブルオカレンス",
  relationship: "リレーション",
};

const TYPE_COLORS: Record<string, string> = {
  script: "bg-orange-100 text-orange-700",
  field: "bg-purple-100 text-purple-700",
  layout: "bg-green-100 text-green-700",
  table: "bg-blue-100 text-blue-700",
  value_list: "bg-yellow-100 text-yellow-700",
  custom_function: "bg-pink-100 text-pink-700",
  table_occurrence: "bg-teal-100 text-teal-700",
  relationship: "bg-indigo-100 text-indigo-700",
};

interface Props {
  query: string;
}

export function SearchResults({ query }: Props) {
  const { searchContains, searchScope, selectedProject, selectedSolution, selectElement, setRightPanel, setSearchDuration } = useAppStore();
  const { data: results, isLoading } = useSearch(
    selectedProject?.id ?? null,
    query,
    searchContains,
    searchScope,
    selectedSolution?.id ?? null,
  );
  const { activeType, setActiveType, countsByType, filteredResults } =
    useSearchFiltering(results ?? []);
  const searchStartRef = useRef<number | null>(null);

  useEffect(() => {
    if (isLoading) {
      searchStartRef.current = Date.now();
      setSearchDuration(null);
    } else if (searchStartRef.current !== null) {
      setSearchDuration(Date.now() - searchStartRef.current);
      searchStartRef.current = null;
    }
  }, [isLoading, setSearchDuration]);

  if (!query.trim()) return null;

  if (isLoading) {
    return <div className="flex items-center gap-2 text-gray-400 text-sm p-4"><Spinner className="w-4 h-4" />検索中...</div>;
  }

  if (!results || results.length === 0) {
    return (
      <div className="text-gray-400 text-sm p-4">見つかりませんでした</div>
    );
  }

  function handleClick(result: SearchResult) {
    const pid = result.project_id;

    switch (result.element_type) {
      case "table":
        selectElement({ kind: "table", projectId: pid, id: result.element_id, name: result.name });
        break;
      case "script":
        selectElement({ kind: "script", projectId: pid, id: result.element_id, name: result.name });
        break;
      case "layout":
        selectElement({ kind: "layout", projectId: pid, id: result.element_id, name: result.name });
        break;
      case "value_list":
        selectElement({ kind: "value_list", projectId: pid, id: result.element_id, name: result.name });
        break;
      case "custom_function":
        selectElement({ kind: "custom_function", projectId: pid, id: result.element_id, name: result.name });
        break;
      case "field":
        if (result.parent_id != null && result.parent_name != null) {
          selectElement({ kind: "table", projectId: pid, id: result.parent_id, name: result.parent_name });
          setRightPanel({
            kind: "field",
            projectId: pid,
            tableId: result.parent_id,
            fieldId: result.element_id,
            tableName: result.parent_name,
          });
        }
        break;
      case "table_occurrence":
        selectElement({ kind: "all_table_occurrences", projectId: pid });
        break;
      case "relationship":
        selectElement({ kind: "all_relationships", projectId: pid });
        break;
    }
  }

  return (
    <div className="space-y-3">
      {/* カテゴリフィルター */}
      <div className="flex flex-wrap gap-1.5">
        <button
          onClick={() => setActiveType(null)}
          className={`px-2.5 py-1 rounded-full text-xs font-medium transition-colors ${
            activeType === null
              ? "bg-gray-700 text-white"
              : "bg-gray-100 text-gray-600 hover:bg-gray-200"
          }`}
        >
          すべて ({results.length})
        </button>
        {Object.entries(countsByType).map(([type, count]) => (
          <button
            key={type}
            onClick={() => setActiveType(activeType === type ? null : type)}
            className={`px-2.5 py-1 rounded-full text-xs font-medium transition-colors ${
              activeType === type
                ? "bg-gray-700 text-white"
                : "bg-gray-100 text-gray-600 hover:bg-gray-200"
            }`}
          >
            {TYPE_LABELS[type] ?? type} ({count})
          </button>
        ))}
      </div>

      {/* 検索結果リスト */}
      <div className="bg-white rounded-lg border border-gray-200 divide-y divide-gray-100">
        {filteredResults.map((result) => (
          <button
            key={`${result.element_type}-${result.element_id}`}
            data-testid="search-result-item"
            className="w-full text-left px-4 py-3 hover:bg-blue-50 transition-colors"
            onClick={() => handleClick(result)}
          >
            <div className="flex items-center gap-2 mb-0.5">
              <span
                className={`text-xs rounded px-1.5 py-0.5 shrink-0 ${
                  TYPE_COLORS[result.element_type] ?? "bg-gray-100 text-gray-600"
                }`}
              >
                {TYPE_LABELS[result.element_type] ?? result.element_type}
              </span>
              <span className="text-sm font-medium text-gray-900 truncate" title={result.name}>
                {highlightText(result.name, query)}
              </span>
              {result.element_type === "field" && result.parent_name && (
                <span className="text-xs text-gray-400 shrink-0">
                  ({result.parent_name})
                </span>
              )}
            </div>
            {result.snippet && (
              <p className="text-xs text-gray-500 truncate pl-0.5">{highlightText(result.snippet, query)}</p>
            )}
          </button>
        ))}
      </div>
    </div>
  );
}
