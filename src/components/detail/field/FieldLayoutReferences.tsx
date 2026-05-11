import { useFieldLayoutRefs, useLayoutRefDebugInfo } from "../../../hooks/fieldRefs";
import { useAppStore } from "../../../stores/appStore";
import { Spinner } from "../../Spinner";
import { SECTION_HEADER } from "../../../styles/tokens";

interface Props {
  projectId: number;
  tableName: string;
  fieldName: string;
}

export function FieldLayoutReferences({ projectId, tableName, fieldName }: Props) {
  const { selectElement } = useAppStore();
  const { data: layoutRefs = [], isLoading } = useFieldLayoutRefs(
    projectId,
    tableName,
    fieldName
  );
  const { data: debugInfo } = useLayoutRefDebugInfo(projectId);

  return (
    <section>
      <h4 className={SECTION_HEADER}>レイアウト使用箇所</h4>
      {isLoading ? (
        <span className="flex items-center gap-1 text-gray-400 text-xs">
          <Spinner className="w-3 h-3" />
          読み込み中...
        </span>
      ) : layoutRefs.length === 0 ? (
        <div className="space-y-1">
          <p className="text-gray-400 text-xs">
            このフィールドが配置されているレイアウトはありません
          </p>
          {debugInfo &&
            (debugInfo.occurrence_count === 0 ||
              debugInfo.layout_field_ref_count === 0) && (
              <div className="bg-amber-50 border border-amber-200 rounded p-2 text-xs text-amber-700">
                <p className="font-semibold mb-1">⚠ DDRの再インポートが必要です</p>
                <p>オカレンスデータ: {debugInfo.occurrence_count}件</p>
                <p>レイアウトフィールドデータ: {debugInfo.layout_field_ref_count}件</p>
                <p className="mt-1 text-amber-600">
                  スキーマ更新後に初めてインポートしたDDRでのみ有効です。既存のDDRは削除して再インポートしてください。
                </p>
              </div>
            )}
        </div>
      ) : (
        <ul className="space-y-1">
          {layoutRefs.map((ref) => (
            <li key={ref.layout_id}>
              <button
                className="text-blue-600 hover:underline text-left break-all w-full"
                onClick={() =>
                  selectElement({
                    kind: "layout",
                    projectId: ref.project_id,
                    id: ref.layout_id,
                    name: ref.layout_name,
                  })
                }
              >
                {ref.layout_name}
              </button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
