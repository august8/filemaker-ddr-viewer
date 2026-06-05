import { useState, useMemo, useEffect, useRef } from "react";
import { useRelationshipList } from "../../hooks/table";
import type { RelationshipRow } from "../../types/ddr";
import { Spinner } from "../Spinner";
import { PAGE_SIZE } from "../../constants";
import { useColumnResize } from "../../hooks/useColumnResize";

interface Props {
  projectId: number;
  highlightId?: number;
}

function predicateLines(rel: RelationshipRow): string[] {
  return rel.predicates.map(
    (p) => `${rel.left_table}::${p.left_field} ${p.operator} ${rel.right_table}::${p.right_field}`
  );
}

export function AllRelationshipsPanel({ projectId, highlightId }: Props) {
  const [page, setPage] = useState(0);
  const [filter, setFilter] = useState("");
  const { data: relationships = [], isLoading } = useRelationshipList(projectId, PAGE_SIZE, page * PAGE_SIZE);
  const highlightRef = useRef<HTMLTableRowElement>(null);
  const { widths, setThRef, getResizeHandleProps } = useColumnResize(3);
  const isLastPage = relationships.length < PAGE_SIZE;

  useEffect(() => {
    if (highlightRef.current) {
      highlightRef.current.scrollIntoView({ block: "center" });
    }
  }, [highlightId, isLoading]);

  const handleFilterChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setFilter(e.target.value);
    setPage(0);
  };

  const filtered = useMemo(() => {
    const q = filter.trim().toLowerCase();
    if (!q) return relationships;
    return relationships.filter(
      (rel) =>
        rel.left_table.toLowerCase().includes(q) ||
        rel.right_table.toLowerCase().includes(q) ||
        rel.predicates.some(
          (p) =>
            p.left_field.toLowerCase().includes(q) ||
            p.right_field.toLowerCase().includes(q)
        )
    );
  }, [relationships, filter]);

  if (isLoading) {
    return <div className="flex items-center gap-2 p-4 text-gray-500 text-sm"><Spinner className="w-4 h-4" />読み込み中...</div>;
  }

  return (
    <div className="flex flex-col h-full">
      {/* ヘッダー */}
      <div className="px-4 pt-4 pb-2 shrink-0">
        <h2 className="text-lg font-bold text-gray-800 mb-2">
          リレーション一覧
          <span className="ml-2 text-sm font-normal text-gray-500">
            {filtered.length} 件
          </span>
        </h2>
        <div className="flex items-center gap-2">
          <input
            type="text"
            placeholder="テーブル名・フィールド名で絞り込み..."
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

      {/* テーブル */}
      <div className="flex-1 overflow-auto [scrollbar-gutter:stable] px-4 pb-4">
        <table className="w-full text-sm border-separate border-spacing-0 table-fixed [&_td]:overflow-hidden [&_th]:overflow-hidden">
          <colgroup>
            {widths.map((w, i) => <col key={i} style={w !== undefined ? { width: `${w}px` } : undefined} />)}
          </colgroup>
          <thead className="sticky top-0 bg-white z-10">
            <tr className="bg-gray-100 text-left">
              <th ref={setThRef(0)} className="px-3 py-2 border border-gray-200 whitespace-nowrap relative">左テーブル<div className="absolute inset-y-0 right-0 w-1 cursor-col-resize select-none hover:bg-blue-400" {...getResizeHandleProps(0)} /></th>
              <th ref={setThRef(1)} className="px-3 py-2 border border-gray-200 whitespace-nowrap relative">右テーブル<div className="absolute inset-y-0 right-0 w-1 cursor-col-resize select-none hover:bg-blue-400" {...getResizeHandleProps(1)} /></th>
              <th ref={setThRef(2)} className="px-3 py-2 border border-gray-200 relative">結合条件<div className="absolute inset-y-0 right-0 w-1 cursor-col-resize select-none hover:bg-blue-400" {...getResizeHandleProps(2)} /></th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((rel) => {
              const lines = predicateLines(rel);
              const isHighlighted = rel.id === highlightId;
              return (
                <tr
                  key={rel.id}
                  ref={isHighlighted ? highlightRef : null}
                  className={`cursor-pointer ${isHighlighted ? "bg-blue-100 hover:bg-blue-100" : "hover:bg-blue-50"}`}
                >
                  <td className="px-3 py-1.5 border border-gray-200 text-gray-700 whitespace-nowrap align-top">
                    {rel.left_table}
                  </td>
                  <td className="px-3 py-1.5 border border-gray-200 text-gray-700 whitespace-nowrap align-top">
                    {rel.right_table}
                  </td>
                  <td className="px-3 py-1.5 border border-gray-200 font-mono text-xs">
                    {lines.length === 0 ? (
                      <span className="text-gray-400">—</span>
                    ) : (
                      <div className="space-y-0.5">
                        {lines.map((line, i) => (
                          <div key={i} className="text-gray-700">{line}</div>
                        ))}
                      </div>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
        {filtered.length === 0 && (
          <p className="text-gray-400 text-sm mt-4 text-center">該当するリレーションなし</p>
        )}
      </div>
    </div>
  );
}
