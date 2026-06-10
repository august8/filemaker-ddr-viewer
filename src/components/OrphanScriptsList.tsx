import { useOrphanScripts } from "../hooks/analysis";
import { useScriptList } from "../hooks/script";
import { useAppStore } from "../stores/appStore";
import { BADGE_VARIANTS, CARD } from "../styles/tokens";
import { Spinner } from "./Spinner";

interface Props {
  projectId: number | null;
}

export function OrphanScriptsList({ projectId }: Props) {
  const { data: orphans, isLoading } = useOrphanScripts(projectId);
  const { data: scripts = [] } = useScriptList(projectId);
  const selectElement = useAppStore((s) => s.selectElement);

  if (projectId === null) return null;
  if (isLoading) return <div className="flex items-center gap-2 text-gray-400 text-sm"><Spinner className="w-4 h-4" />読み込み中...</div>;

  if (!orphans || orphans.length === 0) {
    return (
      <div className={CARD}>
        <h3 className="font-semibold text-gray-800 mb-2">未使用スクリプト</h3>
        <div className="text-sm text-green-600">未使用スクリプトはありません</div>
      </div>
    );
  }

  return (
    <div className={CARD}>
      <h3 className="font-semibold text-gray-800 mb-3">
        未使用スクリプト
        <span className={`ml-2 ${BADGE_VARIANTS.yellow}`}>{orphans.length}</span>
      </h3>
      <ul className="divide-y divide-gray-100">
        {orphans.map((orphan) => {
          const script = scripts.find((s) => s.fm_id === orphan.script_id);
          return (
            <li key={orphan.script_id}>
              {script ? (
                <button
                  className="w-full text-left py-2 text-sm text-blue-700 hover:bg-blue-50 px-2 rounded"
                  onClick={() =>
                    selectElement({
                      kind: "script",
                      projectId,
                      id: script.id,
                      name: script.name,
                    })
                  }
                >
                  {orphan.script_name}
                </button>
              ) : (
                <span className="block py-2 text-sm text-gray-700 px-2">
                  {orphan.script_name}
                </span>
              )}
            </li>
          );
        })}
      </ul>
    </div>
  );
}
