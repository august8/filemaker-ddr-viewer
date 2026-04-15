// src/components/detail/UpgradeCheckPanel.tsx
import { useMemo, useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../../stores/appStore";
import { useUpgradeCheck } from "../../hooks/useTauriCommand";
import type { UpgradeHit } from "../../types/ddr";
import { CARD, SECTION_HEADER } from "../../styles/tokens";
import { Spinner } from "../Spinner";

interface Props {
  solutionId: number;
}

type HitGroup = {
  itemId: string;
  label: string;
  hits: UpgradeHit[];
};

async function exportCsv(hits: UpgradeHit[]) {
  const BOM = "\uFEFF";
  const header = "item_id,project_name,custom_function_name,script_name,step_name,step_text,field_name,table_name";
  const rows = hits.map((h) =>
    [
      h.item_id,
      h.project_name,
      h.custom_function_name ?? "",
      h.script_name ?? "",
      h.step_name ?? "",
      (h.step_text ?? "").replace(/"/g, '""'),
      h.field_name ?? "",
      h.table_name ?? "",
    ]
      .map((v) => `"${v}"`)
      .join(",")
  );
  const csv = BOM + [header, ...rows].join("\r\n");

  const defaultName = `upgrade_check_${new Date().toISOString().replace(/[:.]/g, "").slice(0, 15)}.csv`;
  const path = await save({
    title: "CSV エクスポート",
    defaultPath: defaultName,
    filters: [{ name: "CSV", extensions: ["csv"] }],
  });
  if (!path) return;

  await invoke("write_text_file", { path, content: csv });
}

export function UpgradeCheckPanel({ solutionId }: Props) {
  const { checkItems, selectElement, setRightPanel } = useAppStore();
  const { data: hits = [], isLoading } = useUpgradeCheck(solutionId, checkItems);
  const [openGroups, setOpenGroups] = useState<Set<string>>(new Set());

  function toggleGroup(itemId: string) {
    setOpenGroups((prev) => {
      const next = new Set(prev);
      next.has(itemId) ? next.delete(itemId) : next.add(itemId);
      return next;
    });
  }

  const labelMap = useMemo(() => {
    const m: Record<string, string> = {};
    for (const item of checkItems) m[item.id] = item.label;
    return m;
  }, [checkItems]);

  const groups = useMemo<HitGroup[]>(() => {
    const byId: Record<string, UpgradeHit[]> = {};
    for (const hit of hits) {
      if (!byId[hit.item_id]) byId[hit.item_id] = [];
      byId[hit.item_id].push(hit);
    }
    return checkItems
      .filter((i) => i.enabled)
      .map((i) => ({ itemId: i.id, label: labelMap[i.id] ?? i.id, hits: byId[i.id] ?? [] }));
  }, [hits, checkItems, labelMap]);

  // サマリーグリッド用データ
  const projects = useMemo(
    () => [...new Set(hits.map((h) => h.project_name))].sort(),
    [hits]
  );
  const countMatrix = useMemo(() => {
    const m: Record<string, Record<string, number>> = {};
    for (const hit of hits) {
      if (!m[hit.item_id]) m[hit.item_id] = {};
      m[hit.item_id][hit.project_name] = (m[hit.item_id][hit.project_name] ?? 0) + 1;
    }
    return m;
  }, [hits]);

  function handleScriptClick(hit: UpgradeHit) {
    if (hit.script_id != null && hit.script_name != null) {
      selectElement({ kind: "script", projectId: hit.project_id, id: hit.script_id, name: hit.script_name });
    }
  }

  function handleFieldClick(hit: UpgradeHit) {
    if (hit.field_id != null && hit.table_id != null && hit.table_name != null) {
      selectElement({ kind: "table", projectId: hit.project_id, id: hit.table_id, name: hit.table_name });
      setRightPanel({
        kind: "field",
        projectId: hit.project_id,
        tableId: hit.table_id,
        fieldId: hit.field_id,
        tableName: hit.table_name,
      });
    }
  }

  if (isLoading) {
    return (
      <div className="flex-1 overflow-auto p-6">
        <h2 className="text-lg font-bold text-gray-800 mb-4">アップグレードチェック</h2>
        <div className="flex justify-center py-8"><Spinner className="w-5 h-5" /></div>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-auto p-6 space-y-4">
      {/* ヘッダー */}
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-bold text-gray-800">アップグレードチェック</h2>
        <button
          disabled={hits.length === 0}
          onClick={() => exportCsv(hits)}
          className="px-3 py-1.5 text-xs font-medium rounded border transition-colors
            disabled:opacity-40 disabled:cursor-not-allowed
            enabled:bg-white enabled:border-gray-300 enabled:text-gray-700 enabled:hover:bg-gray-50"
        >
          CSV エクスポート
        </button>
      </div>

      {groups.length === 0 ? (
        <div className={CARD}>
          <p className="text-sm text-gray-400 text-center py-4">ヒットなし</p>
        </div>
      ) : (
        <>
          {/* サマリーグリッド */}
          {groups.length > 0 && (
            <div className={`${CARD} overflow-x-auto`}>
              <p className={`${SECTION_HEADER} mb-3`}>サマリー</p>
              <table className="w-full text-xs border-collapse">
                <thead>
                  <tr>
                    <th className="text-left py-1 pr-3 text-gray-500 font-medium whitespace-nowrap">チェック項目</th>
                    {projects.map((p) => (
                      <th key={p} className="text-right py-1 px-2 text-gray-500 font-medium whitespace-nowrap" title={p}>
                        {p.length > 12 ? `${p.slice(0, 12)}…` : p}
                      </th>
                    ))}
                    <th className="text-right py-1 pl-2 text-gray-600 font-semibold">合計</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-100">
                  {groups.map((group) => (
                    <tr
                      key={group.itemId}
                      className="hover:bg-gray-50 cursor-pointer"
                      onClick={() => {
                        toggleGroup(group.itemId);
                        // スクロールはブラウザに任せる
                      }}
                    >
                      <td className="py-1 pr-3 text-gray-700 whitespace-nowrap">{group.label}</td>
                      {projects.map((p) => (
                        <td key={p} className="py-1 px-2 text-right text-gray-600">
                          {countMatrix[group.itemId]?.[p] ?? "—"}
                        </td>
                      ))}
                      <td className="py-1 pl-2 text-right font-semibold text-gray-800">
                        {group.hits.length}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {/* アコーディオン */}
          <div className="space-y-2">
            {groups.map((group) => {
              const isOpen = openGroups.has(group.itemId);
              // カスタム関数グループはCF名でサブグループ化
              const isCfGroup = group.hits.some((h) => h.custom_function_name != null);
              const cfSubGroups: Record<string, UpgradeHit[]> = {};
              if (isCfGroup) {
                for (const hit of group.hits) {
                  const key = hit.custom_function_name ?? "(不明)";
                  if (!cfSubGroups[key]) cfSubGroups[key] = [];
                  cfSubGroups[key].push(hit);
                }
              }

              return (
                <div key={group.itemId} className={CARD}>
                  {/* グループヘッダー（クリックで展開/折りたたみ） */}
                  <button
                    className="w-full flex items-center justify-between text-left"
                    onClick={() => toggleGroup(group.itemId)}
                  >
                    <span className={SECTION_HEADER}>{group.label}</span>
                    <span className="flex items-center gap-2">
                      <span className="text-xs text-gray-400">{group.hits.length} 件</span>
                      <span className="text-gray-400 text-xs">{isOpen ? "▲" : "▼"}</span>
                    </span>
                  </button>

                  {/* 展開時のヒット一覧 */}
                  {isOpen && (
                    <div className="mt-3 divide-y divide-gray-100">
                      {isCfGroup ? (
                        // カスタム関数グループ: CF名でサブグループ化
                        Object.entries(cfSubGroups).sort(([a], [b]) => a.localeCompare(b)).map(([cfName, cfHits]) => (
                          <div key={cfName} className="pt-2">
                            <p className="text-xs font-semibold text-indigo-600 mb-1 pb-1 border-b border-indigo-100">
                              {cfName}()
                              <span className="ml-1 font-normal text-gray-400">{cfHits.length} 件</span>
                            </p>
                            {cfHits.map((hit, i) => (
                              <HitRow key={i} hit={hit} onScriptClick={handleScriptClick} onFieldClick={handleFieldClick} />
                            ))}
                          </div>
                        ))
                      ) : (
                        group.hits.map((hit, i) => (
                          <HitRow key={i} hit={hit} onScriptClick={handleScriptClick} onFieldClick={handleFieldClick} />
                        ))
                      )}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </>
      )}
    </div>
  );
}

function HitRow({
  hit,
  onScriptClick,
  onFieldClick,
}: {
  hit: UpgradeHit;
  onScriptClick: (h: UpgradeHit) => void;
  onFieldClick: (h: UpgradeHit) => void;
}) {
  const isScript = hit.script_id != null;
  const isField = !isScript && hit.field_id != null;
  return (
    <div className="py-2 flex flex-col gap-0.5">
      <span className="text-xs text-gray-400">{hit.project_name}</span>
      {isScript ? (
        <>
          <button
            className="text-left text-sm text-blue-700 hover:underline"
            onClick={() => onScriptClick(hit)}
          >
            {hit.script_name ?? "(不明なスクリプト)"}
          </button>
          {hit.step_text && (
            <span className="text-xs text-gray-500 font-mono truncate">{hit.step_text}</span>
          )}
        </>
      ) : isField ? (
        <button
          className="text-left text-sm text-blue-700 hover:underline"
          onClick={() => onFieldClick(hit)}
        >
          {hit.table_name ? `${hit.table_name}::` : ""}
          {hit.field_name ?? "(不明なフィールド)"}
        </button>
      ) : (
        <span className="text-sm text-gray-700">
          {hit.table_name ? `${hit.table_name}::` : ""}
          {hit.field_name ?? "(不明なフィールド)"}
        </span>
      )}
    </div>
  );
}
