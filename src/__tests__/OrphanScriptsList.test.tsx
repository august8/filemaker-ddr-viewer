import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { OrphanScriptsList } from "../components/OrphanScriptsList";
import type { OrphanScript } from "../types/ddr";
import { makeScriptRow } from "./testFixtures";

vi.mock("../hooks/analysis", () => ({
  useOrphanScripts: vi.fn(),
}));
vi.mock("../hooks/script", () => ({
  useScriptList: vi.fn(),
}));

vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn((selector: (s: { selectElement: ReturnType<typeof vi.fn> }) => unknown) =>
    selector({ selectElement: mockSelectElement })
  ),
}));

const mockSelectElement = vi.fn();

import { useOrphanScripts } from "../hooks/analysis";
import { useScriptList } from "../hooks/script";

const mockScripts = [
  makeScriptRow({ id: 10, fm_id: 101, name: "UnusedScript", step_count: 3 }),
  makeScriptRow({ id: 11, fm_id: 102, name: "AnotherUnused" }),
];

const mockOrphans: OrphanScript[] = [
  { script_id: 101, script_name: "UnusedScript" },
  { script_id: 102, script_name: "AnotherUnused" },
];

beforeEach(() => {
  vi.clearAllMocks();
});

describe("OrphanScriptsList", () => {
  it("renders_empty_message_when_no_orphans", () => {
    vi.mocked(useOrphanScripts).mockReturnValue(
      { data: [], isLoading: false } as unknown as ReturnType<typeof useOrphanScripts>
    );
    vi.mocked(useScriptList).mockReturnValue(
      { data: mockScripts } as unknown as ReturnType<typeof useScriptList>
    );
    render(<OrphanScriptsList projectId={1} />);
    expect(screen.getByText("未使用スクリプトはありません")).toBeInTheDocument();
  });

  it("renders_orphan_script_names", () => {
    vi.mocked(useOrphanScripts).mockReturnValue(
      { data: mockOrphans, isLoading: false } as unknown as ReturnType<typeof useOrphanScripts>
    );
    vi.mocked(useScriptList).mockReturnValue(
      { data: mockScripts } as unknown as ReturnType<typeof useScriptList>
    );
    render(<OrphanScriptsList projectId={1} />);
    expect(screen.getByText("UnusedScript")).toBeInTheDocument();
    expect(screen.getByText("AnotherUnused")).toBeInTheDocument();
    expect(screen.getByText("2")).toBeInTheDocument();
  });

  it("navigates_to_script_on_click", () => {
    vi.mocked(useOrphanScripts).mockReturnValue(
      { data: [mockOrphans[0]], isLoading: false } as unknown as ReturnType<typeof useOrphanScripts>
    );
    vi.mocked(useScriptList).mockReturnValue(
      { data: mockScripts } as unknown as ReturnType<typeof useScriptList>
    );
    render(<OrphanScriptsList projectId={1} />);
    fireEvent.click(screen.getByText("UnusedScript"));
    expect(mockSelectElement).toHaveBeenCalledWith({
      kind: "script",
      projectId: 1,
      id: 10,
      name: "UnusedScript",
    });
  });

  it("renders_null_when_no_projectId", () => {
    vi.mocked(useOrphanScripts).mockReturnValue(
      { data: undefined, isLoading: false } as unknown as ReturnType<typeof useOrphanScripts>
    );
    vi.mocked(useScriptList).mockReturnValue(
      { data: [] } as unknown as ReturnType<typeof useScriptList>
    );
    const { container } = render(<OrphanScriptsList projectId={null} />);
    expect(container.firstChild).toBeNull();
  });
});
