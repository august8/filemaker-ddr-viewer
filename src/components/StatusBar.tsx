import { useAppStore } from "../stores/appStore";
import { useProjectSummary } from "../hooks/useTauriCommand";

export function StatusBar() {
  const { selectedProject, searchQuery, searchDuration } = useAppStore();
  const { data: summary } = useProjectSummary(selectedProject?.id ?? null);

  // 何も表示しない場合
  if (!selectedProject && !searchQuery.trim()) {
    return <div></div>;
  }

  // 検索中 or 検索結果表示
  if (searchQuery.trim()) {
    return (
      <div className="shrink-0 h-6 bg-gray-100 border-t border-gray-200 flex items-center px-3 gap-3 text-xs text-gray-500">
        <span>
          検索: <span className="font-medium text-gray-700">{searchQuery}</span>
        </span>
        {searchDuration !== null && (
          <span>{searchDuration}ms</span>
        )}
      </div>
    );
  }

  // プロジェクト選択中（要素数表示）
  if (!summary) {
    return <div className="shrink-0 h-6 bg-gray-100 border-t border-gray-200"></div>;
  }

  return (
    <div className="shrink-0 h-6 bg-gray-100 border-t border-gray-200 flex items-center px-3 gap-4 text-xs text-gray-500">
      <span>テーブル: {summary.table_count}</span>
      <span>スクリプト: {summary.script_count}</span>
      <span>レイアウト: {summary.layout_count}</span>
      <span>フィールド: {summary.field_count}</span>
    </div>
  );
}
