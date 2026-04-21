import { useState, useMemo, useEffect, useRef } from "react";
import { useRelationshipList } from "../../hooks/table";
import type { RelationshipRow } from "../../types/ddr";
import { Spinner } from "../Spinner";

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
  const { data: relationships = [], isLoading } = useRelationshipList(projectId);
  const [filter, setFilter] = useState("");
  const highlightRef = useRef<HTMLTableRowElement>(null);

  useEffect(() => {
    if (highlightRef.current) {
      highlightRef.current.scrollIntoView({ block: "center" });
    }
  }, [highlightId, isLoading]);

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
            {filtered.length} / {relationships.length} 件
          </span>
        </h2>
        <input
          type="text"
          placeholder="テーブル名・フィールド名で絞り込み..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          className="w-full px-3 py-1.5 text-sm border border-gray-300 rounded focus:outline-none focus:ring-1 focus:ring-blue-400"
        />
      </div>

      {/* テーブル */}
      <div className="flex-1 overflow-auto px-4 pb-4">
        <table className="w-full text-sm border-collapse">
          <thead className="sticky top-0 bg-white z-10">
            <tr className="bg-gray-100 text-left">
              <th className="px-3 py-2 border border-gray-200 whitespace-nowrap">左テーブル</th>
              <th className="px-3 py-2 border border-gray-200 whitespace-nowrap">右テーブル</th>
              <th className="px-3 py-2 border border-gray-200">結合条件</th>
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
