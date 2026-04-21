import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { BrokenRefsList } from "../components/BrokenRefsList";
import type { BrokenRef } from "../types/ddr";

vi.mock("../hooks/analysis", () => ({
  useBrokenRefs: vi.fn(),
}));
vi.mock("../hooks/script", () => ({
  useScriptList: vi.fn(() => ({ data: [], isLoading: false })),
}));
vi.mock("../hooks/layout", () => ({
  useLayoutList: vi.fn(() => ({ data: [], isLoading: false })),
}));

vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(() => ({ selectElement: vi.fn() })),
}));

import { useBrokenRefs } from "../hooks/analysis";

const mockBrokenRefs: BrokenRef[] = [
  { kind: "performScript", source_name: "Script A", target_script_name: "Missing Script" },
  { kind: "scriptTrigger", source_name: "Layout B", target_script_name: "Gone Script" },
];

describe("BrokenRefsList", () => {
  it("renders_empty_message_when_no_broken_refs", () => {
    vi.mocked(useBrokenRefs).mockReturnValue(
      { data: [], isLoading: false } as unknown as ReturnType<typeof useBrokenRefs>
    );
    render(<BrokenRefsList projectId={1} />);
    expect(screen.getByText("壊れた参照はありません")).toBeInTheDocument();
  });

  it("renders_broken_ref_names", () => {
    vi.mocked(useBrokenRefs).mockReturnValue(
      { data: mockBrokenRefs, isLoading: false } as unknown as ReturnType<typeof useBrokenRefs>
    );
    render(<BrokenRefsList projectId={1} />);
    expect(screen.getByText("Script A")).toBeInTheDocument();
    expect(screen.getByText("Missing Script")).toBeInTheDocument();
    expect(screen.getByText("Layout B")).toBeInTheDocument();
    expect(screen.getByText("Gone Script")).toBeInTheDocument();
  });
});
