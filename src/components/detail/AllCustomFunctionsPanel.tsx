import { useState, useMemo } from "react";
import { useCustomFunctionList } from "../../hooks/catalog";
import { useAppStore } from "../../stores/appStore";
import { Spinner } from "../Spinner";

const PAGE_SIZE = 500;

interface Props {
  projectId: number;
}

export function AllCustomFunctionsPanel({ projectId }: Props) {
  const [page, setPage] = useState(0);
  const [filter, setFilter] = useState("");

  const { data: customFunctions = [], isLoading } = useCustomFunctionList(projectId, PAGE_SIZE, page * PAGE_SIZE);
  const { selectElement } = useAppStore();

  const isLastPage = customFunctions.length < PAGE_SIZE;

  const filtered = useMemo(() => {
    const q = filter.trim().toLowerCase();
    if (!q) return customFunctions;
    return customFunctions.filter(
      (cf) =>
        cf.name.toLowerCase().includes(q) ||
        cf.parameters.toLowerCase().includes(q)
    );
  }, [customFunctions, filter]);

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
          カスタム関数一覧
          <span className="ml-2 text-sm font-normal text-gray-500">
            {filtered.length} 件
          </span>
        </h2>
        <div className="flex items-center gap-2">
          <input
            type="text"
            placeholder="カスタム関数名で絞り込み..."
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
              <th className="px-3 py-2 border border-gray-200">カスタム関数名</th>
              <th className="px-3 py-2 border border-gray-200">パラメータ</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((cf) => (
              <tr
                key={cf.id}
                className="hover:bg-blue-50 cursor-pointer"
                onClick={() => selectElement({ kind: "custom_function", projectId, id: cf.id, name: cf.name })}
              >
                <td className="px-3 py-1.5 border border-gray-200 text-blue-600 hover:underline">
                  {cf.name}
                </td>
                <td className="px-3 py-1.5 border border-gray-200 text-gray-500 font-mono text-xs">
                  {cf.parameters || <span className="text-gray-300">—</span>}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {filtered.length === 0 && (
          <p className="text-gray-400 text-sm mt-4 text-center">該当するカスタム関数なし</p>
        )}
      </div>
    </div>
  );
}
