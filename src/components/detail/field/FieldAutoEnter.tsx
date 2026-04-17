import type { FieldRow } from "../../../types/ddr";
import { BADGE_VARIANTS, CODE_BLOCK, SECTION_HEADER } from "../../../styles/tokens";

const AUTO_ENTER_LABEL: Record<string, string> = {
  Calculation:             "計算式",
  ConstantData:            "定数",
  Serial:                  "シリアル番号",
  Lookup:                  "参照",
  ModificationTimeStamp:   "更新タイムスタンプ",
  ModificationDate:        "更新日",
  ModificationTime:        "更新時刻",
  ModificationAccountName: "更新アカウント名",
  CreationTimeStamp:       "作成タイムスタンプ",
  CreationDate:            "作成日",
  CreationTime:            "作成時刻",
  CreationAccountName:     "作成アカウント名",
};

function parseSerialInfo(calc: string): string {
  const kv: Record<string, string> = {};
  for (const token of calc.split(",")) {
    const eq = token.indexOf("=");
    if (eq >= 0) kv[token.slice(0, eq).trim()] = token.slice(eq + 1).trim();
  }
  const gen =
    kv["generate"] === "OnCreation"
      ? "作成時"
      : kv["generate"] === "OnCommit"
        ? "確定時"
        : (kv["generate"] ?? "");
  return `次の値: ${kv["nextValue"] ?? "?"} / 増分: ${kv["increment"] ?? "?"} / 生成: ${gen}`;
}

interface Props {
  field: FieldRow;
}

export function FieldAutoEnter({ field }: Props) {
  if (field.auto_enter_type === "") return null;

  return (
    <section>
      <h4 className={SECTION_HEADER}>自動入力</h4>
      <dl className="space-y-1">
        <div className="flex gap-2">
          <dt className="text-gray-500 w-24 shrink-0">種別</dt>
          <dd>
            <span className={BADGE_VARIANTS.blue}>
              {AUTO_ENTER_LABEL[field.auto_enter_type] ?? field.auto_enter_type}
            </span>
          </dd>
        </div>
        {!field.auto_enter_allow_editing && (
          <div className="flex gap-2">
            <dt className="text-gray-500 w-24 shrink-0">編集</dt>
            <dd>
              <span className={BADGE_VARIANTS.gray}>編集不可</span>
            </dd>
          </div>
        )}
        {field.auto_enter_calc && (
          <div className="flex flex-col gap-1">
            <dt className="text-gray-500">
              {field.auto_enter_type === "Serial" ? "シリアル情報" : "計算式 / 値"}
            </dt>
            <dd className={CODE_BLOCK}>
              {field.auto_enter_type === "Serial"
                ? parseSerialInfo(field.auto_enter_calc)
                : field.auto_enter_calc}
            </dd>
          </div>
        )}
      </dl>
    </section>
  );
}
