// src/components/detail/FieldDetail.tsx
import type { FieldRow } from "../../types/ddr";
import { Spinner } from "../Spinner";

const AUTO_ENTER_LABEL: Record<string, string> = {
  Calculation:              "計算式",
  ConstantData:             "定数",
  Serial:                   "シリアル番号",
  Lookup:                   "参照",
  ModificationTimeStamp:    "更新タイムスタンプ",
  ModificationDate:         "更新日",
  ModificationTime:         "更新時刻",
  ModificationAccountName:  "更新アカウント名",
  CreationTimeStamp:        "作成タイムスタンプ",
  CreationDate:             "作成日",
  CreationTime:             "作成時刻",
  CreationAccountName:      "作成アカウント名",
};

const INDEX_LABEL: Record<string, string> = {
  All:     "全て",
  Minimal: "最小",
  None:    "なし",
};

function parseSerialInfo(calc: string): string {
  const kv: Record<string, string> = {};
  for (const token of calc.split(",")) {
    const eq = token.indexOf("=");
    if (eq >= 0) kv[token.slice(0, eq).trim()] = token.slice(eq + 1).trim();
  }
  const gen = kv["generate"] === "OnCreation" ? "作成時"
            : kv["generate"] === "OnCommit"   ? "確定時"
            : (kv["generate"] ?? "");
  return `次の値: ${kv["nextValue"] ?? "?"} / 増分: ${kv["increment"] ?? "?"} / 生成: ${gen}`;
}
import { useFieldRefs, useFieldCalcRefs, useFieldLayoutRefs, useFieldRelationshipKeys, useLayoutRefDebugInfo, useScriptList, useLayoutList } from "../../hooks/useTauriCommand";
import { useAppStore } from "../../stores/appStore";
import { BADGE_VARIANTS, CODE_BLOCK, SECTION_HEADER } from "../../styles/tokens";

interface Props {
  field: FieldRow;
  tableName: string;
  projectId: number;
}

export function FieldDetail({ field, tableName, projectId }: Props) {
  const { selectElement, setRightPanel } = useAppStore();
  const { data: refs = [], isLoading: refsLoading } = useFieldRefs(
    projectId,
    tableName,
    field.name
  );
  const { data: layoutRefs = [], isLoading: layoutRefsLoading } = useFieldLayoutRefs(
    projectId,
    tableName,
    field.name
  );
  const { data: relKeyRefs = [], isLoading: relKeyLoading } = useFieldRelationshipKeys(
    projectId,
    tableName,
    field.name
  );
  const { data: calcRefs = [], isLoading: calcRefsLoading } = useFieldCalcRefs(
    projectId,
    tableName,
    field.name
  );
  const { data: debugInfo } = useLayoutRefDebugInfo(projectId);
  const { data: scripts = [] } = useScriptList(projectId);
  const { data: layouts = [] } = useLayoutList(projectId);

  function handleScriptClick(scriptName: string) {
    const script = scripts.find((s) => s.name === scriptName);
    if (script) {
      selectElement({ kind: "script", projectId, id: script.id, name: script.name });
    }
  }

  function handleLayoutClick(layoutName: string) {
    const layout = layouts.find((l) => l.name === layoutName);
    if (layout) {
      selectElement({ kind: "layout", projectId, id: layout.id, name: layout.name });
    }
  }

  function handleCalcFieldClick(refTableId: number, refFieldId: number, refTableName: string) {
    setRightPanel({ kind: "field", projectId, tableId: refTableId, fieldId: refFieldId, tableName: refTableName });
  }

  return (
    <div className="p-4 space-y-4 text-sm">
      {/* フィールドプロパティ */}
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

      {/* 自動入力 */}
      {field.auto_enter_type !== "" && (
        <section>
          <h4 className={SECTION_HEADER}>自動入力</h4>
          <dl className="space-y-1">
            <div className="flex gap-2">
              <dt className="text-gray-500 w-24 shrink-0">種別</dt>
              <dd>
                <span className={BADGE_VARIANTS.blue}>{AUTO_ENTER_LABEL[field.auto_enter_type] ?? field.auto_enter_type}</span>
              </dd>
            </div>
            {!field.auto_enter_allow_editing && (
              <div className="flex gap-2">
                <dt className="text-gray-500 w-24 shrink-0">編集</dt>
                <dd><span className={BADGE_VARIANTS.gray}>編集不可</span></dd>
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
      )}

      {/* 入力値の制限 */}
      {(field.val_not_empty || field.val_unique || field.val_existing ||
        field.val_max_length != null || field.val_value_list ||
        field.val_calc || field.val_range_from || field.val_range_to ||
        field.val_error_message) && (
        <section>
          <h4 className={SECTION_HEADER}>入力値の制限</h4>
          <dl className="space-y-1">
            {(field.val_not_empty || field.val_unique || field.val_existing) && (
              <div className="flex gap-2 flex-wrap">
                <dt className="text-gray-500 w-24 shrink-0">種別</dt>
                <dd className="flex gap-1 flex-wrap">
                  {field.val_not_empty && <span className={BADGE_VARIANTS.red}>入力必須</span>}
                  {field.val_unique && <span className={BADGE_VARIANTS.yellow}>重複不可</span>}
                  {field.val_existing && <span className={BADGE_VARIANTS.gray}>既存値のみ</span>}
                </dd>
              </div>
            )}
            {field.val_always && (
              <div className="flex gap-2">
                <dt className="text-gray-500 w-24 shrink-0">タイミング</dt>
                <dd><span className={BADGE_VARIANTS.gray}>常に</span></dd>
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
      )}

      {/* ストレージ */}
      {(field.index_type || field.container_storage) && (
        <section>
          <h4 className={SECTION_HEADER}>ストレージ</h4>
          <dl className="space-y-1">
            {field.container_storage && (
              <div className="flex gap-2">
                <dt className="text-gray-500 w-24 shrink-0">保存方法</dt>
                <dd>
                  <span className={BADGE_VARIANTS.purple}>
                    {field.container_storage === "Internal" && "内部格納"}
                    {field.container_storage === "Secure"   && "外部格納（セキュア）"}
                    {field.container_storage === "Open"     && "外部格納（オープン）"}
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
      )}

      {/* 計算フィールドとして使用されている箇所 */}
      <section>
        <h4 className={SECTION_HEADER}>計算フィールドとして使用されている箇所</h4>
        {calcRefsLoading ? (
          <span className="flex items-center gap-1 text-gray-400 text-xs"><Spinner className="w-3 h-3" />読み込み中...</span>
        ) : calcRefs.length === 0 ? (
          <p className="text-gray-400 text-xs">他のフィールドの計算式で参照されていません</p>
        ) : (
          <ul className="space-y-1">
            {calcRefs.map((ref) => (
              <li key={ref.field_id}>
                <button
                  className="text-blue-600 hover:underline text-left break-all w-full"
                  onClick={() => handleCalcFieldClick(ref.table_id, ref.field_id, ref.table_name)}
                >
                  {ref.table_name}::{ref.field_name}
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>

      {/* スクリプト使用箇所 */}
      <section>
        <h4 className={SECTION_HEADER}>スクリプト使用箇所</h4>
        {refsLoading ? (
          <span className="flex items-center gap-1 text-gray-400 text-xs"><Spinner className="w-3 h-3" />読み込み中...</span>
        ) : refs.length === 0 ? (
          <p className="text-gray-400 text-xs">このフィールドを参照するスクリプトはありません</p>
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

      {/* レイアウト使用箇所 */}
      <section>
        <h4 className={SECTION_HEADER}>レイアウト使用箇所</h4>
        {layoutRefsLoading ? (
          <span className="flex items-center gap-1 text-gray-400 text-xs"><Spinner className="w-3 h-3" />読み込み中...</span>
        ) : layoutRefs.length === 0 ? (
          <div className="space-y-1">
            <p className="text-gray-400 text-xs">このフィールドが配置されているレイアウトはありません</p>
            {debugInfo && (debugInfo.occurrence_count === 0 || debugInfo.layout_field_ref_count === 0) && (
              <div className="bg-amber-50 border border-amber-200 rounded p-2 text-xs text-amber-700">
                <p className="font-semibold mb-1">⚠ DDRの再インポートが必要です</p>
                <p>オカレンスデータ: {debugInfo.occurrence_count}件</p>
                <p>レイアウトフィールドデータ: {debugInfo.layout_field_ref_count}件</p>
                <p className="mt-1 text-amber-600">スキーマ更新後に初めてインポートしたDDRでのみ有効です。既存のDDRは削除して再インポートしてください。</p>
              </div>
            )}
          </div>
        ) : (
          <ul className="space-y-1">
            {layoutRefs.map((ref) => (
              <li key={ref.layout_id}>
                <button
                  className="text-blue-600 hover:underline text-left break-all w-full"
                  onClick={() => handleLayoutClick(ref.layout_name)}
                >
                  {ref.layout_name}
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>
      {/* リレーションキー使用箇所 */}
      <section>
        <h4 className={SECTION_HEADER}>リレーションキー使用箇所</h4>
        {relKeyLoading ? (
          <span className="flex items-center gap-1 text-gray-400 text-xs"><Spinner className="w-3 h-3" />読み込み中...</span>
        ) : relKeyRefs.length === 0 ? (
          <p className="text-gray-400 text-xs">リレーションキーとして使用されていません</p>
        ) : (
          <ul className="space-y-1">
            {relKeyRefs.map((ref) => (
              <li key={`${ref.relationship_id}-${ref.side}`}>
                <button
                  className="w-full flex items-center gap-2 text-xs hover:bg-blue-50 rounded px-1 py-0.5 transition-colors text-left"
                  onClick={() => selectElement({ kind: "all_relationships", projectId, highlightId: ref.relationship_id })}
                >
                  <span className={ref.side === "left" ? BADGE_VARIANTS.blue : BADGE_VARIANTS.green}>
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
    </div>
  );
}
