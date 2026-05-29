import { useBrokenRefs } from "../hooks/analysis";
import { useAppStore } from "../stores/appStore";
import type { BrokenRef } from "../types/ddr";
import { BADGE_VARIANTS, LIST_ROW } from "../styles/tokens";
import { Spinner } from "./Spinner";

interface Props {
  projectId: number | null;
}

const KIND_LABELS: Record<string, string> = {
  performScript: "Perform Script",
  scriptTrigger: "Script Trigger",
  brokenFieldRef: "壊れたフィールド参照",
  brokenLayoutRef: "壊れたレイアウト参照",
  unknownRef: "参照先不明",
};

export function BrokenRefsList({ projectId }: Props) {
  const { data: brokenRefs, isLoading } = useBrokenRefs(projectId);
  const { selectElement } = useAppStore();

  if (projectId === null) return null;
  if (isLoading) return <div className="flex items-center gap-2 text-gray-400 text-sm"><Spinner className="w-4 h-4" />読み込み中...</div>;

  if (!brokenRefs || brokenRefs.length === 0) {
    return (
      <div className="bg-white rounded-lg border border-gray-200 p-4">
        <h3 className="font-semibold text-gray-800 mb-2">壊れた参照</h3>
        <div className="text-sm text-green-600">壊れた参照はありません</div>
      </div>
    );
  }

  function handleClick(ref: BrokenRef) {
    if (!projectId || ref.source_id == null) return;
    selectElement({
      kind: ref.kind === "scriptTrigger" ? "layout" : "script",
      projectId,
      id: ref.source_id,
      name: ref.source_name,
    });
  }

  return (
    <div className="bg-white rounded-lg border border-gray-200 p-4">
      <h3 className="font-semibold text-gray-800 mb-3">
        壊れた参照
        <span className={`ml-2 ${BADGE_VARIANTS.red}`}>{brokenRefs.length}</span>
      </h3>
      <ul className="divide-y divide-gray-100">
        {brokenRefs.map((ref, idx) => (
          <li key={idx}>
            <button
              className={`${LIST_ROW} rounded`}
              onClick={() => handleClick(ref)}
              title={`${ref.source_name} を表示`}
            >
              <span className={`mr-2 ${BADGE_VARIANTS.gray}`}>{KIND_LABELS[ref.kind] ?? ref.kind}</span>
              <span className="text-gray-700">{ref.source_name}</span>
              <span className="mx-2 text-gray-400">→</span>
              <span className="text-red-600">{ref.target_script_name}</span>
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
