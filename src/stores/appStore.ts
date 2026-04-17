// src/stores/appStore.ts
import { create } from "zustand";
import type { SolutionRow, ProjectRow } from "../types/ddr";

// ---------------------------------------------------------------------------
// アップグレードチェック設定
// ---------------------------------------------------------------------------

export interface CheckItem {
  id: string;
  label: string;
  category: "step" | "function" | "field" | "custom_function";
  detectionType: "step_type_id" | "step_external" | "text_match" | "field_attr" | "any_custom_function";
  detectionValue: string;
  enabled: boolean;
  builtin: boolean;
}

const CHECK_ITEMS_KEY = "fm-ddr-check-items";

const BUILTIN_CHECK_ITEMS: CheckItem[] = [
  { id: "print",              label: "印刷",                       category: "step",            detectionType: "step_type_id",      detectionValue: "43",                enabled: true,  builtin: true },
  { id: "print_setup",        label: "印刷設定",                   category: "step",            detectionType: "step_type_id",      detectionValue: "42",                enabled: true,  builtin: true },
  { id: "ext_script",         label: "外部スクリプト実行",          category: "step",            detectionType: "step_external",     detectionValue: "1",                 enabled: true,  builtin: true },
  { id: "execute_sql",        label: "ExecuteSQL",                 category: "function",        detectionType: "text_match",        detectionValue: "ExecuteSQL",        enabled: true,  builtin: true },
  { id: "evaluate",           label: "Evaluate",                   category: "function",        detectionType: "text_match",        detectionValue: "Evaluate",          enabled: true,  builtin: true },
  { id: "file_path",          label: "file:// パス",               category: "function",        detectionType: "text_match",        detectionValue: "file://",           enabled: true,  builtin: true },
  { id: "middle_values",      label: "MiddleValues (非推奨)",       category: "function",        detectionType: "text_match",        detectionValue: "MiddleValues",      enabled: true,  builtin: true },
  { id: "left_values",        label: "LeftValues (非推奨)",         category: "function",        detectionType: "text_match",        detectionValue: "LeftValues",        enabled: true,  builtin: true },
  { id: "right_values",       label: "RightValues (非推奨)",        category: "function",        detectionType: "text_match",        detectionValue: "RightValues",       enabled: true,  builtin: true },
  { id: "get_summary",        label: "GetSummary",                 category: "function",        detectionType: "text_match",        detectionValue: "GetSummary",        enabled: false, builtin: true },
  { id: "serial_field",       label: "シリアル採番フィールド",      category: "field",           detectionType: "field_attr",        detectionValue: "auto_enter_serial", enabled: true,  builtin: true },
  { id: "container",          label: "コンテナフィールド",          category: "field",           detectionType: "field_attr",        detectionValue: "container",         enabled: false, builtin: true },
  { id: "custom_function_call", label: "カスタム関数呼び出し",      category: "custom_function", detectionType: "any_custom_function", detectionValue: "",                enabled: true,  builtin: true },
];

function loadCheckItems(): CheckItem[] {
  try {
    const stored = localStorage.getItem(CHECK_ITEMS_KEY);
    if (stored) {
      const parsed: CheckItem[] = JSON.parse(stored);
      // builtin 項目が欠けていたら補完
      const ids = new Set(parsed.map((i) => i.id));
      const merged = [...parsed];
      for (const b of BUILTIN_CHECK_ITEMS) {
        if (!ids.has(b.id)) merged.push(b);
      }
      return merged;
    }
  } catch {}
  return BUILTIN_CHECK_ITEMS.map((i) => ({ ...i }));
}

export interface DiffStateData {
  solA: number | null;
  solB: number | null;
  committedA: number | null;
  committedB: number | null;
  expandedTypes: string[];
}

const INITIAL_DIFF_STATE: DiffStateData = {
  solA: null,
  solB: null,
  committedA: null,
  committedB: null,
  expandedTypes: [],
};

export type SelectedElement =
  | { kind: "table"; projectId: number; id: number; name: string }
  | { kind: "all_fields"; projectId: number }
  | { kind: "script"; projectId: number; id: number; name: string }
  | { kind: "layout"; projectId: number; id: number; name: string }
  | { kind: "value_list"; projectId: number; id: number; name: string }
  | { kind: "custom_function"; projectId: number; id: number; name: string }
  | { kind: "all_tables"; projectId: number }
  | { kind: "all_scripts"; projectId: number }
  | { kind: "all_layouts"; projectId: number }
  | { kind: "all_value_lists"; projectId: number }
  | { kind: "all_custom_functions"; projectId: number }
  | { kind: "all_table_occurrences"; projectId: number }
  | { kind: "all_relationships"; projectId: number; highlightId?: number }
  | { kind: "search"; query: string }
  | { kind: "dashboard" }
  | { kind: "diff" }
  | { kind: "security"; projectId: number }
  | { kind: "relationship_graph"; projectId: number }
  | { kind: "upgrade_check"; solutionId: number }
  | null;

export type RightPanelState =
  | { kind: "field"; fieldId: number; tableId: number; projectId: number; tableName: string }
  | { kind: "layout_object"; layoutObjectId: number; layoutId: number }
  | null;

/** SelectedElement の同一性を O(1) で比較するためのキー文字列を返す。 */
function elementKey(el: SelectedElement | undefined): string {
  if (el == null) return "null"; // null と undefined の両方を捕捉
  switch (el.kind) {
    case "table":
    case "script":
    case "layout":
    case "value_list":
    case "custom_function":
      return `${el.kind}:${el.projectId}:${el.id}`;
    case "all_fields":
    case "all_tables":
    case "all_scripts":
    case "all_layouts":
    case "all_value_lists":
    case "all_custom_functions":
    case "all_table_occurrences":
    case "security":
    case "relationship_graph":
      return `${el.kind}:${el.projectId}`;
    case "all_relationships":
      return `${el.kind}:${el.projectId}:${el.highlightId ?? ""}`;
    case "search":
      return `search:${el.query}`;
    case "upgrade_check":
      return `upgrade_check:${el.solutionId}`;
    case "dashboard":
    case "diff":
      return el.kind;
  }
}

const FONT_SIZE_KEY = "fm-ddr-font-size";
const DEFAULT_FONT_SIZE = 14;
const FONT_SIZE_MIN = 10;
const FONT_SIZE_MAX = 24;
const MAX_NAV_HISTORY = 50;

function loadFontSize(): number {
  try {
    const stored = localStorage.getItem(FONT_SIZE_KEY);
    if (stored) {
      const n = parseInt(stored, 10);
      if (!isNaN(n)) return n;
    }
  } catch {}
  return DEFAULT_FONT_SIZE;
}

interface AppState {
  solutions: SolutionRow[];
  selectedSolution: SolutionRow | null;
  selectedProject: ProjectRow | null;
  selectedElement: SelectedElement;
  searchQuery: string;
  fontSize: number;
  showAbout: boolean;
  showUpgradeSettings: boolean;
  rightPanel: RightPanelState;
  navHistory: SelectedElement[];
  navIndex: number;
  diffState: DiffStateData;
  diffContext: { compareProjectId: number } | null;
  searchDuration: number | null;
  searchContains: boolean;
  searchScope: "all" | "solution" | "project";
  checkItems: CheckItem[];
  setCheckItems: (items: CheckItem[]) => void;
  setSolutions: (solutions: SolutionRow[]) => void;
  setSearchDuration: (duration: number | null) => void;
  setSearchContains: (v: boolean) => void;
  setSearchScope: (v: "all" | "solution" | "project") => void;
  selectSolution: (solution: SolutionRow | null) => void;
  selectProject: (project: ProjectRow | null) => void;
  selectElement: (element: SelectedElement) => void;
  navigateFromDiff: (element: SelectedElement, compareProjectId: number) => void;
  setDiffState: (state: DiffStateData) => void;
  setSearchQuery: (query: string) => void;
  setFontSize: (size: number) => void;
  stepFontSize: (step: number) => void;
  setShowAbout: (show: boolean) => void;
  setShowUpgradeSettings: (show: boolean) => void;
  setRightPanel: (panel: RightPanelState) => void;
  navigateBack: () => void;
  navigateForward: () => void;
}

export const useAppStore = create<AppState>((set) => ({
  solutions: [],
  selectedSolution: null,
  selectedProject: null,
  selectedElement: null,
  searchQuery: "",
  fontSize: loadFontSize(),
  showAbout: false,
  showUpgradeSettings: false,
  rightPanel: null,
  navHistory: [],
  navIndex: -1,
  diffState: INITIAL_DIFF_STATE,
  diffContext: null,
  searchDuration: null,
  searchContains: false,
  searchScope: "all",
  checkItems: loadCheckItems(),
  setCheckItems: (items) => {
    try { localStorage.setItem(CHECK_ITEMS_KEY, JSON.stringify(items)); } catch {}
    set({ checkItems: items });
  },
  setSolutions: (solutions) => set({ solutions }),
  setSearchDuration: (duration) => set({ searchDuration: duration }),
  setSearchContains: (v) => set({ searchContains: v }),
  setSearchScope: (v) => set({ searchScope: v }),
  selectSolution: (solution) =>
    set({
      selectedSolution: solution,
      selectedProject: null,
      selectedElement: null,
      searchScope: "all",
      navHistory: [],
      navIndex: -1,
      diffContext: null,
      diffState: { ...INITIAL_DIFF_STATE, solA: solution?.id ?? null },
    }),
  selectProject: (project) =>
    set({ selectedProject: project, selectedElement: null, navHistory: [], navIndex: -1, diffContext: null }),
  selectElement: (element) =>
    set((state) => {
      // 同じ要素なら履歴に積まない
      const current = state.navHistory[state.navIndex];
      if (elementKey(current) === elementKey(element)) {
        return { selectedElement: element, searchQuery: "", diffContext: null };
      }
      // カーソルより後の履歴を切り捨てる
      let baseHistory = state.navHistory.slice(0, state.navIndex + 1);
      // 現在の表示状態を履歴に挿入してから新要素を積む
      let clearedQuery = state.searchQuery;
      if (state.searchQuery.trim()) {
        // 検索結果を表示中 → search エントリを挿入
        const searchEntry: SelectedElement = { kind: "search", query: state.searchQuery };
        const last = baseHistory[baseHistory.length - 1];
        if (elementKey(last) !== elementKey(searchEntry)) {
          baseHistory = [...baseHistory, searchEntry];
        }
        // 詳細パネルを表示するために searchQuery をクリア
        clearedQuery = "";
      } else if (state.selectedElement === null) {
        // ダッシュボードを表示中 → dashboard エントリを挿入
        const dashEntry: SelectedElement = { kind: "dashboard" };
        const last = baseHistory[baseHistory.length - 1];
        if (elementKey(last) !== elementKey(dashEntry)) {
          baseHistory = [...baseHistory, dashEntry];
        }
      }
      const newHistory = [...baseHistory, element].slice(-MAX_NAV_HISTORY);
      return {
        selectedElement: element,
        searchQuery: clearedQuery,
        navHistory: newHistory,
        navIndex: newHistory.length - 1,
        diffContext: null,
      };
    }),
  navigateFromDiff: (element, compareProjectId) =>
    set((state) => {
      const current = state.navHistory[state.navIndex];
      if (elementKey(current) === elementKey(element)) {
        return { selectedElement: element, diffContext: { compareProjectId } };
      }
      let baseHistory = state.navHistory.slice(0, state.navIndex + 1);
      let clearedQuery = state.searchQuery;
      if (state.searchQuery.trim()) {
        const searchEntry: SelectedElement = { kind: "search", query: state.searchQuery };
        const last = baseHistory[baseHistory.length - 1];
        if (elementKey(last) !== elementKey(searchEntry)) {
          baseHistory = [...baseHistory, searchEntry];
        }
        clearedQuery = "";
      } else if (state.selectedElement === null) {
        const dashEntry: SelectedElement = { kind: "dashboard" };
        const last = baseHistory[baseHistory.length - 1];
        if (elementKey(last) !== elementKey(dashEntry)) {
          baseHistory = [...baseHistory, dashEntry];
        }
      }
      const newHistory = [...baseHistory, element].slice(-MAX_NAV_HISTORY);
      return {
        selectedElement: element,
        searchQuery: clearedQuery,
        navHistory: newHistory,
        navIndex: newHistory.length - 1,
        diffContext: { compareProjectId },
      };
    }),
  setDiffState: (diffState) => set({ diffState }),
  setSearchQuery: (query) =>
    set((state) => {
      // search / dashboard エントリが selectedElement のとき（back/forward で戻ってきた状態）は
      // 新しいクエリ入力でライブ検索状態に切り替える
      const updates: Partial<AppState> = { searchQuery: query };
      if (state.selectedElement?.kind === "search" || state.selectedElement?.kind === "dashboard") {
        updates.selectedElement = null;
      }
      return updates;
    }),
  setFontSize: (size) => {
    const clamped = Math.min(FONT_SIZE_MAX, Math.max(FONT_SIZE_MIN, size));
    try { localStorage.setItem(FONT_SIZE_KEY, String(clamped)); } catch {}
    set({ fontSize: clamped });
  },
  stepFontSize: (step) =>
    set((state) => {
      // step === 0 はリセット
      const next = step === 0
        ? DEFAULT_FONT_SIZE
        : Math.min(FONT_SIZE_MAX, Math.max(FONT_SIZE_MIN, state.fontSize + step));
      try { localStorage.setItem(FONT_SIZE_KEY, String(next)); } catch {}
      return { fontSize: next };
    }),
  setShowAbout: (show) => set({ showAbout: show }),
  setShowUpgradeSettings: (show) => set({ showUpgradeSettings: show }),
  setRightPanel: (panel) => set({ rightPanel: panel }),
  navigateBack: () =>
    set((state) => {
      if (state.navIndex <= 0) return {};
      const newIndex = state.navIndex - 1;
      const entry = state.navHistory[newIndex];
      const updates: Partial<AppState> = { navIndex: newIndex, selectedElement: entry, diffContext: null };
      // search エントリなら searchQuery を復元、それ以外はクリアして詳細パネルを表示
      updates.searchQuery = entry?.kind === "search" ? entry.query : "";
      return updates;
    }),
  navigateForward: () =>
    set((state) => {
      if (state.navIndex >= state.navHistory.length - 1) return {};
      const newIndex = state.navIndex + 1;
      const entry = state.navHistory[newIndex];
      const updates: Partial<AppState> = { navIndex: newIndex, selectedElement: entry, diffContext: null };
      updates.searchQuery = entry?.kind === "search" ? entry.query : "";
      return updates;
    }),
}));
