// src/components/detail/TableDetail.tsx
import { useMemo } from "react";
import { useTableFields, useTableList } from "../../hooks/table";
import { useAppStore } from "../../stores/appStore";
import { useColumnResize } from "../../hooks/useColumnResize";
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
  const { widths, setThRef, getResizeHandleProps } = useColumnResize(6);

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
    <div className="flex flex-col h-full">
      <div className="px-4 pt-4 pb-2 shrink-0">
        <h2 className="text-lg font-bold mb-2 text-gray-800">{name}</h2>
        {compareProjectId && diffMap.size > 0 && (
          <div className="flex gap-3 text-xs mb-1">
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
      </div>
      <div className="flex-1 overflow-auto [scrollbar-gutter:stable] px-4 pb-4">
      {allRows.length === 0 ? (
        <p className="text-gray-500">フィールドなし</p>
      ) : (
        <table className="w-full text-sm border-separate border-spacing-0 table-fixed [&_td]:overflow-hidden [&_th]:overflow-hidden">
            <colgroup>
              {widths.map((w, i) => <col key={i} style={w !== undefined ? { width: `${w}px` } : undefined} />)}
            </colgroup>
            <thead className="sticky top-0 bg-white z-10">
              <tr className="bg-gray-100 text-left">
                <th ref={setThRef(0)} className="px-3 py-2 border border-gray-200 relative">フィールド名<div className="absolute inset-y-0 right-0 w-1 cursor-col-resize select-none hover:bg-blue-400" {...getResizeHandleProps(0)} /></th>
                <th ref={setThRef(1)} className="px-3 py-2 border border-gray-200 relative">データ型<div className="absolute inset-y-0 right-0 w-1 cursor-col-resize select-none hover:bg-blue-400" {...getResizeHandleProps(1)} /></th>
                <th ref={setThRef(2)} className="px-3 py-2 border border-gray-200 relative">種別<div className="absolute inset-y-0 right-0 w-1 cursor-col-resize select-none hover:bg-blue-400" {...getResizeHandleProps(2)} /></th>
                <th ref={setThRef(3)} className="px-3 py-2 border border-gray-200 relative">グローバル<div className="absolute inset-y-0 right-0 w-1 cursor-col-resize select-none hover:bg-blue-400" {...getResizeHandleProps(3)} /></th>
                <th ref={setThRef(4)} className="px-3 py-2 border border-gray-200 relative">繰り返し<div className="absolute inset-y-0 right-0 w-1 cursor-col-resize select-none hover:bg-blue-400" {...getResizeHandleProps(4)} /></th>
                <th ref={setThRef(5)} className="px-3 py-2 border border-gray-200 relative">計算式<div className="absolute inset-y-0 right-0 w-1 cursor-col-resize select-none hover:bg-blue-400" {...getResizeHandleProps(5)} /></th>
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
      )}
      </div>
    </div>
  );
}
