import { useProjectSummary } from "../hooks/solutions";
import { useAppStore } from "../stores/appStore";
import { Spinner } from "./Spinner";
import { CARD } from "../styles/tokens";

interface Props {
  projectId: number | null;
}

interface SummaryItem {
  label: string;
  value: number;
  onClick?: () => void;
}

export function ProjectSummaryCard({ projectId }: Props) {
  const { data: summary, isLoading } = useProjectSummary(projectId);
  const { selectElement } = useAppStore();

  if (projectId === null) return null;
  if (isLoading) return <div className="flex items-center gap-2 text-gray-400 text-sm"><Spinner className="w-4 h-4" />読み込み中...</div>;
  if (!summary) return null;

  const pid = projectId;
  const items: SummaryItem[] = [
    { label: "テーブル",           value: summary.table_count,            onClick: () => selectElement({ kind: "all_tables", projectId: pid }) },
    { label: "フィールド",         value: summary.field_count },
    { label: "スクリプト",         value: summary.script_count,           onClick: () => selectElement({ kind: "all_scripts", projectId: pid }) },
    { label: "レイアウト",         value: summary.layout_count,           onClick: () => selectElement({ kind: "all_layouts", projectId: pid }) },
    { label: "テーブルオカレンス", value: summary.table_occurrence_count, onClick: () => selectElement({ kind: "all_table_occurrences", projectId: pid }) },
    { label: "リレーション",       value: summary.relationship_count,     onClick: () => selectElement({ kind: "all_relationships", projectId: pid }) },
    { label: "バリューリスト",     value: summary.value_list_count,       onClick: () => selectElement({ kind: "all_value_lists", projectId: pid }) },
    { label: "カスタム関数",       value: summary.custom_function_count,  onClick: () => selectElement({ kind: "all_custom_functions", projectId: pid }) },
  ];

  return (
    <div className={CARD}>
      <div className="flex items-center justify-between mb-3">
        <h2 className="font-semibold text-gray-800">{summary.project.name}</h2>
        <span className="text-xs text-gray-400 bg-gray-100 rounded px-2 py-0.5">
          FM {summary.project.fm_version}
        </span>
      </div>
      <div className="grid grid-cols-4 gap-3">
        {items.map(({ label, value, onClick }) => (
          <div key={label} className="text-center">
            {onClick ? (
              <button
                className="text-2xl font-bold text-blue-600 hover:text-blue-800 cursor-pointer transition-colors"
                onClick={onClick}
              >
                {value}
              </button>
            ) : (
              <div className="text-2xl font-bold text-blue-600">{value}</div>
            )}
            <div className="text-xs text-gray-500">{label}</div>
          </div>
        ))}
      </div>
    </div>
  );
}
