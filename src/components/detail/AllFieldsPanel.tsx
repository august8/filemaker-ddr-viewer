// src/components/detail/AllFieldsPanel.tsx
import { useState, useMemo } from "react";
import { useAllFields } from "../../hooks/useTauriCommand";
import { useAppStore } from "../../stores/appStore";
import { Spinner } from "../Spinner";

interface Props {
  projectId: number;
}

export function AllFieldsPanel({ projectId }: Props) {
  const { data: fields = [], isLoading } = useAllFields(projectId);
  const { setRightPanel } = useAppStore();
  const [filter, setFilter] = useState("");

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

  if (isLoading) {
    return <div className="flex items-center gap-2 p-4 text-gray-500 text-sm"><Spinner className="w-4 h-4" />読み込み中...</div>;
  }

  return (
    <div className="flex flex-col h-full">
      {/* ヘッダー */}
      <div className="px-4 pt-4 pb-2 shrink-0">
        <h2 className="text-lg font-bold text-gray-800 mb-2">
          フィールド一覧
          <span className="ml-2 text-sm font-normal text-gray-500">
            {filtered.length} / {fields.length} 件
          </span>
        </h2>
        <input
          type="text"
          placeholder="テーブル名・フィールド名・型で絞り込み..."
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
              <th className="px-3 py-2 border border-gray-200 whitespace-nowrap">テーブル</th>
              <th className="px-3 py-2 border border-gray-200 whitespace-nowrap">フィールド名</th>
              <th className="px-3 py-2 border border-gray-200 whitespace-nowrap">型</th>
              <th className="px-3 py-2 border border-gray-200 whitespace-nowrap">種別</th>
              <th className="px-3 py-2 border border-gray-200 whitespace-nowrap">G</th>
              <th className="px-3 py-2 border border-gray-200">コメント</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((field) => (
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
                <td className="px-3 py-1.5 border border-gray-200 text-gray-500 max-w-xs truncate" title={field.comment ?? ""}>
                  {field.comment}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {filtered.length === 0 && (
          <p className="text-gray-400 text-sm mt-4 text-center">該当するフィールドなし</p>
        )}
      </div>
    </div>
  );
}
