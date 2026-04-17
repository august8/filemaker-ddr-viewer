import type { FieldRow } from "../../../types/ddr";
import { BADGE_VARIANTS, CODE_BLOCK, SECTION_HEADER } from "../../../styles/tokens";

interface Props {
  field: FieldRow;
}

export function FieldBasicProperties({ field }: Props) {
  return (
    <section>
      <h4 className={SECTION_HEADER}>フィールドプロパティ</h4>
      <dl className="space-y-1">
        <div className="flex gap-2">
          <dt className="text-gray-500 w-24 shrink-0">名前</dt>
          <dd className="font-medium text-gray-900 break-all">{field.name}</dd>
        </div>
        <div className="flex gap-2">
          <dt className="text-gray-500 w-24 shrink-0">データ型</dt>
          <dd>
            <span className={BADGE_VARIANTS.purple}>{field.data_type}</span>
          </dd>
        </div>
        <div className="flex gap-2">
          <dt className="text-gray-500 w-24 shrink-0">種別</dt>
          <dd>
            <span className={BADGE_VARIANTS.gray}>{field.field_type}</span>
          </dd>
        </div>
        {field.is_global && (
          <div className="flex gap-2">
            <dt className="text-gray-500 w-24 shrink-0">グローバル</dt>
            <dd>
              <span className={BADGE_VARIANTS.blue}>Global</span>
            </dd>
          </div>
        )}
        {field.max_repeat > 1 && (
          <div className="flex gap-2">
            <dt className="text-gray-500 w-24 shrink-0">繰り返し</dt>
            <dd className="text-gray-900">{field.max_repeat}</dd>
          </div>
        )}
        {field.comment && (
          <div className="flex gap-2">
            <dt className="text-gray-500 w-24 shrink-0">コメント</dt>
            <dd className="text-gray-700 break-all">{field.comment}</dd>
          </div>
        )}
        {field.calculation && (
          <div className="flex flex-col gap-1">
            <dt className="text-gray-500">計算式</dt>
            <dd className={CODE_BLOCK}>{field.calculation}</dd>
          </div>
        )}
      </dl>
    </section>
  );
}
