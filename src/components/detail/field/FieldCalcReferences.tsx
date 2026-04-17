import { useFieldCalcRefs } from "../../../hooks/useTauriCommand";
import { useAppStore } from "../../../stores/appStore";
import { Spinner } from "../../Spinner";
import { SECTION_HEADER } from "../../../styles/tokens";

interface Props {
  projectId: number;
  tableName: string;
  fieldName: string;
}

export function FieldCalcReferences({ projectId, tableName, fieldName }: Props) {
  const { setRightPanel } = useAppStore();
  const { data: calcRefs = [], isLoading } = useFieldCalcRefs(
    projectId,
    tableName,
    fieldName
  );

  return (
    <section>
      <h4 className={SECTION_HEADER}>計算フィールドとして使用されている箇所</h4>
      {isLoading ? (
        <span className="flex items-center gap-1 text-gray-400 text-xs">
          <Spinner className="w-3 h-3" />
          読み込み中...
        </span>
      ) : calcRefs.length === 0 ? (
        <p className="text-gray-400 text-xs">他のフィールドの計算式で参照されていません</p>
      ) : (
        <ul className="space-y-1">
          {calcRefs.map((ref) => (
            <li key={ref.field_id}>
              <button
                className="text-blue-600 hover:underline text-left break-all w-full"
                onClick={() =>
                  setRightPanel({
                    kind: "field",
                    projectId,
                    tableId: ref.table_id,
                    fieldId: ref.field_id,
                    tableName: ref.table_name,
                  })
                }
              >
                {ref.table_name}::{ref.field_name}
              </button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
