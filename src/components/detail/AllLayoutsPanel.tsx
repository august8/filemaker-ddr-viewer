import { useState, useMemo } from "react";
import { useLayoutList } from "../../hooks/layout";
import { useAppStore } from "../../stores/appStore";
import { Spinner } from "../Spinner";
import { PAGE_SIZE } from "../../constants";


interface Props {
  projectId: number;
}

export function AllLayoutsPanel({ projectId }: Props) {
  const [page, setPage] = useState(0);
  const [filter, setFilter] = useState("");

  const { data: layouts = [], isLoading } = useLayoutList(projectId, PAGE_SIZE, page * PAGE_SIZE);
  const { selectElement } = useAppStore();

  const isLastPage = layouts.length < PAGE_SIZE;

  const filtered = useMemo(() => {
    const q = filter.trim().toLowerCase();
    if (!q) return layouts;
    return layouts.filter(
      (l) =>
        l.name.toLowerCase().includes(q) ||
        (l.table_occurrence_name ?? "").toLowerCase().includes(q)
    );
  }, [layouts, filter]);

  const handleFilterChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setFilter(e.target.value);
    setPage(0);
  };

  if (isLoading) {
    return <div className="flex items-center gap-2 p-4 text-gray-500 text-sm"><Spinner className="w-4 h-4" />読み込み中...</div>;
  }

  return (
    <div className="flex flex-col h-full">
      <div className="px-4 pt-4 pb-2 shrink-0">
        <h2 className="text-lg font-bold text-gray-800 mb-2">
          レイアウト一覧
          <span className="ml-2 text-sm font-normal text-gray-500">
            {filtered.length} 件
          </span>
        </h2>
        <div className="flex items-center gap-2">
          <input
            type="text"
            placeholder="レイアウト名・テーブルオカレンス名で絞り込み..."
            value={filter}
            onChange={handleFilterChange}
            className="flex-1 px-3 py-1.5 text-sm border border-gray-300 rounded focus:outline-none focus:ring-1 focus:ring-blue-400"
          />
          <button
            onClick={() => setPage((p) => p - 1)}
            disabled={page === 0}
            className="px-2 py-1.5 text-sm border border-gray-300 rounded hover:bg-gray-100 disabled:opacity-40 disabled:cursor-not-allowed"
          >
            &lt; 前
          </button>
          <span className="text-sm text-gray-600 whitespace-nowrap">
            {page + 1} ページ
          </span>
          <button
            onClick={() => setPage((p) => p + 1)}
            disabled={isLastPage}
            className="px-2 py-1.5 text-sm border border-gray-300 rounded hover:bg-gray-100 disabled:opacity-40 disabled:cursor-not-allowed"
          >
            次 &gt;
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-auto px-4 pb-4">
        <table className="w-full text-sm border-collapse">
          <thead className="sticky top-0 bg-white z-10">
            <tr className="bg-gray-100 text-left">
              <th className="px-3 py-2 border border-gray-200">レイアウト名</th>
              <th className="px-3 py-2 border border-gray-200 whitespace-nowrap">テーブルオカレンス</th>
              <th className="px-3 py-2 border border-gray-200 text-right whitespace-nowrap">トリガー数</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((l) => (
              <tr
                key={l.id}
                className="hover:bg-blue-50 cursor-pointer"
                onClick={() => selectElement({ kind: "layout", projectId, id: l.id, name: l.name })}
              >
                <td className="px-3 py-1.5 border border-gray-200 text-blue-600 hover:underline">
                  {l.name}
                </td>
                <td className="px-3 py-1.5 border border-gray-200 text-gray-700">
                  {l.table_occurrence_name ?? <span className="text-gray-400">—</span>}
                </td>
                <td className="px-3 py-1.5 border border-gray-200 text-right text-gray-700">
                  {l.trigger_count}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {filtered.length === 0 && (
          <p className="text-gray-400 text-sm mt-4 text-center">該当するレイアウトなし</p>
        )}
      </div>
    </div>
  );
}
