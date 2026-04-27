import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { AllCustomFunctionsPanel } from "../../components/detail/AllCustomFunctionsPanel";
import { makeCustomFunctionRow } from "../testFixtures";
import { PAGE_SIZE } from "../../constants";

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

const fullPage = Array.from({ length: PAGE_SIZE }, (_, i) =>
  makeCustomFunctionRow({ id: i + 1, fm_id: i + 1, name: `CF${i}`, parameters: "" })
);

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

  it("prev_button_disabled_on_first_page", () => {
    vi.mocked(useCustomFunctionList).mockReturnValue(
      { data: mockCFs, isLoading: false } as unknown as ReturnType<typeof useCustomFunctionList>
    );
    render(<AllCustomFunctionsPanel projectId={1} />);
    expect(screen.getByRole("button", { name: /前/ })).toBeDisabled();
  });

  it("next_button_disabled_when_last_page", () => {
    vi.mocked(useCustomFunctionList).mockReturnValue(
      { data: mockCFs, isLoading: false } as unknown as ReturnType<typeof useCustomFunctionList>
    );
    render(<AllCustomFunctionsPanel projectId={1} />);
    expect(screen.getByRole("button", { name: /次/ })).toBeDisabled();
  });

  it("next_click_increments_offset", () => {
    vi.mocked(useCustomFunctionList)
      .mockReturnValueOnce({ data: fullPage, isLoading: false } as unknown as ReturnType<typeof useCustomFunctionList>)
      .mockReturnValue({ data: mockCFs, isLoading: false } as unknown as ReturnType<typeof useCustomFunctionList>);
    render(<AllCustomFunctionsPanel projectId={1} />);
    fireEvent.click(screen.getByRole("button", { name: /次/ }));
    expect(vi.mocked(useCustomFunctionList).mock.lastCall?.[2]).toBe(PAGE_SIZE);
  });

  it("filter_resets_page_to_zero", () => {
    vi.mocked(useCustomFunctionList).mockReturnValue(
      { data: fullPage, isLoading: false } as unknown as ReturnType<typeof useCustomFunctionList>
    );
    render(<AllCustomFunctionsPanel projectId={1} />);
    fireEvent.click(screen.getByRole("button", { name: /次/ }));
    fireEvent.change(screen.getByPlaceholderText(/絞り込み/), { target: { value: "x" } });
    const calls = vi.mocked(useCustomFunctionList).mock.calls;
    expect(calls[calls.length - 1][2]).toBe(0);
  });
});
