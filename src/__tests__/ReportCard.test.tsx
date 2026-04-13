import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { ReportCard } from "../components/ReportCard";
import type { ReportCard as ReportCardType } from "../types/ddr";

vi.mock("../hooks/useTauriCommand", () => ({
  useReportCard: vi.fn(),
  useScriptList: vi.fn(() => ({ data: [], isLoading: false })),
  useLayoutList: vi.fn(() => ({ data: [], isLoading: false })),
}));

vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(() => ({ selectElement: vi.fn() })),
}));

import { useReportCard } from "../hooks/useTauriCommand";

describe("ReportCard", () => {
  it("renders_nothing_when_no_project", () => {
    vi.mocked(useReportCard).mockReturnValue(
      { data: undefined, isLoading: false } as unknown as ReturnType<typeof useReportCard>
    );
    const { container } = render(<ReportCard projectId={null} />);
    expect(container.firstChild).toBeNull();
  });

  it("renders_error_count", () => {
    const report: ReportCardType = {
      issues: [
        { severity: "Error", category: "BrokenRef", message: "Missing script" },
        { severity: "Error", category: "BrokenRef", message: "Missing field" },
      ],
      error_count: 2,
      warning_count: 0,
      info_count: 0,
    };
    vi.mocked(useReportCard).mockReturnValue(
      { data: report, isLoading: false } as unknown as ReturnType<typeof useReportCard>
    );
    render(<ReportCard projectId={1} />);
    expect(screen.getByText(/エラー 2/)).toBeInTheDocument();
  });

  it("renders_healthy_badge_when_no_issues", () => {
    const report: ReportCardType = {
      issues: [],
      error_count: 0,
      warning_count: 0,
      info_count: 0,
    };
    vi.mocked(useReportCard).mockReturnValue(
      { data: report, isLoading: false } as unknown as ReturnType<typeof useReportCard>
    );
    render(<ReportCard projectId={1} />);
    expect(screen.getByText("健全")).toBeInTheDocument();
  });
});
