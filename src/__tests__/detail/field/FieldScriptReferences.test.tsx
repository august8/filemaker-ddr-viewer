import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { FieldScriptReferences } from "../../../components/detail/field/FieldScriptReferences";

vi.mock("../../../hooks/fieldRefs", () => ({
  useFieldRefs: vi.fn(),
}));
vi.mock("../../../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

import { useFieldRefs } from "../../../hooks/fieldRefs";
import { useAppStore } from "../../../stores/appStore";

const mockSelectElement = vi.fn();

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useAppStore).mockReturnValue({
    selectElement: mockSelectElement,
  } as unknown as ReturnType<typeof useAppStore>);
});

describe("FieldScriptReferences", () => {
  it("shows_empty_message_when_no_refs", () => {
    vi.mocked(useFieldRefs).mockReturnValue({
      data: [],
      isLoading: false,
    } as unknown as ReturnType<typeof useFieldRefs>);
    render(<FieldScriptReferences projectId={1} tableName="T" fieldName="F" />);
    expect(screen.getByText(/このフィールドを参照するスクリプトはありません/)).toBeInTheDocument();
  });

  it("script_ref_uses_ref_project_id_for_navigation", () => {
    vi.mocked(useFieldRefs).mockReturnValue({
      data: [{ script_id: 7, script_name: "MyScript", project_id: 99 }],
      isLoading: false,
    } as unknown as ReturnType<typeof useFieldRefs>);
    render(<FieldScriptReferences projectId={1} tableName="T" fieldName="F" />);
    fireEvent.click(screen.getByRole("button", { name: "MyScript" }));
    expect(mockSelectElement).toHaveBeenCalledWith({
      kind: "script",
      projectId: 99,
      id: 7,
      name: "MyScript",
    });
  });
});
