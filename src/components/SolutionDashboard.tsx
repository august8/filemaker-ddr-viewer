import { useSolutionProjects } from "../hooks/solutions";
import { ProjectSummaryCard } from "./ProjectSummaryCard";
import { Spinner } from "./Spinner";

interface Props {
  solutionId: number;
  solutionName: string;
}

export function SolutionDashboard({ solutionId, solutionName }: Props) {
  const { data: projects, isLoading } = useSolutionProjects(solutionId);

  if (isLoading) {
    return (
      <div className="flex-1 flex items-center justify-center gap-2 text-gray-400 text-sm" data-testid="solution-dashboard-spinner">
        <Spinner className="w-4 h-4" />読み込み中...
      </div>
    );
  }

  if (!projects || projects.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center text-gray-400 text-sm" data-testid="solution-dashboard-empty">
        「{solutionName}」にはファイルがありません
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-auto p-4 space-y-4">
      {projects.map((project) => (
        <div key={project.id} data-testid={`solution-project-card-${project.id}`}>
          <ProjectSummaryCard projectId={project.id} />
        </div>
      ))}
    </div>
  );
}
