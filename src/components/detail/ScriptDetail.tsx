// src/components/detail/ScriptDetail.tsx
import { useMemo, useState } from "react";
import { diffArrays } from "diff";
import { useScriptSteps, useScriptList } from "../../hooks/useTauriCommand";
import { useAppStore } from "../../stores/appStore";
import type { ScriptRow, ScriptStepRow } from "../../types/ddr";
import { WhereUsed } from "./WhereUsed";
import { CallChainTree } from "./CallChainTree";
import { Spinner } from "../Spinner";

interface Props {
  script: ScriptRow;
  projectId: number;
}

type DiffKind = "Added" | "Removed";

interface StepWithDiff extends ScriptStepRow {
  diffKind?: DiffKind;
  isPhantom?: boolean;
}

function stepSignature(s: ScriptStepRow): string {
  return [
    s.name,
    s.step_text ?? "",
    s.calculation ?? "",
    s.script_ref_name ?? "",
    s.script_ref_file ?? "",
    s.enabled ? "1" : "0",
  ].join("|");
}

type Tab = "steps" | "callchain";

/**
 * ステップ名からインデント増減を計算する。
 * FileMaker のネスト構造に対応:
 *   増加: If, Else If, Else, Loop, If [ フラグ ], Loop [ フラグ ]
 *   減少: End If, End Loop, Else If, Else (= 先にアウトデント、その後インデント)
 */
function getIndentDelta(stepName: string): { pre: number; post: number } {
  const name = stepName.trim().toLowerCase();
  // 先にアウトデントしてから同じレベルで表示
  if (name === "else if" || name === "else") return { pre: -1, post: 1 };
  // アウトデント
  if (name === "end if" || name === "end loop") return { pre: -1, post: 0 };
  // インデント
  if (name === "if" || name === "loop") return { pre: 0, post: 1 };
  return { pre: 0, post: 0 };
}

function computeIndents(steps: ScriptStepRow[]): number[] {
  const indents: number[] = [];
  let level = 0;
  for (const step of steps) {
    const { pre, post } = getIndentDelta(step.name);
    level = Math.max(0, level + pre);
    indents.push(level);
    level = Math.max(0, level + post);
  }
  return indents;
}

function StepsPanel({ script }: { script: ScriptRow }) {
  const { data: steps = [], isLoading } = useScriptSteps(script.id);
  const { diffContext } = useAppStore();

  const compareProjectId = diffContext?.compareProjectId ?? null;
  const { data: compareScriptList = [] } = useScriptList(compareProjectId);
  const compareScript = useMemo(
    () => compareScriptList.find((s) => s.name === script.name),
    [compareScriptList, script.name]
  );
  const { data: compareSteps = [] } = useScriptSteps(compareScript?.id ?? null);

  // LCS ベースのシーケンス diff でステップを比較
  const allRows = useMemo((): StepWithDiff[] => {
    if (!compareProjectId) return steps.map((s) => ({ ...s }));
    const hunks = diffArrays(compareSteps, steps, {
      comparator: (a, b) => stepSignature(a) === stepSignature(b),
    });
    const result: StepWithDiff[] = [];
    for (const hunk of hunks) {
      if (hunk.removed) {
        for (const s of hunk.value) {
          result.push({ ...s, diffKind: "Removed", isPhantom: true });
        }
      } else if (hunk.added) {
        for (const s of hunk.value) {
          result.push({ ...s, diffKind: "Added" });
        }
      } else {
        for (const s of hunk.value) {
          result.push({ ...s });
        }
      }
    }
    return result;
  }, [steps, compareSteps, compareProjectId]);

  const indents = useMemo(() => computeIndents(allRows), [allRows]);

  // ファントム行を除いた行番号を事前計算
  const lineNumbers = useMemo(() => {
    let n = 0;
    return allRows.map((row) => (row.isPhantom ? null : ++n));
  }, [allRows]);

  const hasDiff = compareProjectId !== null && allRows.some((r) => r.diffKind);

  if (isLoading) {
    return <div className="flex items-center gap-2 p-4 text-gray-500 text-sm"><Spinner className="w-4 h-4" />読み込み中...</div>;
  }

  return (
    <div className="p-4">
      {hasDiff && (
        <div className="flex gap-3 text-xs mb-2">
          {allRows.some((r) => r.diffKind === "Added") && (
            <span className="inline-flex items-center gap-1 text-green-700">
              <span className="w-2 h-2 rounded-full bg-green-400 inline-block" />
              追加
            </span>
          )}
          {allRows.some((r) => r.diffKind === "Removed") && (
            <span className="inline-flex items-center gap-1 text-red-600">
              <span className="w-2 h-2 rounded-full bg-red-400 inline-block" />
              削除
            </span>
          )}
        </div>
      )}
      {allRows.length === 0 ? (
        <p className="text-gray-500">ステップなし</p>
      ) : (
        <div className="font-mono text-sm bg-gray-50 border border-gray-200 rounded overflow-auto">
          {allRows.map((step, idx) => {
            const rowBg = step.isPhantom
              ? "bg-red-50 opacity-70 cursor-default"
              : step.diffKind === "Added"
              ? "bg-green-50"
              : step.enabled
              ? "hover:bg-blue-50"
              : "opacity-40 hover:bg-gray-100";
            return (
              <div
                key={step.isPhantom ? `phantom-${idx}` : step.id}
                className={`flex gap-3 px-3 py-1 border-b border-gray-100 last:border-b-0 ${rowBg}`}
              >
                {/* 行番号（ファントム行はダッシュ） */}
                <span className="text-gray-400 text-xs w-7 shrink-0 pt-0.5 text-right select-none">
                  {lineNumbers[idx] ?? "–"}
                </span>
                {/* インデント + ステップ内容 */}
                <div
                  className="flex-1 min-w-0"
                  style={{ paddingLeft: `${indents[idx] * 1.25}rem` }}
                >
                  {step.step_text ? (
                    <span
                      className={step.enabled ? "text-gray-800" : "text-gray-400 line-through"}
                    >
                      {step.step_text}
                    </span>
                  ) : (
                    <span className="text-gray-600">
                      {step.name}
                      {!step.step_text && step.script_ref_name && (
                        <span className="ml-2 text-blue-600">[{step.script_ref_name}]</span>
                      )}
                    </span>
                  )}
                </div>
                {/* 無効マーク（ファントム行には表示しない） */}
                {!step.enabled && !step.isPhantom && (
                  <span className="text-xs text-gray-400 shrink-0">[無効]</span>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

export function ScriptDetail({ script, projectId }: Props) {
  const [activeTab, setActiveTab] = useState<Tab>("steps");

  const tabs: { key: Tab; label: string }[] = [
    { key: "steps", label: "ステップ" },
    { key: "callchain", label: "コールチェーン" },
  ];

  return (
    <div className="flex flex-col h-full">
      {/* ヘッダー */}
      <div className="px-4 pt-4 pb-0 border-b border-gray-200 bg-white">
        <h2 className="text-lg font-bold mb-4 text-gray-800">{script.name}</h2>
        <div className="text-xs text-gray-500 mb-3 flex gap-3">
          {script.run_with_full_access && (
            <span className="text-orange-600 font-medium">完全アクセス権で実行</span>
          )}
          <span>{script.step_count} ステップ</span>
        </div>
        {/* タブ */}
        <div className="flex gap-0">
          {tabs.map((tab) => (
            <button
              key={tab.key}
              onClick={() => setActiveTab(tab.key)}
              className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
                activeTab === tab.key
                  ? "border-blue-500 text-blue-700"
                  : "border-transparent text-gray-500 hover:text-gray-700"
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>
      </div>

      {/* タブコンテンツ */}
      <div className="flex-1 overflow-auto">
        {activeTab === "steps" && (
          <div>
            <StepsPanel script={script} />
            <div className="border-t border-gray-200 mx-4">
              <WhereUsed projectId={projectId} scriptId={script.fm_id} />
            </div>
          </div>
        )}
        {activeTab === "callchain" && (
          <CallChainTree projectId={projectId} scriptFmId={script.fm_id} />
        )}
      </div>
    </div>
  );
}
