// src/components/navigation/CategoryTree.tsx
import { useState, useEffect, useRef, forwardRef, useCallback } from "react";
import { flushSync } from "react-dom";
import { useTableList, useTableOccurrenceList, useRelationshipList } from "../../hooks/table";
import { useScriptList } from "../../hooks/script";
import { useLayoutList } from "../../hooks/layout";
import { useValueListList, useCustomFunctionList } from "../../hooks/catalog";
import { useAppStore } from "../../stores/appStore";
import type { SelectedElement } from "../../stores/appStore";

interface Props {
  projectId: number;
}

interface CategorySectionProps {
  label: string;
  icon: string;
  count: number;
  isOpen: boolean;
  onToggle: () => void;
  children: React.ReactNode;
}

function CategorySection({
  label,
  icon,
  count,
  isOpen,
  onToggle,
  children,
}: CategorySectionProps) {
  return (
    <div>
      <button
        className="w-full flex items-center justify-between px-9 py-1.5 text-sm font-semibold text-gray-700 hover:bg-gray-100 transition-colors"
        onClick={onToggle}
      >
        <span className="flex items-center gap-1.5">
          <span className="text-base">{icon}</span>
          {label} ({count})
        </span>
        <span className="text-xs text-gray-400">{isOpen ? "▲" : "▼"}</span>
      </button>
      {isOpen && <div className="bg-gray-50">{children}</div>}
    </div>
  );
}

interface ItemRowProps {
  name: string;
  isSelected: boolean;
  onClick: () => void;
}

const ItemRow = forwardRef<HTMLButtonElement, ItemRowProps>(
  ({ name, isSelected, onClick }, ref) => (
    <button
      ref={ref}
      className={`w-full text-left px-12 py-1.5 text-sm truncate hover:bg-blue-50 transition-colors ${
        isSelected ? "bg-blue-100 text-blue-800 font-medium" : "text-gray-600"
      }`}
      onClick={onClick}
      title={name}
    >
      {name}
    </button>
  )
);

export function CategoryTree({ projectId }: Props) {
  const [openCategories, setOpenCategories] = useState<Record<string, boolean>>(
    {}
  );
  const { selectedElement, selectElement, setRightPanel } = useAppStore();
  const selectedItemRef = useRef<HTMLButtonElement>(null);
  // データの遅延ロード後にスクロールが必要なときにセットする
  const pendingScrollRef = useRef(false);

  // カテゴリが開いているときのみ projectId を渡し、クエリを有効化する
  const tablesProjectId = openCategories["tables"] ? projectId : null;
  const scriptsProjectId = openCategories["scripts"] ? projectId : null;
  const layoutsProjectId = openCategories["layouts"] ? projectId : null;
  const valueListsProjectId = openCategories["value_lists"] ? projectId : null;
  const customFunctionsProjectId = openCategories["custom_functions"] ? projectId : null;

  const { data: tables = [] } = useTableList(tablesProjectId);
  const { data: scripts = [] } = useScriptList(scriptsProjectId);
  const { data: layouts = [] } = useLayoutList(layoutsProjectId);
  const { data: valueLists = [] } = useValueListList(valueListsProjectId);
  const { data: customFunctions = [] } = useCustomFunctionList(customFunctionsProjectId);
  const { data: tableOccurrences = [] } = useTableOccurrenceList(projectId);
  const { data: relationships = [] } = useRelationshipList(projectId);

  const toggle = (category: string) => {
    setOpenCategories((prev) => ({ ...prev, [category]: !prev[category] }));
  };

  const tryScroll = useCallback(() => {
    if (pendingScrollRef.current && selectedItemRef.current) {
      pendingScrollRef.current = false;
      selectedItemRef.current.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }
  }, []);

  // データ到着後にスクロールを再試行（遅延ロード時に ref が null だったケース）
  useEffect(() => {
    tryScroll();
  }, [tables, scripts, layouts, valueLists, customFunctions, tryScroll]);

  // selectedElement が変化したとき、対応するカテゴリを自動展開してハイライト行にスクロールする
  // flushSync でカテゴリ展開を同期的に DOM に反映してから scrollIntoView を呼ぶことで
  // ref が確実に設定済みになり、requestAnimationFrame による遅延・競合を回避する
  useEffect(() => {
    if (!selectedElement) return;
    const kindToCategory: Partial<Record<string, string>> = {
      table: "tables",
      script: "scripts",
      layout: "layouts",
      value_list: "value_lists",
      custom_function: "custom_functions",
    };
    const category = kindToCategory[selectedElement.kind];
    if (category) {
      flushSync(() => {
        setOpenCategories((prev) => ({ ...prev, [category]: true }));
      });
    }
    // flushSync 後は DOM が更新済みで ref も設定されているため直接スクロールできる
    // 遅延ロード中（データ未到着）の場合は pendingScrollRef に記録して後で再試行する
    if (selectedItemRef.current) {
      selectedItemRef.current.scrollIntoView({ block: "nearest", behavior: "smooth" });
    } else if (category) {
      pendingScrollRef.current = true;
    }
  }, [selectedElement]);

  const isSelected = (element: SelectedElement) => {
    if (!selectedElement || !element) return false;
    if (selectedElement.kind !== element.kind) return false;
    if ("id" in selectedElement && "id" in element) {
      return selectedElement.id === element.id;
    }
    // id を持たない kind（all_fields など）はkind一致で選択済みとみなす
    return true;
  };

  const totalFieldCount = tables.reduce((sum, t) => sum + t.field_count, 0);

  return (
    <div className="overflow-auto">
      {/* フィールド（テーブル横断一覧） */}
      <button
        className={`w-full flex items-center justify-between px-9 py-1.5 text-sm font-semibold transition-colors ${
          isSelected({ kind: "all_fields", projectId })
            ? "bg-blue-100 text-blue-800"
            : "text-gray-700 hover:bg-gray-100"
        }`}
        onClick={() => {
          setRightPanel(null);
          selectElement({ kind: "all_fields", projectId });
        }}
      >
        <span className="flex items-center gap-1.5">
          <span className="text-base">🔢</span>
          フィールド ({totalFieldCount})
        </span>
      </button>

      {/* テーブル */}
      <CategorySection
        label="テーブル"
        icon="📊"
        count={tables.length}
        isOpen={!!openCategories["tables"]}
        onToggle={() => toggle("tables")}
      >
        {tables.map((t) => {
          const sel = isSelected({ kind: "table", projectId, id: t.id, name: t.name });
          return (
            <ItemRow
              key={t.id}
              ref={sel ? selectedItemRef : null}
              name={t.name}
              isSelected={sel}
              onClick={() => {
                setRightPanel(null);
                selectElement({ kind: "table", projectId, id: t.id, name: t.name });
              }}
            />
          );
        })}
      </CategorySection>

      {/* テーブルオカレンス */}
      <button
        className={`w-full flex items-center justify-between px-9 py-1.5 text-sm font-semibold transition-colors ${
          isSelected({ kind: "all_table_occurrences", projectId })
            ? "bg-blue-100 text-blue-800"
            : "text-gray-700 hover:bg-gray-100"
        }`}
        onClick={() => {
          setRightPanel(null);
          selectElement({ kind: "all_table_occurrences", projectId });
        }}
      >
        <span className="flex items-center gap-1.5">
          <span className="text-base">🔁</span>
          テーブルオカレンス ({tableOccurrences.length})
        </span>
      </button>

      {/* リレーション */}
      <button
        className={`w-full flex items-center justify-between px-9 py-1.5 text-sm font-semibold transition-colors ${
          isSelected({ kind: "all_relationships", projectId })
            ? "bg-blue-100 text-blue-800"
            : "text-gray-700 hover:bg-gray-100"
        }`}
        onClick={() => {
          setRightPanel(null);
          selectElement({ kind: "all_relationships", projectId });
        }}
      >
        <span className="flex items-center gap-1.5">
          <span className="text-base">🔗</span>
          リレーション ({relationships.length})
        </span>
      </button>

      {/* スクリプト */}
      <CategorySection
        label="スクリプト"
        icon="📜"
        count={scripts.length}
        isOpen={!!openCategories["scripts"]}
        onToggle={() => toggle("scripts")}
      >
        {scripts.map((s) => {
          const sel = isSelected({ kind: "script", projectId, id: s.id, name: s.name });
          return (
            <ItemRow
              key={s.id}
              ref={sel ? selectedItemRef : null}
              name={s.name}
              isSelected={sel}
              onClick={() => {
                setRightPanel(null);
                selectElement({ kind: "script", projectId, id: s.id, name: s.name });
              }}
            />
          );
        })}
      </CategorySection>

      {/* レイアウト */}
      <CategorySection
        label="レイアウト"
        icon="🖼"
        count={layouts.length}
        isOpen={!!openCategories["layouts"]}
        onToggle={() => toggle("layouts")}
      >
        {layouts.map((l) => {
          const sel = isSelected({ kind: "layout", projectId, id: l.id, name: l.name });
          return (
            <ItemRow
              key={l.id}
              ref={sel ? selectedItemRef : null}
              name={l.name}
              isSelected={sel}
              onClick={() => {
                setRightPanel(null);
                selectElement({ kind: "layout", projectId, id: l.id, name: l.name });
              }}
            />
          );
        })}
      </CategorySection>

      {/* バリューリスト */}
      <CategorySection
        label="バリューリスト"
        icon="📋"
        count={valueLists.length}
        isOpen={!!openCategories["value_lists"]}
        onToggle={() => toggle("value_lists")}
      >
        {valueLists.map((v) => {
          const sel = isSelected({ kind: "value_list", projectId, id: v.id, name: v.name });
          return (
            <ItemRow
              key={v.id}
              ref={sel ? selectedItemRef : null}
              name={v.name}
              isSelected={sel}
              onClick={() => {
                setRightPanel(null);
                selectElement({ kind: "value_list", projectId, id: v.id, name: v.name });
              }}
            />
          );
        })}
      </CategorySection>

      {/* カスタム関数 */}
      <CategorySection
        label="カスタム関数"
        icon="ƒ"
        count={customFunctions.length}
        isOpen={!!openCategories["custom_functions"]}
        onToggle={() => toggle("custom_functions")}
      >
        {customFunctions.map((cf) => {
          const sel = isSelected({ kind: "custom_function", projectId, id: cf.id, name: cf.name });
          return (
            <ItemRow
              key={cf.id}
              ref={sel ? selectedItemRef : null}
              name={cf.name}
              isSelected={sel}
              onClick={() => {
                setRightPanel(null);
                selectElement({ kind: "custom_function", projectId, id: cf.id, name: cf.name });
              }}
            />
          );
        })}
      </CategorySection>

      {/* リレーショングラフ */}
      <button
        className={`w-full flex items-center gap-1.5 px-9 py-1.5 text-sm font-semibold transition-colors ${
          isSelected({ kind: "relationship_graph", projectId })
            ? "bg-blue-100 text-blue-800"
            : "text-gray-700 hover:bg-gray-100"
        }`}
        onClick={() => {
          setRightPanel(null);
          selectElement({ kind: "relationship_graph", projectId });
        }}
      >
        <span className="text-base">🗺</span>
        リレーショングラフ
      </button>

      {/* セキュリティ */}
      <button
        className={`w-full flex items-center gap-1.5 px-9 py-1.5 text-sm font-semibold transition-colors ${
          isSelected({ kind: "security", projectId })
            ? "bg-blue-100 text-blue-800"
            : "text-gray-700 hover:bg-gray-100"
        }`}
        onClick={() => {
          setRightPanel(null);
          selectElement({ kind: "security", projectId });
        }}
      >
        <span className="text-base">🔒</span>
        セキュリティ
      </button>


    </div>
  );
}
