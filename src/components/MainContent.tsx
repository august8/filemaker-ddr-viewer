import { SearchResults } from "./SearchResults";
import { ProjectSummaryCard } from "./ProjectSummaryCard";
import { ReportCard } from "./ReportCard";
import { BrokenRefsList } from "./BrokenRefsList";
import { OrphanScriptsList } from "./OrphanScriptsList";
import { UnusedFieldsList } from "./UnusedFieldsList";
import { DiffView } from "./DiffView";
import { Spinner } from "./Spinner";
import { TableDetail } from "./detail/TableDetail";
import { ScriptDetail } from "./detail/ScriptDetail";
import { LayoutDetail } from "./detail/LayoutDetail";
import { ValueListDetail } from "./detail/ValueListDetail";
import { CustomFunctionDetail } from "./detail/CustomFunctionDetail";
import { AllFieldsPanel } from "./detail/AllFieldsPanel";
import { AllTablesPanel } from "./detail/AllTablesPanel";
import { AllScriptsPanel } from "./detail/AllScriptsPanel";
import { AllLayoutsPanel } from "./detail/AllLayoutsPanel";
import { AllValueListsPanel } from "./detail/AllValueListsPanel";
import { AllCustomFunctionsPanel } from "./detail/AllCustomFunctionsPanel";
import { AllTableOccurrencesPanel } from "./detail/AllTableOccurrencesPanel";
import { AllRelationshipsPanel } from "./detail/AllRelationshipsPanel";
import { SecurityPanel } from "./detail/SecurityPanel";
import { RelationshipGraphPanel } from "./detail/RelationshipGraphPanel";
import { UpgradeCheckPanel } from "./detail/UpgradeCheckPanel";
import { useAppStore } from "../stores/appStore";
import {
  useScriptList,
  useLayoutList,
  useValueListList,
  useCustomFunctionList,
} from "../hooks/useTauriCommand";

export function MainContent() {
  const { selectedProject, selectedElement, searchQuery } = useAppStore();
  // selectedElement.projectId を優先する（検索結果から別プロジェクトの要素をクリックした場合に正しいプロジェクトを使用するため）。
  // selectedElement が null または projectId を持たない種別（diff 等）の場合は selectedProject にフォールバック。
  const elementProjectId =
    selectedElement && "projectId" in selectedElement ? selectedElement.projectId : null;
  const projectId = elementProjectId ?? selectedProject?.id ?? null;

  // 常時フェッチ: 検索結果クリック時にキャッシュ済みデータで即遷移できるようにする
  const { data: scripts = [], isLoading: scriptsLoading } = useScriptList(projectId);
  const { data: layouts = [], isLoading: layoutsLoading } = useLayoutList(projectId);
  const { data: valueLists = [], isLoading: valueListsLoading } = useValueListList(projectId);
  const { data: customFunctions = [], isLoading: cfLoading } = useCustomFunctionList(projectId);

  // 検索クエリがある場合は selectedElement より優先して検索結果を表示
  if (searchQuery.trim()) {
    return (
      <div className="flex-1 overflow-auto p-4 space-y-4">
        <SearchResults query={searchQuery} />
      </div>
    );
  }

  if (selectedElement) {
    switch (selectedElement.kind) {
      case "all_fields":
        return (
          <div className="flex-1 overflow-hidden flex flex-col">
            <AllFieldsPanel projectId={selectedElement.projectId} />
          </div>
        );
      case "table":
        return (
          <div className="flex-1 overflow-auto">
            <TableDetail
              projectId={selectedElement.projectId}
              tableId={selectedElement.id}
              name={selectedElement.name}
            />
          </div>
        );
      case "script": {
        if (scriptsLoading) return <div className="flex-1 flex items-center justify-center gap-2 text-gray-400 text-sm"><Spinner className="w-4 h-4" />読み込み中...</div>;
        const script = scripts.find((s) => s.id === selectedElement.id);
        if (script) {
          return (
            <div className="flex-1 overflow-auto">
              <ScriptDetail script={script} projectId={selectedElement.projectId} />
            </div>
          );
        }
        return <div className="flex-1 flex items-center justify-center text-gray-400 text-sm">要素が見つかりません (ID: {selectedElement.id})</div>;
      }
      case "layout": {
        if (layoutsLoading) return <div className="flex-1 flex items-center justify-center gap-2 text-gray-400 text-sm"><Spinner className="w-4 h-4" />読み込み中...</div>;
        const layout = layouts.find((l) => l.id === selectedElement.id);
        if (layout) {
          return (
            <div className="flex-1 overflow-auto">
              <LayoutDetail layout={layout} projectId={selectedElement.projectId} />
            </div>
          );
        }
        return <div className="flex-1 flex items-center justify-center text-gray-400 text-sm">要素が見つかりません (ID: {selectedElement.id})</div>;
      }
      case "value_list": {
        if (valueListsLoading) return <div className="flex-1 flex items-center justify-center gap-2 text-gray-400 text-sm"><Spinner className="w-4 h-4" />読み込み中...</div>;
        const valueList = valueLists.find((v) => v.id === selectedElement.id);
        if (valueList) {
          return (
            <div className="flex-1 overflow-auto">
              <ValueListDetail valueList={valueList} />
            </div>
          );
        }
        return <div className="flex-1 flex items-center justify-center text-gray-400 text-sm">要素が見つかりません (ID: {selectedElement.id})</div>;
      }
      case "custom_function": {
        if (cfLoading) return <div className="flex-1 flex items-center justify-center gap-2 text-gray-400 text-sm"><Spinner className="w-4 h-4" />読み込み中...</div>;
        const cf = customFunctions.find((c) => c.id === selectedElement.id);
        if (cf) {
          return (
            <div className="flex-1 overflow-auto">
              <CustomFunctionDetail customFunction={cf} />
            </div>
          );
        }
        return <div className="flex-1 flex items-center justify-center text-gray-400 text-sm">要素が見つかりません (ID: {selectedElement.id})</div>;
      }
      case "all_tables":
        return (
          <div className="flex-1 overflow-hidden flex flex-col">
            <AllTablesPanel projectId={selectedElement.projectId} />
          </div>
        );
      case "all_scripts":
        return (
          <div className="flex-1 overflow-hidden flex flex-col">
            <AllScriptsPanel projectId={selectedElement.projectId} />
          </div>
        );
      case "all_layouts":
        return (
          <div className="flex-1 overflow-hidden flex flex-col">
            <AllLayoutsPanel projectId={selectedElement.projectId} />
          </div>
        );
      case "all_value_lists":
        return (
          <div className="flex-1 overflow-hidden flex flex-col">
            <AllValueListsPanel projectId={selectedElement.projectId} />
          </div>
        );
      case "all_custom_functions":
        return (
          <div className="flex-1 overflow-hidden flex flex-col">
            <AllCustomFunctionsPanel projectId={selectedElement.projectId} />
          </div>
        );
      case "all_table_occurrences":
        return (
          <div className="flex-1 overflow-hidden flex flex-col">
            <AllTableOccurrencesPanel projectId={selectedElement.projectId} />
          </div>
        );
      case "all_relationships":
        return (
          <div className="flex-1 overflow-hidden flex flex-col">
            <AllRelationshipsPanel projectId={selectedElement.projectId} highlightId={selectedElement.highlightId} />
          </div>
        );
      case "diff":
        return <DiffView />;
      case "security":
        return (
          <div className="flex-1 overflow-auto">
            <SecurityPanel projectId={selectedElement.projectId} />
          </div>
        );
      case "relationship_graph":
        return (
          <div className="flex-1 overflow-hidden flex flex-col">
            <RelationshipGraphPanel projectId={selectedElement.projectId} />
          </div>
        );
      case "upgrade_check":
        return <UpgradeCheckPanel solutionId={selectedElement.solutionId} />;
      case "dashboard":
        break; // ダッシュボード表示 → switch を抜けて下のダッシュボード return へ
    }
  }

  return (
    <div className="flex-1 overflow-auto p-4 space-y-4">
      <ProjectSummaryCard projectId={projectId} />
      <ReportCard projectId={projectId} />
      <BrokenRefsList projectId={projectId} />
      <OrphanScriptsList projectId={projectId} />
      <UnusedFieldsList projectId={projectId} />
    </div>
  );
}
