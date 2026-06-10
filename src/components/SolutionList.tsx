import { useState, useRef } from "react";
import { useSolutions, useDeleteSolution, useDeleteProject, useSolutionProjects, useRenameSolution } from "../hooks/solutions";
import { useAppStore } from "../stores/appStore";
import { CategoryTree } from "./navigation/CategoryTree";
import { Spinner } from "./Spinner";
import type { SolutionRow } from "../types/ddr";

function ProjectItems({ solution }: { solution: SolutionRow }) {
  const { data: projects, isLoading } = useSolutionProjects(solution.id);
  const { selectedProject, selectProject, setRightPanel, purgeProjectFromHistory } = useAppStore();
  const { mutate: deleteProject, isPending: isDeletingProject } = useDeleteProject();
  const [confirmingProjectId, setConfirmingProjectId] = useState<number | null>(null);
  const [deletingProjectId, setDeletingProjectId] = useState<number | null>(null);

  if (isLoading) {
    return <li className="flex items-center gap-1.5 px-6 py-1 text-xs text-gray-400"><Spinner className="w-3 h-3" />読み込み中...</li>;
  }

  if (!projects || projects.length === 0) {
    return <li className="px-6 py-1 text-xs text-gray-400">ファイルなし</li>;
  }

  return (
    <>
      {projects.map((project) => {
        const isSelected = selectedProject?.id === project.id;
        const isDeleting = deletingProjectId === project.id && isDeletingProject;
        return (
          <li key={project.id} className={isDeleting ? "opacity-50 pointer-events-none" : ""}>
            <div
              className={`flex items-center justify-between px-6 py-1.5 cursor-pointer text-sm hover:bg-blue-50 ${
                isSelected ? "bg-blue-100 text-blue-700 font-medium" : "text-gray-700"
              }`}
              onClick={() => { setRightPanel(null); selectProject(project); }}
            >
              <span className="truncate" title={project.name}>
                📄 {project.name}
                <span className="ml-1 text-xs text-gray-400">{project.fm_version}</span>
              </span>
              {isDeleting ? (
                <Spinner className="w-3.5 h-3.5 ml-1" />
              ) : (
                <button
                  className="ml-1 text-gray-300 hover:text-red-500 flex-shrink-0 text-lg leading-none"
                  onClick={(e) => {
                    e.stopPropagation();
                    setConfirmingProjectId(project.id);
                  }}
                  aria-label="削除"
                >
                  ×
                </button>
              )}
            </div>
            {confirmingProjectId === project.id && (
              <div className="flex items-center gap-2 px-6 py-1.5 bg-red-50 border-t border-red-100 text-xs">
                <span className="text-red-700 flex-1">「{project.name}」を削除しますか？</span>
                <button
                  className="px-2 py-0.5 bg-red-600 text-white rounded hover:bg-red-700"
                  onClick={() => {
                    setDeletingProjectId(project.id);
                    deleteProject(project.id, {
                      onSettled: () => setDeletingProjectId(null),
                    });
                    purgeProjectFromHistory(project.id);
                    if (selectedProject?.id === project.id) {
                      setRightPanel(null);
                      selectProject(null);
                    }
                    setConfirmingProjectId(null);
                  }}
                >
                  削除
                </button>
                <button
                  className="px-2 py-0.5 bg-gray-200 text-gray-700 rounded hover:bg-gray-300"
                  onClick={() => setConfirmingProjectId(null)}
                >
                  キャンセル
                </button>
              </div>
            )}
            {/* 選択中プロジェクトの直下に項目ツリーを表示 */}
            {isSelected && (
              <div className="border-t border-gray-100">
                <CategoryTree projectId={project.id} />
              </div>
            )}
          </li>
        );
      })}
    </>
  );
}

export function SolutionList() {
  const { data: solutions, isLoading, isError } = useSolutions();
  const { mutate: deleteSolution, isPending: isDeletingSolution } = useDeleteSolution();
  const { mutate: renameSolution } = useRenameSolution();
  const { selectedSolution, selectSolution, selectProject, selectElement, setRightPanel, renameSolutionInStore } = useAppStore();
  const [confirmingSolutionId, setConfirmingSolutionId] = useState<number | null>(null);
  const [deletingSolutionId, setDeletingSolutionId] = useState<number | null>(null);
  const [renamingSolutionId, setRenamingSolutionId] = useState<number | null>(null);
  const [renameValue, setRenameValue] = useState("");
  const resolvedRef = useRef(false);

  if (isLoading) {
    return <div className="flex items-center gap-2 p-4 text-gray-500 text-sm"><Spinner className="w-4 h-4" />読み込み中...</div>;
  }

  if (isError) {
    return <div className="p-4 text-red-500 text-sm">読み込みエラー</div>;
  }

  if (!solutions || solutions.length === 0) {
    return (
      <div className="p-4 text-gray-400 text-sm">
        DDR をインポートしてください
      </div>
    );
  }

  return (
    <ul className="divide-y divide-gray-100">
      {solutions.map((solution) => {
        const isDeleting = deletingSolutionId === solution.id && isDeletingSolution;
        const isRenaming = renamingSolutionId === solution.id;
        return (
        <li key={solution.id} className={isDeleting ? "opacity-50 pointer-events-none" : ""}>
          {/* ソリューション行 */}
          <div
            className={`group flex items-center justify-between px-3 py-2 cursor-pointer hover:bg-gray-50 ${
              selectedSolution?.id === solution.id ? "bg-gray-100" : ""
            }`}
            onClick={() => {
              if (!isRenaming) {
                setRightPanel(null);
                selectSolution(solution);
              }
            }}
          >
            <div className="flex-1 min-w-0">
              {isRenaming ? (
                <input
                  className="w-full px-1 text-sm font-medium border border-blue-400 rounded outline-none"
                  value={renameValue}
                  autoFocus
                  onChange={(e) => setRenameValue(e.target.value)}
                  onClick={(e) => e.stopPropagation()}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      if (!resolvedRef.current) {
                        resolvedRef.current = true;
                        const trimmed = renameValue.trim();
                        if (trimmed && trimmed !== solution.name) {
                          renameSolution(
                            { solutionId: solution.id, newName: trimmed },
                            { onSuccess: () => renameSolutionInStore(solution.id, trimmed) }
                          );
                        }
                      }
                      setRenamingSolutionId(null);
                    } else if (e.key === "Escape") {
                      resolvedRef.current = true;
                      setRenamingSolutionId(null);
                    }
                  }}
                  onBlur={() => {
                    if (!resolvedRef.current) {
                      resolvedRef.current = true;
                      const trimmed = renameValue.trim();
                      if (trimmed && trimmed !== solution.name) {
                        renameSolution(
                          { solutionId: solution.id, newName: trimmed },
                          { onSuccess: () => renameSolutionInStore(solution.id, trimmed) }
                        );
                      }
                    }
                    setRenamingSolutionId(null);
                  }}
                />
              ) : (
                <>
                  <div className="truncate text-sm font-medium text-gray-900" title={solution.name}>
                    🗂 {solution.name}
                  </div>
                  <div className="text-xs text-gray-400">
                    {solution.imported_at.slice(0, 16)}
                  </div>
                </>
              )}
            </div>
            {isDeleting ? (
              <Spinner className="w-4 h-4 ml-2" />
            ) : (
              <span className="flex items-center gap-0.5 flex-shrink-0 ml-2">
                {!confirmingSolutionId && !isRenaming && (
                  <button
                    className="text-gray-300 hover:text-blue-500 opacity-0 group-hover:opacity-100 text-sm leading-none"
                    onClick={(e) => {
                      e.stopPropagation();
                      resolvedRef.current = false;
                      setRenamingSolutionId(solution.id);
                      setRenameValue(solution.name);
                    }}
                    aria-label="名前を変更"
                  >
                    ✏
                  </button>
                )}
                <button
                  className="ml-0.5 text-gray-300 hover:text-red-500 opacity-0 group-hover:opacity-100 text-lg leading-none"
                  onClick={(e) => {
                    e.stopPropagation();
                    setConfirmingSolutionId(solution.id);
                  }}
                  aria-label="削除"
                >
                  ×
                </button>
              </span>
            )}
          </div>
          {confirmingSolutionId === solution.id && (
            <div className="flex items-center gap-2 px-3 py-1.5 bg-red-50 border-t border-red-100 text-xs">
              <span className="text-red-700 flex-1">「{solution.name}」を削除しますか？</span>
              <button
                className="px-2 py-0.5 bg-red-600 text-white rounded hover:bg-red-700"
                onClick={() => {
                  setDeletingSolutionId(solution.id);
                  deleteSolution(solution.id, {
                    onSettled: () => setDeletingSolutionId(null),
                  });
                  if (selectedSolution?.id === solution.id) {
                    setRightPanel(null);
                    selectSolution(null);
                    selectProject(null);
                  }
                  setConfirmingSolutionId(null);
                }}
              >
                削除
              </button>
              <button
                className="px-2 py-0.5 bg-gray-200 text-gray-700 rounded hover:bg-gray-300"
                onClick={() => setConfirmingSolutionId(null)}
              >
                キャンセル
              </button>
            </div>
          )}
          {/* ソリューション展開時: アップグレードチェック + プロジェクト一覧 */}
          {selectedSolution?.id === solution.id && (
            <>
              <div className="px-3 py-1 border-t border-gray-100">
                <button
                  className="w-full flex items-center gap-1.5 px-2 py-1 text-xs font-semibold text-gray-600 hover:bg-gray-100 rounded transition-colors"
                  onClick={() => {
                    setRightPanel(null);
                    selectElement({ kind: "upgrade_check", solutionId: solution.id });
                  }}
                >
                  <span>🔍</span>
                  アップグレードチェック
                </button>
              </div>
              <ul>
                <ProjectItems solution={solution} />
              </ul>
            </>
          )}
        </li>
      );
      })}
    </ul>
  );
}
