import { useSolutionProjectSummaries } from "../hooks/solutions";
import { useAppStore } from "../stores/appStore";
import { Spinner } from "./Spinner";

interface Props {
  solutionId: number;
  solutionName: string;
}

const COLUMNS = [
  { key: "table_count",              label: "テーブル",       kind: "all_tables" },
  { key: "field_count",              label: "フィールド",     kind: "all_fields" },
  { key: "script_count",             label: "スクリプト",     kind: "all_scripts" },
  { key: "layout_count",             label: "レイアウト",     kind: "all_layouts" },
  { key: "table_occurrence_count",   label: "TO",             kind: "all_table_occurrences" },
  { key: "relationship_count",       label: "リレーション",   kind: "all_relationships" },
  { key: "value_list_count",         label: "バリューリスト", kind: "all_value_lists" },
  { key: "custom_function_count",    label: "カスタム関数",   kind: "all_custom_functions" },
] as const;

type ColKey = (typeof COLUMNS)[number]["key"];

export function SolutionDashboard({ solutionId, solutionName }: Props) {
  const { data: summaries, isLoading } = useSolutionProjectSummaries(solutionId);
  const { selectElement, navigateToProject } = useAppStore();

  if (isLoading) {
    return (
      <div className="flex-1 flex items-center justify-center gap-2 text-gray-400 text-sm" data-testid="solution-dashboard-spinner">
        <Spinner className="w-4 h-4" />読み込み中...
      </div>
    );
  }

  if (!summaries || summaries.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center text-gray-400 text-sm" data-testid="solution-dashboard-empty">
        「{solutionName}」にはファイルがありません
      </div>
    );
  }

  const total = (key: ColKey) => summaries.reduce((s, r) => s + r[key], 0);
  const avg   = (key: ColKey) => (total(key) / summaries.length).toFixed(1);

  return (
    <div className="flex-1 overflow-auto p-4">
      <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
        <div className="px-4 py-3 border-b border-gray-100">
          <h2 className="font-semibold text-gray-800">{solutionName}</h2>
          <p className="text-xs text-gray-400">{summaries.length} ファイル</p>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead className="bg-gray-50 text-xs text-gray-500 uppercase tracking-wide">
              <tr>
                <th className="px-4 py-2 text-left font-medium">ファイル</th>
                <th className="px-2 py-2 text-left text-xs text-gray-400">バージョン</th>
                {COLUMNS.map(c => (
                  <th key={c.key} className="px-3 py-2 text-right font-medium">{c.label}</th>
                ))}
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-100">
              {summaries.map(s => (
                <tr
                  key={s.project.id}
                  className="hover:bg-blue-50 cursor-pointer transition-colors"
                  onClick={() => navigateToProject(s.project)}
                >
                  <td className="px-4 py-2 font-medium text-blue-700 truncate max-w-[180px]" title={s.project.name}>
                    {s.project.name}
                  </td>
                  <td className="px-2 py-2 text-xs text-gray-400">{s.project.fm_version}</td>
                  {COLUMNS.map(c => (
                    <td key={c.key} className="px-3 py-2 text-right">
                      <button
                        className="text-blue-600 hover:text-blue-800 hover:underline"
                        onClick={e => {
                          e.stopPropagation();
                          selectElement({ kind: c.kind, projectId: s.project.id });
                        }}
                      >
                        {s[c.key]}
                      </button>
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
            <tfoot className="border-t-2 border-gray-300 text-xs">
              <tr data-testid="solution-total-row" className="bg-slate-100 font-bold text-gray-800">
                <td className="px-4 py-2" colSpan={2}>合計</td>
                {COLUMNS.map(c => (
                  <td key={c.key} className="px-3 py-2 text-right">{total(c.key)}</td>
                ))}
              </tr>
              <tr data-testid="solution-average-row" className="bg-slate-50 text-gray-500 italic">
                <td className="px-4 py-2" colSpan={2}>平均</td>
                {COLUMNS.map(c => (
                  <td key={c.key} className="px-3 py-2 text-right">{avg(c.key)}</td>
                ))}
              </tr>
            </tfoot>
          </table>
        </div>
      </div>
    </div>
  );
}
