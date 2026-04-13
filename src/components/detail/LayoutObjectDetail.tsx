// src/components/detail/LayoutObjectDetail.tsx
import React, { useMemo } from "react";
import { CODE_BLOCK } from "../../styles/tokens";
import { Spinner } from "../Spinner";
import {
  useLayoutObjects,
  useLayoutObjectConditions,
  useLayoutList,
  useResolveLayoutField,
} from "../../hooks/useTauriCommand";
import { useAppStore } from "../../stores/appStore";
import type { LayoutObjectRow } from "../../types/ddr";

interface Props {
  layoutObjectId: number;
  layoutId: number;
}

export function LayoutObjectDetail({ layoutObjectId, layoutId }: Props) {
  const { data: objects = [], isLoading } = useLayoutObjects(layoutId);
  const obj = objects.find((o) => o.id === layoutObjectId);

  const { selectedProject, selectedElement, diffContext, setRightPanel } = useAppStore();
  const projectId = selectedProject?.id ?? null;

  const { data: conditions = [] } = useLayoutObjectConditions(obj?.id ?? null);

  const { data: fieldLocation } = useResolveLayoutField(
    projectId,
    obj?.field_table_occurrence ?? null,
    obj?.field_name ?? null
  );

  // 差分コンテキストから比較オブジェクトを取得
  const compareProjectId = diffContext?.compareProjectId ?? null;
  const layoutName =
    selectedElement?.kind === "layout" ? selectedElement.name : null;

  const { data: compareLayouts = [] } = useLayoutList(compareProjectId);
  const compareLayout = useMemo(
    () => (layoutName ? compareLayouts.find((l) => l.name === layoutName) : null),
    [compareLayouts, layoutName]
  );
  const { data: compareObjects = [] } = useLayoutObjects(compareLayout?.id ?? null);
  const compareObj = useMemo(
    () => (obj ? compareObjects.find((o) => o.object_key === obj.object_key) : null),
    [compareObjects, obj]
  );

  if (isLoading) {
    return <div className="flex items-center gap-2 p-4 text-gray-500 text-sm"><Spinner className="w-4 h-4" />読み込み中...</div>;
  }

  if (!obj) {
    return <div className="p-4 text-gray-400 text-sm">オブジェクトが見つかりません</div>;
  }

  const hasBounds =
    obj.bound_top !== null &&
    obj.bound_left !== null &&
    obj.bound_bottom !== null &&
    obj.bound_right !== null;

  const width =
    hasBounds ? Math.round((obj.bound_right! - obj.bound_left!) * 10) / 10 : null;
  const height =
    hasBounds ? Math.round((obj.bound_bottom! - obj.bound_top!) * 10) / 10 : null;

  return (
    <div className="p-4 space-y-4 text-sm">
      {/* 種別 */}
      <div>
        <div className="text-xs font-semibold text-gray-500 uppercase tracking-wide mb-1">種別</div>
        <div className="font-mono text-gray-800">{obj.object_type}</div>
      </div>

      {/* オブジェクト名 */}
      {obj.object_name && (
        <div>
          <div className="text-xs font-semibold text-gray-500 uppercase tracking-wide mb-1">オブジェクト名</div>
          <div className="text-gray-800">{obj.object_name}</div>
        </div>
      )}

      {/* 位置・サイズ */}
      {hasBounds && (
        <div>
          <div className="text-xs font-semibold text-gray-500 uppercase tracking-wide mb-2">Location / Size</div>
          <div className="grid grid-cols-2 gap-x-6 gap-y-1 text-xs text-gray-700">
            <div className="flex justify-between">
              <span className="text-gray-500">X (Left)</span>
              <span className="font-mono">{obj.bound_left}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-500">Width</span>
              <span className="font-mono">{width}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-500">Y (Top)</span>
              <span className="font-mono">{obj.bound_top}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-500">Height</span>
              <span className="font-mono">{height}</span>
            </div>
          </div>
        </div>
      )}

      {/* フィールド参照 */}
      {obj.field_name && (
        <div>
          <div className="text-xs font-semibold text-gray-500 uppercase tracking-wide mb-1">フィールド</div>
          {fieldLocation && projectId !== null ? (
            <button
              className="text-blue-700 font-medium hover:underline text-left"
              onClick={() =>
                setRightPanel({
                  kind: "field",
                  projectId,
                  tableId: fieldLocation.table_id,
                  fieldId: fieldLocation.field_id,
                  tableName: fieldLocation.table_name,
                })
              }
            >
              {obj.field_table_occurrence
                ? `${obj.field_table_occurrence}::${obj.field_name}`
                : obj.field_name}
            </button>
          ) : (
            <span className="text-blue-700 font-medium">
              {obj.field_table_occurrence
                ? `${obj.field_table_occurrence}::${obj.field_name}`
                : obj.field_name}
            </span>
          )}
        </div>
      )}

      {/* ツールチップ */}
      <div>
        <div className="text-xs font-semibold text-gray-500 uppercase tracking-wide mb-1">ツールチップ</div>
        {obj.tooltip ? (
          <pre className={CODE_BLOCK}>{obj.tooltip}</pre>
        ) : (
          <span className="text-gray-400">なし</span>
        )}
      </div>

      {/* 条件非表示 */}
      <div>
        <div className="text-xs font-semibold text-gray-500 uppercase tracking-wide mb-1">条件非表示</div>
        {obj.hide_condition ? (
          <pre className={CODE_BLOCK}>{obj.hide_condition}</pre>
        ) : (
          <span className="text-gray-400">なし</span>
        )}
      </div>

      {/* 条件付き書式 */}
      <div>
        <div className="text-xs font-semibold text-gray-500 uppercase tracking-wide mb-2">条件付き書式</div>
        {conditions.length === 0 ? (
          <span className="text-gray-400">なし</span>
        ) : (
          <div className="space-y-2">
            {conditions.map((cond) => (
              <div
                key={cond.id}
                className="bg-gray-50 border border-gray-200 rounded p-2 text-xs space-y-1"
              >
                <div className="flex items-center gap-2">
                  <span className="text-gray-500 shrink-0">ルール {cond.rule_order + 1}</span>
                  {cond.format_css && (
                    <span
                      className="inline-block w-3 h-3 rounded-sm border border-gray-300 shrink-0"
                      style={parseCssColor(cond.format_css)}
                      title={cond.format_css}
                    />
                  )}
                </div>
                <pre className="font-mono whitespace-pre-wrap break-all text-gray-700">
                  {cond.calculation}
                </pre>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* 変更点（差分コンテキストがある場合のみ表示） */}
      {diffContext && (
        <div>
          <div className="text-xs font-semibold text-gray-500 uppercase tracking-wide mb-2">変更点</div>
          <DiffSection current={obj} compare={compareObj ?? undefined} />
        </div>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// 差分表示サブコンポーネント
// ---------------------------------------------------------------------------

interface DiffSectionProps {
  current: LayoutObjectRow;
  compare: LayoutObjectRow | undefined;
}

function DiffSection({ current, compare }: DiffSectionProps) {
  if (!compare) {
    return <span className="text-gray-400 text-xs">比較対象なし（新規追加オブジェクト）</span>;
  }

  const changes = collectChanges(compare, current);

  if (changes.length === 0) {
    return <span className="text-gray-400 text-xs">変更点なし</span>;
  }

  return (
    <div className="space-y-1.5">
      {changes.map((c) => (
        <div key={c.label} className="text-xs">
          <span className="font-medium text-gray-600">{c.label}: </span>
          <span className="text-red-600 line-through mr-1">{c.oldVal}</span>
          <span className="text-gray-400 mr-1">→</span>
          <span className="text-green-700">{c.newVal}</span>
        </div>
      ))}
    </div>
  );
}

interface Change {
  label: string;
  oldVal: string;
  newVal: string;
}

function boundsStr(obj: LayoutObjectRow): string | null {
  if (
    obj.bound_top === null ||
    obj.bound_left === null ||
    obj.bound_bottom === null ||
    obj.bound_right === null
  ) {
    return null;
  }
  const w = Math.round((obj.bound_right - obj.bound_left) * 10) / 10;
  const h = Math.round((obj.bound_bottom - obj.bound_top) * 10) / 10;
  return `(L:${obj.bound_left} T:${obj.bound_top} W:${w} H:${h})`;
}

function collectChanges(oldObj: LayoutObjectRow, newObj: LayoutObjectRow): Change[] {
  const changes: Change[] = [];

  const oldBounds = boundsStr(oldObj);
  const newBounds = boundsStr(newObj);
  if (oldBounds !== newBounds) {
    changes.push({
      label: "位置",
      oldVal: oldBounds ?? "なし",
      newVal: newBounds ?? "なし",
    });
  }

  const textFields: Array<{ key: keyof LayoutObjectRow; label: string }> = [
    { key: "object_name", label: "オブジェクト名" },
    { key: "button_label", label: "ボタンラベル" },
    { key: "tooltip", label: "ツールチップ" },
    { key: "hide_condition", label: "条件非表示" },
    { key: "field_table_occurrence", label: "テーブルオカレンス" },
    { key: "field_name", label: "フィールド名" },
  ];

  for (const { key, label } of textFields) {
    const oldVal = (oldObj[key] as string | null) ?? null;
    const newVal = (newObj[key] as string | null) ?? null;
    if (oldVal !== newVal) {
      changes.push({
        label,
        oldVal: oldVal ?? "なし",
        newVal: newVal ?? "なし",
      });
    }
  }

  return changes;
}

/** CSS 文字列から color プロパティのみ抽出してインラインスタイルに変換する。 */
function parseCssColor(css: string): React.CSSProperties {
  const match = css.match(/color\s*:\s*([^;]+)/);
  return match ? { backgroundColor: match[1].trim() } : {};
}
