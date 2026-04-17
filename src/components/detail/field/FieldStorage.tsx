import type { FieldRow } from "../../../types/ddr";
import { BADGE_VARIANTS, SECTION_HEADER } from "../../../styles/tokens";

const INDEX_LABEL: Record<string, string> = {
  All:     "全て",
  Minimal: "最小",
  None:    "なし",
};

interface Props {
  field: FieldRow;
}

export function FieldStorage({ field }: Props) {
  if (!field.index_type && !field.container_storage) return null;

  return (
    <section>
      <h4 className={SECTION_HEADER}>ストレージ</h4>
      <dl className="space-y-1">
        {field.container_storage && (
          <div className="flex gap-2">
            <dt className="text-gray-500 w-24 shrink-0">保存方法</dt>
            <dd>
              <span className={BADGE_VARIANTS.purple}>
                {field.container_storage === "Internal" && "内部格納"}
                {field.container_storage === "Secure" && "外部格納（セキュア）"}
                {field.container_storage === "Open" && "外部格納（オープン）"}
              </span>
            </dd>
          </div>
        )}
        {field.index_type && (
          <div className="flex gap-2">
            <dt className="text-gray-500 w-24 shrink-0">インデックス</dt>
            <dd>
              <span className={BADGE_VARIANTS.gray}>
                {INDEX_LABEL[field.index_type] ?? field.index_type}
              </span>
            </dd>
          </div>
        )}
      </dl>
    </section>
  );
}
