import { useState, useMemo } from "react";
import { BADGE_VARIANTS } from "../../styles/tokens";
import { Spinner } from "../Spinner";
import { useTableOccurrenceList, useTableList } from "../../hooks/table";
import { useLayoutList } from "../../hooks/layout";
import { useAppStore } from "../../stores/appStore";
import type { LayoutRow } from "../../types/ddr";
import { PAGE_SIZE } from "../../constants";


interface Props {
  projectId: number;
}

export function AllTableOccurrencesPanel({ projectId }: Props) {
  const [page, setPage] = useState(0);
  const [filter, setFilter] = useState("");

  const { data: tableOccurrences = [], isLoading } = useTableOccurrenceList(projectId, PAGE_SIZE, page * PAGE_SIZE);
  const { data: tables = [] } = useTableList(projectId);
  const { data: layouts = [] } = useLayoutList(projectId);
  const { selectElement, setRightPanel } = useAppStore();

  const isLastPage = tableOccurrences.length < PAGE_SIZE;

  const tableMap = useMemo(
    () => new Map(tables.map((t) => [t.name, t])),
    [tables]
  );

  const layoutsByTO = useMemo(() => {
    const map = new Map<string, LayoutRow[]>();
    for (const l of layouts) {
      if (l.table_occurrence_name) {
        const arr = map.get(l.table_occurrence_name) ?? [];
        arr.push(l);
        map.set(l.table_occurrence_name, arr);
      }
    }
    return map;
  }, [layouts]);

  const filtered = useMemo(() => {
    const q = filter.trim().toLowerCase();
    if (!q) return tableOccurrences;
    return tableOccurrences.filter(
      (to) =>
        to.occurrence_name.toLowerCase().includes(q) ||
        to.base_table_name.toLowerCase().includes(q) ||
        to.source_file.toLowerCase().includes(q)
    );
  }, [tableOccurrences, filter]);

  const handleFilterChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setFilter(e.target.value);
    setPage(0);
  };

  if (isLoading) {
    return <div className="flex items-center gap-2 p-4 text-gray-500 text-sm"><Spinner className="w-4 h-4" />読み込み中...</div>;
  }

  return (
    <div className="flex flex-col h-full">
      {/* ヘッダー */}
      <div className="px-4 pt-4 pb-2 shrink-0">
        <h2 className="text-lg font-bold text-gray-800 mb-2">
          テーブルオカレンス一覧
          <span className="ml-2 text-sm font-normal text-gray-500">
            {filtered.length} 件
          </span>
        </h2>
        <div className="flex items-center gap-2">
          <input
            type="text"
            placeholder="オカレンス名・ベーステーブル名で絞り込み..."
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
      <div className="flex-1 overflow-auto px-4 pb-4">
        <table className="w-full text-sm border-collapse">
          <thead className="sticky top-0 bg-white z-10">
            <tr className="bg-gray-100 text-left">
              <th className="px-3 py-2 border border-gray-200 whitespace-nowrap">オカレンス名</th>
              <th className="px-3 py-2 border border-gray-200 whitespace-nowrap">ベーステーブル</th>
              <th className="px-3 py-2 border border-gray-200 whitespace-nowrap">外部ファイル</th>
              <th className="px-3 py-2 border border-gray-200">使用レイアウト</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((to) => {
              const table = tableMap.get(to.base_table_name);
              const toLayouts = layoutsByTO.get(to.occurrence_name) ?? [];
              return (
                <tr key={to.id} className="hover:bg-blue-50 cursor-pointer">
                  <td className="px-3 py-1.5 border border-gray-200 font-medium text-gray-800 whitespace-nowrap">
                    {to.occurrence_name}
                  </td>
                  <td className="px-3 py-1.5 border border-gray-200 whitespace-nowrap">
                    {table ? (
                      <button
                        className="text-blue-600 hover:text-blue-800 hover:underline text-left"
                        onClick={() => {
                          setRightPanel(null);
                          selectElement({
                            kind: "table",
                            projectId,
                            id: table.id,
                            name: table.name,
                          });
                        }}
                      >
                        {to.base_table_name}
                      </button>
                    ) : (
                      <span className="text-gray-500">{to.base_table_name}</span>
                    )}
                  </td>
                  <td className="px-3 py-1.5 border border-gray-200 whitespace-nowrap text-xs text-gray-500">
                    {to.source_file || "—"}
                  </td>
                  <td className="px-3 py-1.5 border border-gray-200">
                    {toLayouts.length === 0 ? (
                      <span className="text-gray-400">—</span>
                    ) : (
                      <div className="flex flex-wrap gap-1">
                        {toLayouts.map((l) => (
                          <button
                            key={l.id}
                            className={`${BADGE_VARIANTS.green} hover:bg-green-200 whitespace-nowrap cursor-pointer`}
                            onClick={() => {
                              setRightPanel(null);
                              selectElement({
                                kind: "layout",
                                projectId,
                                id: l.id,
                                name: l.name,
                              });
                            }}
                          >
                            {l.name}
                          </button>
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
          <p className="text-gray-400 text-sm mt-4 text-center">該当するテーブルオカレンスなし</p>
        )}
      </div>
    </div>
  );
}
