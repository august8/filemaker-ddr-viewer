import { useCallback, useEffect, useState } from "react";
import { useAppStore } from "../stores/appStore";

export function SearchBar() {
  const {
    searchQuery,
    setSearchQuery,
    searchContains,
    setSearchContains,
    searchScope,
    setSearchScope,
    selectedProject,
    selectedSolution,
  } = useAppStore();
  const [localValue, setLocalValue] = useState(searchQuery);

  // searchQuery が外部から変更されたとき（selectElement によるクリア等）に localValue を同期する。
  // これにより、詳細画面への遷移後に debounce が旧クエリを復元するのを防ぐ。
  useEffect(() => {
    setLocalValue(searchQuery);
  }, [searchQuery]);

  // debounce: 300ms
  useEffect(() => {
    const timer = setTimeout(() => {
      setSearchQuery(localValue);
    }, 300);
    return () => clearTimeout(timer);
  }, [localValue, setSearchQuery]);

  const handleClear = useCallback(() => {
    setLocalValue("");
    setSearchQuery("");
  }, [setSearchQuery]);

  return (
    <div className="flex flex-col gap-1">
      <div className="relative flex items-center">
        <input
          type="text"
          value={localValue}
          onChange={(e) => setLocalValue(e.target.value)}
          placeholder="スクリプト、フィールド、レイアウトを検索..."
          className="w-full pl-3 pr-8 py-1.5 border border-gray-300 rounded text-sm focus:outline-none focus:border-blue-400"
        />
        {localValue && (
          <button
            onClick={handleClear}
            className="absolute right-2 text-gray-400 hover:text-gray-600"
            aria-label="クリア"
          >
            ×
          </button>
        )}
      </div>
      <div className="flex items-center gap-3 text-xs text-gray-500 px-0.5">
        {/* マッチモードトグル */}
        <div className="flex items-center gap-0.5">
          <button
            onClick={() => setSearchContains(false)}
            className={`px-1.5 py-0.5 rounded transition-colors ${
              !searchContains
                ? "bg-blue-100 text-blue-700 font-medium"
                : "hover:bg-gray-100 text-gray-500"
            }`}
          >
            前方一致
          </button>
          <button
            onClick={() => setSearchContains(true)}
            className={`px-1.5 py-0.5 rounded transition-colors ${
              searchContains
                ? "bg-blue-100 text-blue-700 font-medium"
                : "hover:bg-gray-100 text-gray-500"
            }`}
          >
            部分一致
          </button>
        </div>
        <span className="text-gray-300">|</span>
        {/* スコープトグル */}
        <div className="flex items-center gap-0.5">
          <button
            onClick={() => setSearchScope("all")}
            className={`px-1.5 py-0.5 rounded transition-colors ${
              searchScope === "all"
                ? "bg-blue-100 text-blue-700 font-medium"
                : "hover:bg-gray-100 text-gray-500"
            }`}
          >
            全体
          </button>
          <button
            onClick={() => selectedSolution !== null && setSearchScope("solution")}
            disabled={selectedSolution === null}
            className={`px-1.5 py-0.5 rounded transition-colors ${
              searchScope === "solution"
                ? "bg-blue-100 text-blue-700 font-medium"
                : selectedSolution === null
                ? "text-gray-300 cursor-not-allowed"
                : "hover:bg-gray-100 text-gray-500"
            }`}
          >
            ソリューション
          </button>
          <button
            onClick={() => selectedProject !== null && setSearchScope("project")}
            disabled={selectedProject === null}
            className={`px-1.5 py-0.5 rounded transition-colors ${
              searchScope === "project"
                ? "bg-blue-100 text-blue-700 font-medium"
                : selectedProject === null
                ? "text-gray-300 cursor-not-allowed"
                : "hover:bg-gray-100 text-gray-500"
            }`}
          >
            プロジェクト
          </button>
        </div>
      </div>
    </div>
  );
}
