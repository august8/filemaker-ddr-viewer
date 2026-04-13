// src/components/detail/UpgradeSettingsPanel.tsx
import { useState } from "react";
import { useAppStore, type CheckItem } from "../../stores/appStore";
import { CARD, SECTION_HEADER } from "../../styles/tokens";

const CATEGORY_LABEL: Record<CheckItem["category"], string> = {
  step: "スクリプトステップ",
  function: "関数・計算式",
  field: "フィールド",
  custom_function: "カスタム関数",
};

const DETECTION_TYPE_LABEL: Record<CheckItem["detectionType"], string> = {
  step_type_id: "ステップ ID",
  step_external: "外部参照ステップ ID",
  text_match: "テキスト検索",
  field_attr: "フィールド属性",
  any_custom_function: "全カスタム関数",
};

const EMPTY_CUSTOM: Omit<CheckItem, "id" | "builtin"> = {
  label: "",
  category: "step",
  detectionType: "step_type_id",
  detectionValue: "",
  enabled: true,
};

export function UpgradeSettingsPanel({ onClose }: { onClose: () => void }) {
  const { checkItems, setCheckItems } = useAppStore();
  const [showAddForm, setShowAddForm] = useState(false);
  const [newItem, setNewItem] = useState<Omit<CheckItem, "id" | "builtin">>(EMPTY_CUSTOM);

  function toggleEnabled(id: string) {
    setCheckItems(checkItems.map((i) => (i.id === id ? { ...i, enabled: !i.enabled } : i)));
  }

  function deleteItem(id: string) {
    setCheckItems(checkItems.filter((i) => i.id !== id));
  }

  function resetBuiltins() {
    const BUILTIN_IDS = new Set(checkItems.filter((i) => i.builtin).map((i) => i.id));
    setCheckItems(
      checkItems.map((i) =>
        BUILTIN_IDS.has(i.id) ? { ...i, enabled: true } : i
      )
    );
  }

  function handleAddItem() {
    if (!newItem.label.trim() || !newItem.detectionValue.trim()) return;
    const id = `custom_${Date.now()}`;
    setCheckItems([...checkItems, { ...newItem, id, builtin: false }]);
    setNewItem(EMPTY_CUSTOM);
    setShowAddForm(false);
  }

  const builtins = checkItems.filter((i) => i.builtin);
  const customs = checkItems.filter((i) => !i.builtin);

  return (
    <div className="flex flex-col">
      {/* モーダルヘッダー */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-gray-200 shrink-0">
        <h2 className="text-lg font-bold text-gray-800">アップグレードチェック設定</h2>
        <button
          className="text-gray-400 hover:text-gray-700 text-xl leading-none"
          onClick={onClose}
        >
          ×
        </button>
      </div>
      <div className="overflow-auto p-6">

      {/* ビルトイン項目 */}
      <div className={`${CARD} mb-4`}>
        <div className="flex items-center justify-between mb-3">
          <p className={SECTION_HEADER}>ビルトイン項目</p>
          <button
            className="text-xs text-blue-600 hover:underline"
            onClick={resetBuiltins}
          >
            デフォルトに戻す
          </button>
        </div>
        <div className="divide-y divide-gray-100">
          {builtins.map((item) => (
            <div key={item.id} className="py-2 flex items-center gap-3">
              <input
                type="checkbox"
                checked={item.enabled}
                onChange={() => toggleEnabled(item.id)}
                className="shrink-0"
              />
              <span className="flex-1 text-sm text-gray-700">{item.label}</span>
              <span className="text-xs text-gray-400">
                {CATEGORY_LABEL[item.category]} / {DETECTION_TYPE_LABEL[item.detectionType]}:{" "}
                <code className="font-mono">{item.detectionValue}</code>
              </span>
            </div>
          ))}
        </div>
      </div>

      {/* カスタム項目 */}
      <div className={CARD}>
        <div className="flex items-center justify-between mb-3">
          <p className={SECTION_HEADER}>カスタム項目</p>
          <button
            className="text-xs text-blue-600 hover:underline"
            onClick={() => setShowAddForm(!showAddForm)}
          >
            {showAddForm ? "キャンセル" : "+ カスタム追加"}
          </button>
        </div>

        {showAddForm && (
          <div className="mb-3 p-3 bg-gray-50 border border-gray-200 rounded space-y-2">
            <div className="grid grid-cols-2 gap-2">
              <div>
                <label className="text-xs text-gray-500 block mb-0.5">名称</label>
                <input
                  type="text"
                  className="w-full text-sm border border-gray-300 rounded px-2 py-1"
                  value={newItem.label}
                  onChange={(e) => setNewItem({ ...newItem, label: e.target.value })}
                  placeholder="例: GetNthRecord"
                />
              </div>
              <div>
                <label className="text-xs text-gray-500 block mb-0.5">カテゴリ</label>
                <select
                  className="w-full text-sm border border-gray-300 rounded px-2 py-1"
                  value={newItem.category}
                  onChange={(e) =>
                    setNewItem({ ...newItem, category: e.target.value as CheckItem["category"] })
                  }
                >
                  {(Object.keys(CATEGORY_LABEL) as CheckItem["category"][]).map((k) => (
                    <option key={k} value={k}>{CATEGORY_LABEL[k]}</option>
                  ))}
                </select>
              </div>
              <div>
                <label className="text-xs text-gray-500 block mb-0.5">検出タイプ</label>
                <select
                  className="w-full text-sm border border-gray-300 rounded px-2 py-1"
                  value={newItem.detectionType}
                  onChange={(e) =>
                    setNewItem({ ...newItem, detectionType: e.target.value as CheckItem["detectionType"] })
                  }
                >
                  {(Object.keys(DETECTION_TYPE_LABEL) as CheckItem["detectionType"][]).map((k) => (
                    <option key={k} value={k}>{DETECTION_TYPE_LABEL[k]}</option>
                  ))}
                </select>
              </div>
              <div>
                <label className="text-xs text-gray-500 block mb-0.5">検出値</label>
                <input
                  type="text"
                  className="w-full text-sm border border-gray-300 rounded px-2 py-1"
                  value={newItem.detectionValue}
                  onChange={(e) => setNewItem({ ...newItem, detectionValue: e.target.value })}
                  placeholder="例: GetNthRecord"
                />
              </div>
            </div>
            <div className="flex justify-end">
              <button
                className="px-4 py-1 text-sm rounded border bg-blue-50 border-blue-300 text-blue-700 hover:bg-blue-100 disabled:opacity-40"
                disabled={!newItem.label.trim() || !newItem.detectionValue.trim()}
                onClick={handleAddItem}
              >
                追加
              </button>
            </div>
          </div>
        )}

        {customs.length === 0 ? (
          <p className="text-sm text-gray-400 text-center py-2">カスタム項目なし</p>
        ) : (
          <div className="divide-y divide-gray-100">
            {customs.map((item) => (
              <div key={item.id} className="py-2 flex items-center gap-3">
                <input
                  type="checkbox"
                  checked={item.enabled}
                  onChange={() => toggleEnabled(item.id)}
                  className="shrink-0"
                />
                <span className="flex-1 text-sm text-gray-700">{item.label}</span>
                <span className="text-xs text-gray-400">
                  {CATEGORY_LABEL[item.category]} / {DETECTION_TYPE_LABEL[item.detectionType]}:{" "}
                  <code className="font-mono">{item.detectionValue}</code>
                </span>
                <button
                  className="text-xs text-red-500 hover:text-red-700 shrink-0"
                  onClick={() => deleteItem(item.id)}
                >
                  削除
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
      </div>
    </div>
  );
}
