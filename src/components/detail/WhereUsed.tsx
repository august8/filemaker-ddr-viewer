// src/components/detail/WhereUsed.tsx
import { useCallers, useScriptList } from "../../hooks/script";
import { useAppStore } from "../../stores/appStore";

interface Props {
  projectId: number;
  scriptId: number;
}

export function WhereUsed({ projectId, scriptId }: Props) {
  const { data: callerIds = [], isLoading } = useCallers(projectId, scriptId);
  const { data: scripts = [] } = useScriptList(projectId);
  const selectElement = useAppStore((s) => s.selectElement);

  if (isLoading) return null;
  if (callerIds.length === 0) {
    return (
      <div className="px-4 pb-4">
        <h3 className="text-sm font-semibold text-gray-600 mb-1">呼び出し元</h3>
        <p className="text-sm text-gray-400">どこからも呼ばれていません</p>
      </div>
    );
  }

  const callers = scripts.filter((s) => callerIds.includes(s.fm_id));

  return (
    <div className="px-4 pb-4">
      <h3 className="text-sm font-semibold text-gray-600 mb-1">
        呼び出し元 <span className="text-gray-400 font-normal">({callers.length})</span>
      </h3>
      <ul className="space-y-0.5">
        {callers.map((s) => (
          <li key={s.id}>
            <button
              className="w-full text-left text-sm px-2 py-1 rounded hover:bg-blue-50 text-blue-700 hover:text-blue-900"
              onClick={() =>
                selectElement({
                  kind: "script",
                  projectId,
                  id: s.id,
                  name: s.name,
                })
              }
            >
              {s.name}
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
