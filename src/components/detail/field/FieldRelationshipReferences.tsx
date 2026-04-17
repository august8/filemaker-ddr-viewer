import { useFieldRelationshipKeys } from "../../../hooks/useTauriCommand";
import { useAppStore } from "../../../stores/appStore";
import { Spinner } from "../../Spinner";
import { BADGE_VARIANTS, SECTION_HEADER } from "../../../styles/tokens";

interface Props {
  projectId: number;
  tableName: string;
  fieldName: string;
}

export function FieldRelationshipReferences({ projectId, tableName, fieldName }: Props) {
  const { selectElement } = useAppStore();
  const { data: relKeyRefs = [], isLoading } = useFieldRelationshipKeys(
    projectId,
    tableName,
    fieldName
  );

  return (
    <section>
      <h4 className={SECTION_HEADER}>リレーションキー使用箇所</h4>
      {isLoading ? (
        <span className="flex items-center gap-1 text-gray-400 text-xs">
          <Spinner className="w-3 h-3" />
          読み込み中...
        </span>
      ) : relKeyRefs.length === 0 ? (
        <p className="text-gray-400 text-xs">リレーションキーとして使用されていません</p>
      ) : (
        <ul className="space-y-1">
          {relKeyRefs.map((ref) => (
            <li key={`${ref.relationship_id}-${ref.side}`}>
              <button
                className="w-full flex items-center gap-2 text-xs hover:bg-blue-50 rounded px-1 py-0.5 transition-colors text-left"
                onClick={() =>
                  selectElement({
                    kind: "all_relationships",
                    projectId,
                    highlightId: ref.relationship_id,
                  })
                }
              >
                <span
                  className={
                    ref.side === "left" ? BADGE_VARIANTS.blue : BADGE_VARIANTS.green
                  }
                >
                  {ref.side === "left" ? "左" : "右"}
                </span>
                <span className="font-medium text-gray-800">{ref.relationship_name}</span>
                <span className="text-gray-400">
                  {ref.left_table} {ref.operator} {ref.right_table}
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
