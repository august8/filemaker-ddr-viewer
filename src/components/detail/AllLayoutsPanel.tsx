import { useState, useMemo } from "react";
import { useLayoutList } from "../../hooks/layout";
import { useAppStore } from "../../stores/appStore";
import { Spinner } from "../Spinner";
import { PAGE_SIZE } from "../../constants";
import { useColumnResize } from "../../hooks/useColumnResize";
import { FILTER_INPUT, PAGINATION_BUTTON } from "../../styles/tokens";


interface Props {
  projectId: number;
}

export function AllLayoutsPanel({ projectId }: Props) {
  const [page, setPage] = useState(0);
  const [filter, setFilter] = useState("");

  const { data: layouts = [], isLoading } = useLayoutList(projectId, PAGE_SIZE, page * PAGE_SIZE);
  const { selectElement } = useAppStore();
  const { widths, setThRef, getResizeHandleProps } = useColumnResize(3);

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
            className={FILTER_INPUT}
          />
          <button
            onClick={() => setPage((p) => p - 1)}
            disabled={page === 0}
            className={PAGINATION_BUTTON}
          >
            &lt; 前
          </button>
          <span className="text-sm text-gray-600 whitespace-nowrap">
            {page + 1} ページ
          </span>
          <button
            onClick={() => setPage((p) => p + 1)}
            disabled={isLastPage}
            className={PAGINATION_BUTTON}
          >
            次 &gt;
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-auto [scrollbar-gutter:stable] px-4 pb-4">
        <table className="w-full text-sm border-separate border-spacing-0 table-fixed [&_td]:overflow-hidden [&_th]:overflow-hidden">
          <colgroup>
            {widths.map((w, i) => <col key={i} style={w !== undefined ? { width: `${w}px` } : undefined} />)}
          </colgroup>
          <thead className="sticky top-0 bg-white z-10">
            <tr className="bg-gray-100 text-left">
              <th ref={setThRef(0)} className="px-3 py-2 border border-gray-200 relative">レイアウト名<div className="absolute inset-y-0 right-0 w-1 cursor-col-resize select-none hover:bg-blue-400" {...getResizeHandleProps(0)} /></th>
              <th ref={setThRef(1)} className="px-3 py-2 border border-gray-200 whitespace-nowrap relative">テーブルオカレンス<div className="absolute inset-y-0 right-0 w-1 cursor-col-resize select-none hover:bg-blue-400" {...getResizeHandleProps(1)} /></th>
              <th ref={setThRef(2)} className="px-3 py-2 border border-gray-200 text-right whitespace-nowrap relative">トリガー数<div className="absolute inset-y-0 right-0 w-1 cursor-col-resize select-none hover:bg-blue-400" {...getResizeHandleProps(2)} /></th>
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
