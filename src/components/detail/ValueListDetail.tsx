// src/components/detail/ValueListDetail.tsx
import { useValueListItems } from "../../hooks/catalog";
import type { ValueListRow } from "../../types/ddr";
import { Spinner } from "../Spinner";

interface Props {
  valueList: ValueListRow;
}

export function ValueListDetail({ valueList }: Props) {
  const { data: items = [], isLoading } = useValueListItems(valueList.id);

  if (isLoading) {
    return <div className="flex items-center gap-2 p-4 text-gray-500 text-sm"><Spinner className="w-4 h-4" />読み込み中...</div>;
  }

  return (
    <div className="p-4">
      <h2 className="text-lg font-bold mb-4 text-gray-800">{valueList.name}</h2>

      {/* ソース種別 */}
      <div className="mb-4">
        <span className="text-sm font-semibold text-gray-600">ソース種別: </span>
        <span className="inline-block bg-gray-100 text-gray-700 text-sm px-2 py-0.5 rounded">
          {valueList.source}
        </span>
      </div>

      {/* カスタム値一覧 */}
      {valueList.source === "Custom" && (
        <div>
          <h3 className="text-sm font-semibold text-gray-700 mb-2">値一覧</h3>
          {items.length === 0 ? (
            <p className="text-gray-500 text-sm">値なし</p>
          ) : (
            <ul className="border border-gray-200 rounded divide-y divide-gray-200">
              {items.map((item, idx) => (
                <li
                  key={idx}
                  className="px-3 py-2 text-sm text-gray-700 hover:bg-blue-50"
                >
                  {item}
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  );
}
