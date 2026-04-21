import { useEffect, useRef, useState, useCallback } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { getVersion } from "@tauri-apps/api/app";
import { ImportButton } from "./components/ImportButton";
import { SolutionList } from "./components/SolutionList";
import { SearchBar } from "./components/SearchBar";
import { RightPanel } from "./components/RightPanel";
import { FieldDetail } from "./components/detail/FieldDetail";
import { LayoutObjectDetail } from "./components/detail/LayoutObjectDetail";
import { MainContent } from "./components/MainContent";
import { StatusBar } from "./components/StatusBar";
import { UpgradeSettingsPanel } from "./components/detail/UpgradeSettingsPanel";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { useAppStore } from "./stores/appStore";
import { useTableFields } from "./hooks/table";
import "./index.css";

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false } },
});

const SIDEBAR_MIN = 180;
const SIDEBAR_MAX = 600;
const SIDEBAR_DEFAULT = 288; // w-72
const RIGHTPANEL_MIN = 200;
const RIGHTPANEL_MAX = 700;
const RIGHTPANEL_DEFAULT = 288; // w-72

function AboutModal({ onClose }: { onClose: () => void }) {
  const [version, setVersion] = useState<string>("");
  useEffect(() => { getVersion().then(setVersion); }, []);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      onClick={onClose}
    >
      <div
        className="bg-white rounded-lg shadow-xl p-6 w-80"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 className="text-lg font-bold text-blue-700 mb-1">FM DDR Analyzer</h2>
        <p className="text-sm text-gray-500 mb-3">バージョン {version}</p>
        <p className="text-sm text-gray-600 mb-4">
          FileMaker Database Design Report (DDR) XML を解析・可視化するデスクトップツール。
        </p>
        <button
          className="w-full px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 text-sm"
          onClick={onClose}
        >
          閉じる
        </button>
      </div>
    </div>
  );
}

function RightPanelContent({ width }: { width: number }) {
  const { rightPanel, setRightPanel } = useAppStore();

  if (!rightPanel) return null;

  if (rightPanel.kind === "field") {
    const { projectId, tableId, tableName, fieldId } = rightPanel;
    return (
      <FieldPanelInner
        width={width}
        projectId={projectId}
        tableId={tableId}
        tableName={tableName}
        fieldId={fieldId}
        onClose={() => setRightPanel(null)}
      />
    );
  }

  if (rightPanel.kind === "layout_object") {
    const { layoutObjectId, layoutId } = rightPanel;
    return (
      <RightPanel title="オブジェクト詳細" onClose={() => setRightPanel(null)} width={width}>
        <LayoutObjectDetail layoutObjectId={layoutObjectId} layoutId={layoutId} />
      </RightPanel>
    );
  }

  return null;
}

function FieldPanelInner({
  width,
  projectId,
  tableId,
  tableName,
  fieldId,
  onClose,
}: {
  width: number;
  projectId: number;
  tableId: number;
  tableName: string;
  fieldId: number;
  onClose: () => void;
}) {
  const { data: fields = [] } = useTableFields(projectId, tableId);
  const field = fields.find((f) => f.id === fieldId);
  if (!field) return null;
  return (
    <RightPanel title="フィールド詳細" onClose={onClose} width={width}>
      <FieldDetail field={field} tableName={tableName} projectId={projectId} />
    </RightPanel>
  );
}

function App() {
  const {
    selectedElement,
    fontSize,
    showAbout,
    showUpgradeSettings,
    rightPanel,
    stepFontSize,
    setShowAbout,
    setShowUpgradeSettings,
    selectElement,
    setRightPanel,
    navHistory,
    navIndex,
    navigateBack,
    navigateForward,
  } = useAppStore();
  const [sidebarWidth, setSidebarWidth] = useState(SIDEBAR_DEFAULT);
  const [rightPanelWidth, setRightPanelWidth] = useState(RIGHTPANEL_DEFAULT);
  const isDragging = useRef<"left" | "right" | null>(null);
  const startX = useRef(0);
  const startWidth = useRef(0);

  // フォントサイズを html 要素に適用（rem ベースの Tailwind クラス全体に反映）
  useEffect(() => {
    document.documentElement.style.fontSize = `${fontSize}px`;
  }, [fontSize]);

  // マウスサイドボタン（戻る: button 3 / 進む: button 4）でナビゲーション履歴を操作
  useEffect(() => {
    const handleMouseNav = (e: MouseEvent) => {
      if (e.button === 3) { e.preventDefault(); navigateBack(); }
      if (e.button === 4) { e.preventDefault(); navigateForward(); }
    };
    window.addEventListener("mousedown", handleMouseNav);
    return () => window.removeEventListener("mousedown", handleMouseNav);
  }, [navigateBack, navigateForward]);

  // Tauri メニューイベントを購読
  useEffect(() => {
    const unlisten1 = listen<number>("font-size-step", (e) => {
      stepFontSize(e.payload);
    });
    const unlisten2 = listen<void>("show-about", () => {
      setShowAbout(true);
    });
    const unlisten3 = listen<void>("open-upgrade-settings", () => {
      setShowUpgradeSettings(true);
    });
    return () => {
      unlisten1.then((f) => f());
      unlisten2.then((f) => f());
      unlisten3.then((f) => f());
    };
  }, [stepFontSize, setShowAbout, setShowUpgradeSettings]);

  // サイドバー・右パネルリサイズ
  const onLeftDragStart = useCallback((e: React.MouseEvent) => {
    isDragging.current = "left";
    startX.current = e.clientX;
    startWidth.current = sidebarWidth;
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
  }, [sidebarWidth]);

  const onRightDragStart = useCallback((e: React.MouseEvent) => {
    isDragging.current = "right";
    startX.current = e.clientX;
    startWidth.current = rightPanelWidth;
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
  }, [rightPanelWidth]);

  useEffect(() => {
    const onMove = (e: MouseEvent) => {
      if (!isDragging.current) return;
      const delta = e.clientX - startX.current;
      if (isDragging.current === "left") {
        const newWidth = Math.min(SIDEBAR_MAX, Math.max(SIDEBAR_MIN, startWidth.current + delta));
        setSidebarWidth(newWidth);
      } else {
        // 右パネルは左へドラッグすると広がる（逆方向）
        const newWidth = Math.min(RIGHTPANEL_MAX, Math.max(RIGHTPANEL_MIN, startWidth.current - delta));
        setRightPanelWidth(newWidth);
      }
    };
    const onUp = () => {
      isDragging.current = null;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
    return () => {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
  }, []);

  return (
    <QueryClientProvider client={queryClient}>
      <div
        className="flex flex-col h-screen bg-gray-50"
      >
        {/* ヘッダー */}
        <header className="flex items-center gap-4 px-4 py-2 bg-white border-b border-gray-200 shrink-0">
          <ImportButton />
          <div className="flex-1">
            <SearchBar />
          </div>
        </header>

        {/* ボディ */}
        <div className="flex flex-1 overflow-hidden">
          {/* サイドバー */}
          <aside
            className="border-r border-gray-200 bg-white overflow-auto flex flex-col shrink-0"
            style={{ width: sidebarWidth }}
          >
            <div className="border-b border-gray-200">
              <button
                className={`w-full flex items-center gap-2 px-3 py-2 text-sm font-semibold transition-colors ${
                  selectedElement?.kind === "diff"
                    ? "bg-blue-100 text-blue-800"
                    : "text-gray-700 hover:bg-gray-100"
                }`}
                onClick={() => {
                  setRightPanel(null);
                  selectElement({ kind: "diff" });
                }}
              >
                <span>⇄</span>
                <span>差分比較</span>
              </button>
            </div>
            <SolutionList />
          </aside>

          {/* 左ドラッグハンドル */}
          <div
            className="w-px cursor-col-resize bg-gray-200 hover:bg-blue-400 active:bg-blue-500 transition-colors shrink-0"
            onMouseDown={onLeftDragStart}
          />

          {/* メインエリア（ナビバー + コンテンツ） */}
          <div className="flex-1 flex flex-col overflow-hidden">
            {/* ← → ナビゲーションバー */}
            <div className="flex items-center gap-1 px-2 py-1 border-b border-gray-200 bg-white shrink-0">
              <button
                onClick={navigateBack}
                disabled={navIndex <= 0}
                className="w-7 h-7 flex items-center justify-center rounded hover:bg-gray-100 disabled:opacity-30 disabled:cursor-not-allowed text-gray-500"
                title="戻る (Alt+←)"
              >
                ◀
              </button>
              <button
                onClick={navigateForward}
                disabled={navIndex >= navHistory.length - 1}
                className="w-7 h-7 flex items-center justify-center rounded hover:bg-gray-100 disabled:opacity-30 disabled:cursor-not-allowed text-gray-500"
                title="進む (Alt+→)"
              >
                ▶
              </button>
            </div>
            <ErrorBoundary resetKey={selectedElement}>
              <MainContent />
            </ErrorBoundary>
          </div>

          {/* 右パネル */}
          {rightPanel && (
            <>
              {/* 右ドラッグハンドル */}
              <div
                className="w-px cursor-col-resize bg-gray-200 hover:bg-blue-400 active:bg-blue-500 transition-colors shrink-0"
                onMouseDown={onRightDragStart}
              />
              <ErrorBoundary resetKey={rightPanel}>
                <RightPanelContent width={rightPanelWidth} />
              </ErrorBoundary>
            </>
          )}
        </div>
        <StatusBar />
      </div>

      {/* バージョン情報モーダル */}
      {showAbout && <AboutModal onClose={() => setShowAbout(false)} />}

      {/* アップグレードチェック設定モーダル */}
      {showUpgradeSettings && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
          onClick={() => setShowUpgradeSettings(false)}
        >
          <div
            className="bg-white rounded-lg shadow-xl w-[680px] max-w-[90vw] max-h-[80vh] overflow-auto"
            onClick={(e) => e.stopPropagation()}
          >
            <UpgradeSettingsPanel onClose={() => setShowUpgradeSettings(false)} />
          </div>
        </div>
      )}
    </QueryClientProvider>
  );
}

export default App;
