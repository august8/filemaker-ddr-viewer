// src/components/detail/TableDetail.tsx
import { useMemo } from "react";
import { useTableFields, useTableList } from "../../hooks/table";
import { useAppStore } from "../../stores/appStore";
import type { FieldRow } from "../../types/ddr";
import { Spinner } from "../Spinner";
import { BADGE_VARIANTS } from "../../styles/tokens";

interface Props {
  projectId: number;
  tableId: number;
  name: string;
}

type DiffKind = "Added" | "Removed" | "Modified";

interface FieldWithDiff extends FieldRow {
  diffKind?: DiffKind;
  isPhantom?: boolean; // 比較元にのみ存在する行（Removed）
}

const DIFF_ROW_BG: Record<DiffKind, string> = {
  Added: "bg-green-50",
  Removed: "bg-red-50",
  Modified: "bg-yellow-50",
};

const DIFF_NAME_CLASS: Record<DiffKind, string> = {
  Added: "text-green-700 font-medium",
  Removed: "text-red-600 font-medium line-through",
  Modified: "text-yellow-700 font-medium",
};

export function TableDetail({ projectId, tableId, name }: Props) {
  const { data: fields = [], isLoading } = useTableFields(projectId, tableId);
  const { rightPanel, setRightPanel, diffContext } = useAppStore();

  // diff ハイライト用: 比較元プロジェクトのフィールドを取得
  const compareProjectId = diffContext?.compareProjectId ?? null;
  const { data: compareTables = [] } = useTableList(compareProjectId);
  const compareTable = useMemo(
    () => compareTables.find((t) => t.name === name),
    [compareTables, name]
  );
  const { data: compareFields = [] } = useTableFields(
    compareProjectId,
    compareTable?.id ?? null
  );

  // フィールドごとの差分種別を計算
  const diffMap = useMemo((): Map<string, DiffKind> => {
    if (!compareProjectId) return new Map();
    const currentNames = new Set(fields.map((f) => f.name));
    const compareByName = new Map(compareFields.map((f) => [f.name, f]));
    const result = new Map<string, DiffKind>();
    for (const field of fields) {
      if (!compareByName.has(field.name)) {
        result.set(field.name, "Added");
      } else {
        const cmp = compareByName.get(field.name)!;
        if (
          field.data_type !== cmp.data_type ||
          field.field_type !== cmp.field_type ||
          field.is_global !== cmp.is_global ||
          field.max_repeat !== cmp.max_repeat ||
          field.calculation !== cmp.calculation
        ) {
          result.set(field.name, "Modified");
        }
      }
    }
    // 比較元にのみ存在するフィールドを Removed としてマーク
    for (const [cmpName] of compareByName) {
      if (!currentNames.has(cmpName)) {
        result.set(cmpName, "Removed");
      }
    }
    return result;
  }, [fields, compareFields, compareProjectId]);

  // Removed フィールドのファントム行（比較元から取得）を末尾に追加
  const allRows = useMemo((): FieldWithDiff[] => {
    const phantoms: FieldWithDiff[] = compareFields
      .filter((f) => diffMap.get(f.name) === "Removed")
      .map((f) => ({ ...f, diffKind: "Removed" as DiffKind, isPhantom: true }));
    return [
      ...fields.map((f) => ({ ...f, diffKind: diffMap.get(f.name) })),
      ...phantoms,
    ];
  }, [fields, compareFields, diffMap]);

  if (isLoading) {
    return <div className="flex items-center gap-2 p-4 text-gray-500 text-sm"><Spinner className="w-4 h-4" />読み込み中...</div>;
  }

  return (
    <div className="p-4">
      <h2 className="text-lg font-bold mb-4 text-gray-800">{name}</h2>
      {compareProjectId && diffMap.size > 0 && (
        <div className="flex gap-3 text-xs mb-3">
          {[...new Set(diffMap.values())].includes("Added") && (
            <span className="inline-flex items-center gap-1 text-green-700">
              <span className="w-2 h-2 rounded-full bg-green-400 inline-block" />
              追加
            </span>
          )}
          {[...new Set(diffMap.values())].includes("Removed") && (
            <span className="inline-flex items-center gap-1 text-red-600">
              <span className="w-2 h-2 rounded-full bg-red-400 inline-block" />
              削除
            </span>
          )}
          {[...new Set(diffMap.values())].includes("Modified") && (
            <span className="inline-flex items-center gap-1 text-yellow-700">
              <span className="w-2 h-2 rounded-full bg-yellow-400 inline-block" />
              変更
            </span>
          )}
        </div>
      )}
      {allRows.length === 0 ? (
        <p className="text-gray-500">フィールドなし</p>
      ) : (
        <div className="overflow-auto">
          <table className="w-full text-sm border-collapse">
            <thead>
              <tr className="bg-gray-100 text-left">
                <th className="px-3 py-2 border border-gray-200">フィールド名</th>
                <th className="px-3 py-2 border border-gray-200">データ型</th>
                <th className="px-3 py-2 border border-gray-200">種別</th>
                <th className="px-3 py-2 border border-gray-200">グローバル</th>
                <th className="px-3 py-2 border border-gray-200">繰り返し</th>
                <th className="px-3 py-2 border border-gray-200">計算式</th>
              </tr>
            </thead>
            <tbody>
              {allRows.map((field) => {
                const isSelected =
                  !field.isPhantom &&
                  rightPanel?.kind === "field" &&
                  rightPanel.fieldId === field.id;
                const dk = field.diffKind;
                const rowBg = dk
                  ? DIFF_ROW_BG[dk]
                  : isSelected
                  ? "bg-blue-50 hover:bg-blue-100"
                  : "hover:bg-blue-50";
                const nameClass = dk ? DIFF_NAME_CLASS[dk] : "font-medium";
                return (
                  <tr
                    key={field.isPhantom ? `phantom-${field.name}` : field.id}
                    className={`${rowBg} ${field.isPhantom ? "cursor-default opacity-70" : "cursor-pointer"}`}
                    onClick={() => {
                      if (field.isPhantom) return;
                      setRightPanel({
                        kind: "field",
                        fieldId: field.id,
                        tableId,
                        projectId,
                        tableName: name,
                      });
                    }}
                  >
                    <td className={`px-3 py-2 border border-gray-200 ${nameClass}`}>
                      {field.name}
                    </td>
                    <td className="px-3 py-2 border border-gray-200">
                      {field.data_type}
                    </td>
                    <td className="px-3 py-2 border border-gray-200">
                      {field.field_type}
                    </td>
                    <td className="px-3 py-2 border border-gray-200 text-center">
                      {field.is_global && (
                        <span className={BADGE_VARIANTS.blue}>G</span>
                      )}
                    </td>
                    <td className="px-3 py-2 border border-gray-200 text-center">
                      {field.max_repeat}
                    </td>
                    <td className="px-3 py-2 border border-gray-200 font-mono text-xs max-w-xs truncate" title={field.calculation ?? ""}>
                      {field.calculation ?? ""}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
