import { useState } from "react";
import { useUnusedFields } from "../hooks/analysis";
import { useTableList } from "../hooks/table";
import { useAppStore } from "../stores/appStore";
import { BADGE_VARIANTS, CARD } from "../styles/tokens";
import { Spinner } from "./Spinner";

interface Props {
  projectId: number | null;
}

export function UnusedFieldsList({ projectId }: Props) {
  const { data: unused, isLoading } = useUnusedFields(projectId);
  const { data: tables = [] } = useTableList(projectId);
  const { selectElement, setRightPanel } = useAppStore();
  const [expanded, setExpanded] = useState(false);

  if (projectId === null) return null;
  if (isLoading) return <div className="flex items-center gap-2 text-gray-400 text-sm"><Spinner className="w-4 h-4" />読み込み中...</div>;
  if (!unused) return null;

  if (unused.length === 0) {
    return (
      <div className={CARD}>
        <h3 className="font-semibold text-gray-800 mb-2">未参照フィールド</h3>
        <div className="text-sm text-green-600">未参照フィールドはありません</div>
      </div>
    );
  }

  // テーブルごとにグループ化
  const grouped = unused.reduce<Record<string, typeof unused>>((acc, f) => {
    (acc[f.table_name] ??= []).push(f);
    return acc;
  }, {});

  const PREVIEW_COUNT = 5;
  const allTableNames = Object.keys(grouped).sort();
  const shownTableNames = expanded ? allTableNames : allTableNames.slice(0, PREVIEW_COUNT);

  function handleTableClick(tableName: string) {
    if (!projectId) return;
    const table = tables.find((t) => t.name === tableName);
    if (table) selectElement({ kind: "table", projectId, id: table.id, name: table.name });
  }

  return (
    <div className={CARD}>
      <div className="flex items-center justify-between mb-3">
        <h3 className="font-semibold text-gray-800">
          未参照フィールド
          <span className={`ml-2 ${BADGE_VARIANTS.yellow}`}>{unused.length}</span>
        </h3>
        <span className="text-xs text-gray-400">
          ※ OccName::FieldName 形式以外のベア参照は対象外
        </span>
      </div>
      <div className="space-y-2">
        {shownTableNames.map((tableName) => {
          const fields = grouped[tableName];
          return (
            <div key={tableName} className="border border-gray-100 rounded">
              <button
                className="w-full flex items-center justify-between px-3 py-1.5 text-xs font-medium text-gray-700 hover:bg-gray-50 text-left"
                onClick={() => handleTableClick(tableName)}
              >
                <span>{tableName}</span>
                <span className={BADGE_VARIANTS.gray}>{fields.length}</span>
              </button>
              <ul className="divide-y divide-gray-50 px-3 pb-1">
                {fields.map((f) => (
                  <li key={f.field_name}>
                    <button
                      className="w-full flex items-center gap-2 py-1 hover:bg-blue-50 rounded px-1 transition-colors text-left"
                      onClick={() => {
                        if (!projectId) return;
                        const table = tables.find((t) => t.name === f.table_name);
                        if (!table) return;
                        selectElement({ kind: "table", projectId, id: table.id, name: table.name });
                        setRightPanel({ kind: "field", projectId, tableId: table.id, tableName: table.name, fieldId: f.field_id });
                      }}
                    >
                      <span className="text-xs text-gray-700 flex-1">{f.field_name}</span>
                      <span className={BADGE_VARIANTS.purple}>{f.data_type}</span>
                      <span className={BADGE_VARIANTS.gray}>{f.field_type}</span>
                    </button>
                  </li>
                ))}
              </ul>
            </div>
          );
        })}
      </div>
      {allTableNames.length > PREVIEW_COUNT && (
        <button
          className="mt-2 text-xs text-blue-600 hover:underline"
          onClick={() => setExpanded((v) => !v)}
        >
          {expanded
            ? "折りたたむ"
            : `他 ${allTableNames.length - PREVIEW_COUNT} テーブルを表示`}
        </button>
      )}
    </div>
  );
}
