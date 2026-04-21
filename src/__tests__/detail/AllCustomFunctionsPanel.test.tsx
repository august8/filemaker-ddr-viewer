import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { AllCustomFunctionsPanel } from "../../components/detail/AllCustomFunctionsPanel";
import { makeCustomFunctionRow } from "../testFixtures";

vi.mock("../../hooks/catalog", () => ({
  useCustomFunctionList: vi.fn(),
}));

vi.mock("../../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

import { useCustomFunctionList } from "../../hooks/catalog";
import { useAppStore } from "../../stores/appStore";

const mockCFs = [
  makeCustomFunctionRow({ id: 1, fm_id: 1, name: "FormatDate", parameters: "date; format" }),
  makeCustomFunctionRow({ id: 2, fm_id: 2, name: "IsEmpty" }),
];

const mockSelectElement = vi.fn();

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useAppStore).mockReturnValue({
    selectElement: mockSelectElement,
  } as unknown as ReturnType<typeof useAppStore>);
});

describe("AllCustomFunctionsPanel", () => {
  it("row_click_navigates_to_custom_function", () => {
    vi.mocked(useCustomFunctionList).mockReturnValue(
      { data: mockCFs, isLoading: false } as unknown as ReturnType<typeof useCustomFunctionList>
    );
    render(<AllCustomFunctionsPanel projectId={1} />);
    fireEvent.click(screen.getByText("FormatDate"));
    expect(mockSelectElement).toHaveBeenCalledWith({
      kind: "custom_function",
      projectId: 1,
      id: 1,
      name: "FormatDate",
    });
  });
});
