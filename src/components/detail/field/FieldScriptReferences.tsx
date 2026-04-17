import { useFieldRefs, useScriptList } from "../../../hooks/useTauriCommand";
import { useAppStore } from "../../../stores/appStore";
import { Spinner } from "../../Spinner";
import { SECTION_HEADER } from "../../../styles/tokens";

interface Props {
  projectId: number;
  tableName: string;
  fieldName: string;
}

export function FieldScriptReferences({ projectId, tableName, fieldName }: Props) {
  const { selectElement } = useAppStore();
  const { data: refs = [], isLoading } = useFieldRefs(projectId, tableName, fieldName);
  const { data: scripts = [] } = useScriptList(projectId);

  function handleScriptClick(scriptName: string) {
    const script = scripts.find((s) => s.name === scriptName);
    if (script) {
      selectElement({ kind: "script", projectId, id: script.id, name: script.name });
    }
  }

  return (
    <section>
      <h4 className={SECTION_HEADER}>スクリプト使用箇所</h4>
      {isLoading ? (
        <span className="flex items-center gap-1 text-gray-400 text-xs">
          <Spinner className="w-3 h-3" />
          読み込み中...
        </span>
      ) : refs.length === 0 ? (
        <p className="text-gray-400 text-xs">
          このフィールドを参照するスクリプトはありません
        </p>
      ) : (
        <ul className="space-y-1">
          {refs.map((ref) => (
            <li key={ref.script_id}>
              <button
                className="text-blue-600 hover:underline text-left break-all w-full"
                onClick={() => handleScriptClick(ref.script_name)}
              >
                {ref.script_name}
              </button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
