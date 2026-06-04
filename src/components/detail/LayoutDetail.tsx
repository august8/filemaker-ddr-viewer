// src/components/detail/LayoutDetail.tsx
import { useMemo } from "react";
import { useLayoutTriggers, useLayoutObjects, useLayoutList } from "../../hooks/layout";
import { useScriptList } from "../../hooks/script";
import { useAppStore } from "../../stores/appStore";
import type { LayoutRow, LayoutObjectRow } from "../../types/ddr";
import { Spinner } from "../Spinner";
import { SECTION_HEADER } from "../../styles/tokens";

interface Props {
  layout: LayoutRow;
  projectId: number;
}

type DiffKind = "Added" | "Removed" | "Modified";

interface ObjectWithDiff extends LayoutObjectRow {
  diffKind?: DiffKind;
  isPhantom?: boolean;
}

const DIFF_ROW_BG: Record<DiffKind, string> = {
  Added: "bg-green-50",
  Removed: "bg-red-50",
  Modified: "bg-yellow-50",
};

const DIFF_TEXT: Record<DiffKind, string> = {
  Added: "text-green-700",
  Removed: "text-red-600 line-through",
  Modified: "text-yellow-700",
};

function objectSignature(obj: LayoutObjectRow): string {
  return [
    obj.object_type,
    obj.object_name ?? "",
    obj.button_label ?? "",
    obj.field_table_occurrence ?? "",
    obj.field_name ?? "",
    obj.tooltip ?? "",
    obj.hide_condition ?? "",
    obj.bound_top ?? "",
    obj.bound_left ?? "",
    obj.bound_bottom ?? "",
    obj.bound_right ?? "",
  ].join("|");
}

export function LayoutDetail({ layout, projectId }: Props) {
  const { data: triggers = [], isLoading: triggersLoading } = useLayoutTriggers(layout.id);
  const { data: objects = [], isLoading: objectsLoading } = useLayoutObjects(layout.id);
  const { data: scripts = [] } = useScriptList(projectId);
  const { setRightPanel, rightPanel, selectElement, diffContext } = useAppStore();

  // diff ハイライト用: 比較元プロジェクトの同名レイアウトのオブジェクトを取得
  const compareProjectId = diffContext?.compareProjectId ?? null;
  const { data: compareLayouts = [] } = useLayoutList(compareProjectId);
  const compareLayout = useMemo(
    () => compareLayouts.find((l) => l.name === layout.name),
    [compareLayouts, layout.name]
  );
  const { data: compareObjects = [] } = useLayoutObjects(compareLayout?.id ?? null);

  // オブジェクトごとの差分種別を object_key で計算
  const diffMap = useMemo((): Map<number, DiffKind> => {
    if (!compareProjectId) return new Map();
    const currentKeys = new Set(objects.map((o) => o.object_key));
    const compareByKey = new Map(compareObjects.map((o) => [o.object_key, o]));
    const result = new Map<number, DiffKind>();
    for (const obj of objects) {
      if (!compareByKey.has(obj.object_key)) {
        result.set(obj.object_key, "Added");
      } else {
        const cmp = compareByKey.get(obj.object_key)!;
        if (objectSignature(obj) !== objectSignature(cmp)) {
          result.set(obj.object_key, "Modified");
        }
      }
    }
    for (const [key] of compareByKey) {
      if (!currentKeys.has(key)) {
        result.set(key, "Removed");
      }
    }
    return result;
  }, [objects, compareObjects, compareProjectId]);

  // Removed オブジェクトのファントム行（比較元から取得）を末尾に追加
  const allObjects = useMemo((): ObjectWithDiff[] => {
    const phantoms: ObjectWithDiff[] = compareObjects
      .filter((o) => diffMap.get(o.object_key) === "Removed")
      .map((o) => ({ ...o, diffKind: "Removed" as DiffKind, isPhantom: true }));
    return [
      ...objects.map((o) => ({ ...o, diffKind: diffMap.get(o.object_key) })),
      ...phantoms,
    ];
  }, [objects, compareObjects, diffMap]);

  const isLoading = triggersLoading || objectsLoading;

  if (isLoading) {
    return <div className="flex items-center gap-2 p-4 text-gray-500 text-sm"><Spinner className="w-4 h-4" />読み込み中...</div>;
  }

  return (
    <div className="p-4">
      <h2 className="text-lg font-bold mb-4 text-gray-800">{layout.name}</h2>

      {/* テーブルオカレンス */}
      <div className="mb-4">
        <span className="text-sm font-semibold text-gray-600">テーブルオカレンス: </span>
        <span className="text-sm text-gray-800">
          {layout.table_occurrence_name ?? "—"}
        </span>
      </div>

      {/* トリガー一覧 */}
      <h3 className={SECTION_HEADER}>スクリプトトリガー</h3>
      {triggers.length === 0 ? (
        <p className="text-gray-500 text-sm mb-6">トリガーなし</p>
      ) : (
        <table className="w-full text-sm border-collapse mb-6">
          <thead>
            <tr className="bg-gray-100 text-left">
              <th className="px-3 py-2 border border-gray-200">イベント</th>
              <th className="px-3 py-2 border border-gray-200">スクリプト名</th>
              <th className="px-3 py-2 border border-gray-200">ファイル</th>
            </tr>
          </thead>
          <tbody>
            {triggers.map((trigger) => {
              const targetScript = scripts.find((s) => s.name === trigger.script_name);
              return (
                <tr key={trigger.id} className="hover:bg-blue-50">
                  <td className="px-3 py-2 border border-gray-200 font-mono text-xs">
                    {trigger.event}
                  </td>
                  <td className="px-3 py-2 border border-gray-200">
                    {targetScript ? (
                      <button
                        className="text-blue-600 hover:underline text-left"
                        onClick={() => {
                          setRightPanel(null);
                          selectElement({
                            kind: "script",
                            projectId,
                            id: targetScript.id,
                            name: targetScript.name,
                          });
                        }}
                      >
                        {trigger.script_name}
                      </button>
                    ) : (
                      <span className="text-gray-500">{trigger.script_name}</span>
                    )}
                  </td>
                  <td className="px-3 py-2 border border-gray-200 text-gray-500 text-xs">
                    {trigger.file_name}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      )}

      {/* オブジェクト一覧 */}
      <h3 className={SECTION_HEADER}>
        オブジェクト一覧
        <span className="ml-2 text-xs font-normal text-gray-400">
          クリックで詳細を表示
        </span>
      </h3>
      {compareProjectId && diffMap.size > 0 && (
        <div className="flex gap-3 text-xs mb-2">
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
      {allObjects.length === 0 ? (
        <p className="text-gray-500 text-sm">オブジェクトなし</p>
      ) : (
        <table className="w-full text-sm border-collapse">
          <thead>
            <tr className="bg-gray-100 text-left">
              <th className="px-3 py-2 border border-gray-200">種別</th>
              <th className="px-3 py-2 border border-gray-200">オブジェクト名</th>
              <th className="px-3 py-2 border border-gray-200">ラベル</th>
              <th className="px-3 py-2 border border-gray-200">フィールド</th>
              <th className="px-3 py-2 border border-gray-200">ツールチップ</th>
              <th className="px-3 py-2 border border-gray-200">条件非表示</th>
            </tr>
          </thead>
          <tbody>
            {allObjects.map((obj) => {
              const isSelected =
                !obj.isPhantom &&
                rightPanel?.kind === "layout_object" &&
                rightPanel.layoutObjectId === obj.id;
              const isBrokenField =
                obj.object_type === "Field" &&
                (obj.field_table_occurrence === "" || obj.field_name === "");
              const dk = obj.diffKind;
              const rowBg = dk
                ? DIFF_ROW_BG[dk]
                : isBrokenField
                ? "bg-red-50"
                : isSelected
                ? "bg-blue-100 text-blue-800"
                : "hover:bg-blue-50";
              const textClass = dk ? DIFF_TEXT[dk] : "";
              return (
                <tr
                  key={obj.isPhantom ? `phantom-${obj.object_key}` : obj.id}
                  className={`${rowBg} ${obj.isPhantom ? "cursor-default opacity-70" : "cursor-pointer"}`}
                  onClick={() => {
                    if (obj.isPhantom) return;
                    setRightPanel(
                      isSelected
                        ? null
                        : { kind: "layout_object", layoutObjectId: obj.id, layoutId: layout.id }
                    );
                  }}
                >
                  <td className={`px-3 py-2 border border-gray-200 text-xs font-mono ${textClass}`}>
                    {obj.object_type}
                  </td>
                  <td className={`px-3 py-2 border border-gray-200 text-xs ${textClass}`}>
                    {obj.object_name ?? <span className="text-gray-300">—</span>}
                  </td>
                  <td className={`px-3 py-2 border border-gray-200 text-xs ${textClass}`}>
                    {obj.button_label ?? <span className="text-gray-300">—</span>}
                  </td>
                  <td className={`px-3 py-2 border border-gray-200 ${textClass}`}>
                    {isBrokenField
                      ? <span className="text-red-500 text-xs">フィールドが見つかりません #{obj.object_key}</span>
                      : obj.field_table_occurrence && obj.field_name
                      ? `${obj.field_table_occurrence}::${obj.field_name}`
                      : <span className="text-gray-400">—</span>}
                  </td>
                  <td className={`px-3 py-2 border border-gray-200 text-xs text-gray-500 max-w-xs truncate ${textClass}`} title={obj.tooltip ?? ""}>
                    {obj.tooltip ?? <span className="text-gray-300">—</span>}
                  </td>
                  <td className={`px-3 py-2 border border-gray-200 text-xs text-gray-500 max-w-xs truncate ${textClass}`} title={obj.hide_condition ?? ""}>
                    {obj.hide_condition ?? <span className="text-gray-300">—</span>}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      )}
    </div>
  );
}
