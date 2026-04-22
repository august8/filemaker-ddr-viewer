// src/components/detail/CallChainTree.tsx
import { useState } from "react";
import { useCallChain, useScriptList } from "../../hooks/script";
import { useAppStore } from "../../stores/appStore";
import type { CallChainNode } from "../../types/ddr";
import { Spinner } from "../Spinner";

interface Props {
  projectId: number;
  scriptFmId: number;
}

interface NodeProps {
  node: CallChainNode;
  projectId: number;
  scripts: { id: number; fm_id: number; name: string }[];
  onNavigate: (id: number, name: string) => void;
}

function TreeNode({ node, projectId, scripts, onNavigate }: NodeProps) {
  const [expanded, setExpanded] = useState(true);
  const hasChildren = node.children.length > 0;
  const script = scripts.find((s) => s.fm_id === node.script_id);

  return (
    <div>
      <div className="flex items-center gap-1 group">
        {/* 展開トグル */}
        <button
          className={`w-4 h-4 flex items-center justify-center text-gray-400 shrink-0 ${
            hasChildren ? "hover:text-gray-700 cursor-pointer" : "cursor-default"
          }`}
          onClick={() => hasChildren && setExpanded((v) => !v)}
          tabIndex={hasChildren ? 0 : -1}
        >
          {hasChildren ? (expanded ? "▾" : "▸") : "·"}
        </button>

        {/* スクリプト名 */}
        {node.is_cycle ? (
          <span className="text-xs px-2 py-0.5 rounded bg-orange-100 text-orange-700 font-mono">
            ↩ {node.script_name} (循環)
          </span>
        ) : script ? (
          <button
            className="text-sm text-blue-700 hover:underline text-left"
            onClick={() => onNavigate(script.id, script.name)}
          >
            {node.script_name}
          </button>
        ) : (
          <span className="text-sm text-gray-600">{node.script_name}</span>
        )}

        {/* 深度バッジ（ルート以外） */}
        {node.depth > 0 && (
          <span className="text-xs text-gray-300 font-mono">d{node.depth}</span>
        )}
      </div>

      {/* 子ノード */}
      {hasChildren && expanded && (
        <div className="ml-5 mt-0.5 border-l border-gray-200 pl-3 space-y-0.5">
          {node.children.map((child, idx) => (
            <TreeNode
              key={`${child.script_id}-${idx}`}
              node={child}
              projectId={projectId}
              scripts={scripts}
              onNavigate={onNavigate}
            />
          ))}
        </div>
      )}
    </div>
  );
}

export function CallChainTree({ projectId, scriptFmId }: Props) {
  const { data: chain, isLoading, isError } = useCallChain(projectId, scriptFmId);
  const { data: scripts = [] } = useScriptList(projectId);
  const selectElement = useAppStore((s) => s.selectElement);

  const handleNavigate = (id: number, name: string) => {
    selectElement({ kind: "script", projectId, id, name });
  };

  if (isLoading) {
    return <div className="flex items-center gap-2 text-gray-400 text-sm p-4"><Spinner className="w-4 h-4" />読み込み中...</div>;
  }

  if (isError || !chain) {
    return <div className="text-red-500 text-sm p-4">コールチェーンの取得に失敗しました</div>;
  }

  return (
    <div className="p-4">
      <p className="text-xs text-gray-500 mb-3">
        クリックでスクリプト詳細へ。▸/▾ で折りたたみ可能。
      </p>
      <div className="font-mono text-sm space-y-0.5">
        <TreeNode
          node={chain}
          projectId={projectId}
          scripts={scripts}
          onNavigate={handleNavigate}
        />
      </div>
    </div>
  );
}
