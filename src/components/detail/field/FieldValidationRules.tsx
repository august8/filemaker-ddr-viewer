import type { FieldRow } from "../../../types/ddr";
import { BADGE_VARIANTS, CODE_BLOCK, SECTION_HEADER } from "../../../styles/tokens";

interface Props {
  field: FieldRow;
}

export function FieldValidationRules({ field }: Props) {
  const hasValidation =
    field.val_not_empty ||
    field.val_unique ||
    field.val_existing ||
    field.val_max_length != null ||
    field.val_value_list ||
    field.val_calc ||
    field.val_range_from ||
    field.val_range_to ||
    field.val_error_message;

  if (!hasValidation) return null;

  return (
    <section>
      <h4 className={SECTION_HEADER}>入力値の制限</h4>
      <dl className="space-y-1">
        {(field.val_not_empty || field.val_unique || field.val_existing) && (
          <div className="flex gap-2 flex-wrap">
            <dt className="text-gray-500 w-24 shrink-0">種別</dt>
            <dd className="flex gap-1 flex-wrap">
              {field.val_not_empty && (
                <span className={BADGE_VARIANTS.red}>入力必須</span>
              )}
              {field.val_unique && (
                <span className={BADGE_VARIANTS.yellow}>重複不可</span>
              )}
              {field.val_existing && (
                <span className={BADGE_VARIANTS.gray}>既存値のみ</span>
              )}
            </dd>
          </div>
        )}
        {field.val_always && (
          <div className="flex gap-2">
            <dt className="text-gray-500 w-24 shrink-0">タイミング</dt>
            <dd>
              <span className={BADGE_VARIANTS.gray}>常に</span>
            </dd>
          </div>
        )}
        {field.val_max_length != null && (
          <div className="flex gap-2">
            <dt className="text-gray-500 w-24 shrink-0">最大文字数</dt>
            <dd className="text-gray-900">{field.val_max_length}</dd>
          </div>
        )}
        {field.val_value_list && (
          <div className="flex gap-2">
            <dt className="text-gray-500 w-24 shrink-0">値一覧</dt>
            <dd className="text-gray-900">{field.val_value_list}</dd>
          </div>
        )}
        {(field.val_range_from != null || field.val_range_to != null) && (
          <div className="flex gap-2">
            <dt className="text-gray-500 w-24 shrink-0">範囲</dt>
            <dd className="text-gray-900">
              {field.val_range_from ?? "?"} 〜 {field.val_range_to ?? "?"}
            </dd>
          </div>
        )}
        {field.val_calc && (
          <div className="flex flex-col gap-1">
            <dt className="text-gray-500">計算式</dt>
            <dd className={CODE_BLOCK}>{field.val_calc}</dd>
          </div>
        )}
        {field.val_error_message && (
          <div className="flex gap-2">
            <dt className="text-gray-500 w-24 shrink-0">エラーメッセージ</dt>
            <dd className="text-gray-700 break-all">{field.val_error_message}</dd>
          </div>
        )}
      </dl>
    </section>
  );
}
