import { useReportCard } from "../hooks/analysis";
import { useScriptList } from "../hooks/script";
import { useLayoutList } from "../hooks/layout";
import { useAppStore } from "../stores/appStore";
import type { Severity, ReportIssue } from "../types/ddr";
import { BADGE_VARIANTS } from "../styles/tokens";
import { Spinner } from "./Spinner";

interface Props {
  projectId: number | null;
}

const SEVERITY_ROW: Record<Severity, string> = {
  Error: "bg-red-50 text-red-700",
  Warning: "bg-yellow-50 text-yellow-700",
  Info: "bg-blue-50 text-blue-700",
};

export function ReportCard({ projectId }: Props) {
  const { data: report, isLoading } = useReportCard(projectId);
  const { data: scripts = [] } = useScriptList(projectId);
  const { data: layouts = [] } = useLayoutList(projectId);
  const { selectElement } = useAppStore();

  if (projectId === null) return null;
  if (isLoading) return <div className="flex items-center gap-2 text-gray-400 text-sm"><Spinner className="w-4 h-4" />読み込み中...</div>;
  if (!report) return null;

  const isHealthy = report.error_count === 0 && report.warning_count === 0;

  function handleIssueClick(issue: ReportIssue) {
    if (!projectId || !issue.element_kind || !issue.element_name) return;
    if (issue.element_kind === "script") {
      const script = scripts.find((s) => s.name === issue.element_name);
      if (script) selectElement({ kind: "script", projectId, id: script.id, name: script.name });
    } else if (issue.element_kind === "layout") {
      const layout = layouts.find((l) => l.name === issue.element_name);
      if (layout) selectElement({ kind: "layout", projectId, id: layout.id, name: layout.name });
    }
  }

  return (
    <div className="bg-white rounded-lg border border-gray-200 p-4">
      <div className="flex items-center justify-between mb-3">
        <h3 className="font-semibold text-gray-800">レポートカード</h3>
        {isHealthy ? (
          <span className={BADGE_VARIANTS.green}>健全</span>
        ) : (
          <div className="flex gap-2 text-xs">
            {report.error_count > 0 && (
              <span className={BADGE_VARIANTS.red}>エラー {report.error_count}</span>
            )}
            {report.warning_count > 0 && (
              <span className={BADGE_VARIANTS.yellow}>警告 {report.warning_count}</span>
            )}
            {report.info_count > 0 && (
              <span className={BADGE_VARIANTS.blue}>情報 {report.info_count}</span>
            )}
          </div>
        )}
      </div>
      {report.issues.length > 0 && (
        <ul className="divide-y divide-gray-100 mt-3">
          {report.issues.map((issue, idx) => {
            const isClickable = !!issue.element_kind && !!issue.element_name;
            return isClickable ? (
              <li key={idx} className={SEVERITY_ROW[issue.severity]}>
                <button
                  className="w-full text-left px-3 py-2 text-xs hover:brightness-95"
                  onClick={() => handleIssueClick(issue)}
                  title={`${issue.element_name} を表示`}
                >
                  <span className="font-medium">[{issue.category}]</span> {issue.message}
                </button>
              </li>
            ) : (
              <li
                key={idx}
                className={`text-xs px-3 py-2 ${SEVERITY_ROW[issue.severity]}`}
              >
                <span className="font-medium">[{issue.category}]</span> {issue.message}
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
