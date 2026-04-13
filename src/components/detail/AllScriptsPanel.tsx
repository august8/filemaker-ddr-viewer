import { useState, useMemo } from "react";
import { useScriptList } from "../../hooks/useTauriCommand";
import { useAppStore } from "../../stores/appStore";
import { Spinner } from "../Spinner";

interface Props {
  projectId: number;
}

export function AllScriptsPanel({ projectId }: Props) {
  const { data: scripts = [], isLoading } = useScriptList(projectId);
  const { selectElement } = useAppStore();
  const [filter, setFilter] = useState("");

  const filtered = useMemo(() => {
    const q = filter.trim().toLowerCase();
    if (!q) return scripts;
    return scripts.filter((s) => s.name.toLowerCase().includes(q));
  }, [scripts, filter]);

  if (isLoading) {
    return <div className="flex items-center gap-2 p-4 text-gray-500 text-sm"><Spinner className="w-4 h-4" />読み込み中...</div>;
  }

  return (
    <div className="flex flex-col h-full">
      <div className="px-4 pt-4 pb-2 shrink-0">
        <h2 className="text-lg font-bold text-gray-800 mb-2">
          スクリプト一覧
          <span className="ml-2 text-sm font-normal text-gray-500">
            {filtered.length} / {scripts.length} 件
          </span>
        </h2>
        <input
          type="text"
          placeholder="スクリプト名で絞り込み..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          className="w-full px-3 py-1.5 text-sm border border-gray-300 rounded focus:outline-none focus:ring-1 focus:ring-blue-400"
        />
      </div>

      <div className="flex-1 overflow-auto px-4 pb-4">
        <table className="w-full text-sm border-collapse">
          <thead className="sticky top-0 bg-white z-10">
            <tr className="bg-gray-100 text-left">
              <th className="px-3 py-2 border border-gray-200">スクリプト名</th>
              <th className="px-3 py-2 border border-gray-200 text-right whitespace-nowrap">ステップ数</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((s) => (
              <tr
                key={s.id}
                className="hover:bg-blue-50 cursor-pointer"
                onClick={() => selectElement({ kind: "script", projectId, id: s.id, name: s.name })}
              >
                <td className="px-3 py-1.5 border border-gray-200 text-blue-600 hover:underline">
                  {s.name}
                </td>
                <td className="px-3 py-1.5 border border-gray-200 text-right text-gray-700">
                  {s.step_count}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {filtered.length === 0 && (
          <p className="text-gray-400 text-sm mt-4 text-center">該当するスクリプトなし</p>
        )}
      </div>
    </div>
  );
}
