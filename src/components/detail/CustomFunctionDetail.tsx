// src/components/detail/CustomFunctionDetail.tsx
import type { CustomFunctionRow } from "../../types/ddr";
import { CODE_BLOCK, SECTION_HEADER } from "../../styles/tokens";

interface Props {
  customFunction: CustomFunctionRow;
}

export function CustomFunctionDetail({ customFunction }: Props) {
  return (
    <div className="p-4">
      <h2 className="text-lg font-bold mb-4 text-gray-800">
        {customFunction.name}
      </h2>

      {/* パラメータ */}
      <div className="mb-4">
        <h3 className={SECTION_HEADER}>パラメータ</h3>
        {customFunction.parameters ? (
          <p className={`${CODE_BLOCK} text-sm`}>{customFunction.parameters}</p>
        ) : (
          <p className="text-gray-500 text-sm">パラメータなし</p>
        )}
      </div>

      {/* 計算式 */}
      <div>
        <h3 className={SECTION_HEADER}>計算式</h3>
        {customFunction.calculation ? (
          <pre className={`${CODE_BLOCK} text-sm overflow-auto`}>
            {customFunction.calculation}
          </pre>
        ) : (
          <p className="text-gray-500 text-sm">計算式なし</p>
        )}
      </div>
    </div>
  );
}
