import { useState, useMemo, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useAllFields } from "../../hooks/table";
import { useAppStore } from "../../stores/appStore";
import { Spinner } from "../Spinner";
import { PAGE_SIZE } from "../../constants";
import { useColumnResize } from "../../hooks/useColumnResize";
import { FILTER_INPUT, PAGINATION_BUTTON } from "../../styles/tokens";


interface Props {
  projectId: number;
}

export function AllFieldsPanel({ projectId }: Props) {
  const [page, setPage] = useState(0);
  const [filter, setFilter] = useState("");

  const { data: fields = [], isLoading } = useAllFields(
    projectId,
    PAGE_SIZE,
    page * PAGE_SIZE
  );
  const { setRightPanel } = useAppStore();

  const isLastPage = fields.length < PAGE_SIZE;

  const filtered = useMemo(() => {
    const q = filter.trim().toLowerCase();
    if (!q) return fields;
    return fields.filter(
      (f) =>
        f.name.toLowerCase().includes(q) ||
        f.table_name.toLowerCase().includes(q) ||
        f.data_type.toLowerCase().includes(q) ||
        f.comment.toLowerCase().includes(q)
    );
  }, [fields, filter]);

  const scrollRef = useRef<HTMLDivElement>(null);
  const { widths, setThRef, getResizeHandleProps } = useColumnResize(6);

  const virtualizer = useVirtualizer({
    count: filtered.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => 40,
    overscan: 10,
  });

  const virtualRows = virtualizer.getVirtualItems();
  const totalSize = virtualizer.getTotalSize();
  const paddingTop = virtualRows[0]?.start ?? 0;
  const lastVirtualRow = virtualRows[virtualRows.length - 1];
  const paddingBottom = totalSize - (lastVirtualRow?.end ?? 0);

  const handleFilterChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setFilter(e.target.value);
    setPage(0);
  };

  if (isLoading) {
    return (
      <div className="flex items-center gap-2 p-4 text-gray-500 text-sm">
        <Spinner className="w-4 h-4" />
        読み込み中...
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* ヘッダー */}
      <div className="px-4 pt-4 pb-2 shrink-0">
        <h2 className="text-lg font-bold text-gray-800 mb-2">
          フィールド一覧
          <span className="ml-2 text-sm font-normal text-gray-500">
            {filtered.length} 件
          </span>
        </h2>
        <div className="flex items-center gap-2">
          <input
            type="text"
            placeholder="テーブル名・フィールド名・型で絞り込み..."
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

      {/* テーブル */}
      <div ref={scrollRef} className="flex-1 overflow-auto [scrollbar-gutter:stable] px-4 pb-4">
        <table className="w-full text-sm border-separate border-spacing-0 table-fixed [&_td]:overflow-hidden [&_th]:overflow-hidden">
          <colgroup>
            {widths.map((w, i) => <col key={i} style={w !== undefined ? { width: `${w}px` } : undefined} />)}
          </colgroup>
          <thead className="sticky top-0 bg-white z-10">
            <tr className="bg-gray-100 text-left">
              <th ref={setThRef(0)} className="px-3 py-2 border border-gray-200 whitespace-nowrap relative">テーブル<div className="absolute inset-y-0 right-0 w-1 cursor-col-resize select-none hover:bg-blue-400" {...getResizeHandleProps(0)} /></th>
              <th ref={setThRef(1)} className="px-3 py-2 border border-gray-200 whitespace-nowrap relative">フィールド名<div className="absolute inset-y-0 right-0 w-1 cursor-col-resize select-none hover:bg-blue-400" {...getResizeHandleProps(1)} /></th>
              <th ref={setThRef(2)} className="px-3 py-2 border border-gray-200 whitespace-nowrap relative">型<div className="absolute inset-y-0 right-0 w-1 cursor-col-resize select-none hover:bg-blue-400" {...getResizeHandleProps(2)} /></th>
              <th ref={setThRef(3)} className="px-3 py-2 border border-gray-200 whitespace-nowrap relative">種別<div className="absolute inset-y-0 right-0 w-1 cursor-col-resize select-none hover:bg-blue-400" {...getResizeHandleProps(3)} /></th>
              <th ref={setThRef(4)} className="px-3 py-2 border border-gray-200 whitespace-nowrap relative">G<div className="absolute inset-y-0 right-0 w-1 cursor-col-resize select-none hover:bg-blue-400" {...getResizeHandleProps(4)} /></th>
              <th ref={setThRef(5)} className="px-3 py-2 border border-gray-200 relative">コメント<div className="absolute inset-y-0 right-0 w-1 cursor-col-resize select-none hover:bg-blue-400" {...getResizeHandleProps(5)} /></th>
            </tr>
          </thead>
          <tbody>
            {/* top spacer */}
            <tr><td colSpan={6} style={{ height: `${paddingTop}px`, padding: 0 }} /></tr>
            {virtualRows.map((vRow) => {
              const field = filtered[vRow.index];
              return (
                <tr
                  key={field.id}
                  className="hover:bg-blue-50 cursor-pointer"
                  onClick={() =>
                    setRightPanel({
                      kind: "field",
                      projectId,
                      tableId: field.table_id,
                      fieldId: field.id,
                      tableName: field.table_name,
                    })
                  }
                >
                  <td className="px-3 py-1.5 border border-gray-200 text-gray-500 whitespace-nowrap">
                    {field.table_name}
                  </td>
                  <td className="px-3 py-1.5 border border-gray-200 font-medium text-gray-800 whitespace-nowrap">
                    {field.name}
                  </td>
                  <td className="px-3 py-1.5 border border-gray-200 text-xs font-mono whitespace-nowrap">
                    {field.data_type}
                  </td>
                  <td className="px-3 py-1.5 border border-gray-200 text-xs whitespace-nowrap">
                    {field.field_type === "normal"
                      ? ""
                      : field.field_type === "calculated"
                      ? "計算"
                      : field.field_type === "summary"
                      ? "集計"
                      : field.field_type}
                  </td>
                  <td className="px-3 py-1.5 border border-gray-200 text-center text-xs">
                    {field.is_global ? "G" : ""}
                  </td>
                  <td
                    className="px-3 py-1.5 border border-gray-200 text-gray-500 max-w-xs truncate"
                    title={field.comment ?? ""}
                  >
                    {field.comment}
                  </td>
                </tr>
              );
            })}
            {/* bottom spacer */}
            <tr><td colSpan={6} style={{ height: `${paddingBottom}px`, padding: 0 }} /></tr>
          </tbody>
        </table>
        {filtered.length === 0 && (
          <p className="text-gray-400 text-sm mt-4 text-center">該当するフィールドなし</p>
        )}
      </div>
    </div>
  );
}
