// src/styles/tokens.ts
// UI スタイルトークン — 全コンポーネント共通のクラス定数

/** バッジの基底クラス（色クラスを別途追加する） */
export const BADGE_BASE = "inline-block text-xs font-medium rounded px-2 py-0.5";

/** 色付きバッジ（情報バッジ・ステータス表示など） */
export const BADGE_VARIANTS = {
  blue:   `${BADGE_BASE} bg-blue-100 text-blue-700`,
  purple: `${BADGE_BASE} bg-purple-100 text-purple-700`,
  green:  `${BADGE_BASE} bg-green-100 text-green-700`,
  yellow: `${BADGE_BASE} bg-yellow-100 text-yellow-700`,
  red:    `${BADGE_BASE} bg-red-100 text-red-700`,
  gray:   `${BADGE_BASE} bg-gray-100 text-gray-700`,
} as const;

/** DiffView 専用バッジ（border あり、Added/Removed/Modified） */
export const DIFF_BADGE_VARIANTS = {
  Added:    "bg-green-100 text-green-700 border-green-200",
  Removed:  "bg-red-100 text-red-600 border-red-200",
  Modified: "bg-yellow-100 text-yellow-700 border-yellow-200",
} as const;

/** セクション小見出し（h3/h4 レベル） */
export const SECTION_HEADER = "text-sm font-semibold text-gray-700 mb-2";

/** パネルメイン見出し（h2 レベル） */
export const MAIN_HEADER = "text-lg font-bold text-gray-800 mb-4";

/** コード・計算式・CSS などの等幅テキスト表示領域 */
export const CODE_BLOCK =
  "bg-gray-50 border border-gray-200 rounded p-2 font-mono text-xs whitespace-pre-wrap break-all";

/** カードコンテナ（白背景・グレーボーダー・角丸・パディング） */
export const CARD = "bg-white rounded-xl border border-gray-200 p-4";

/** クリック可能なリスト行ボタン */
export const LIST_ROW = "w-full text-left px-3 py-2 text-sm hover:bg-blue-50 transition-colors";

/** セレクト・インプット標準スタイル */
export const SELECT_INPUT =
  "border border-gray-300 rounded px-3 py-1.5 text-gray-700 bg-white focus:outline-none focus:ring-1 focus:ring-blue-400";

/** All*Panel 系フィルタ入力（flex-1 + text-sm 込み） */
export const FILTER_INPUT =
  "flex-1 px-3 py-1.5 text-sm border border-gray-300 rounded text-gray-700 bg-white focus:outline-none focus:ring-1 focus:ring-blue-400";

/** ページネーションボタン（前ページ・次ページ共通） */
export const PAGINATION_BUTTON =
  "px-2 py-1.5 text-sm border border-gray-300 rounded hover:bg-gray-100 disabled:opacity-40 disabled:cursor-not-allowed";
