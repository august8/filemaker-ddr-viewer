import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { AllScriptsPanel } from "../../components/detail/AllScriptsPanel";
import { makeScriptRow } from "../testFixtures";

vi.mock("../../hooks/script", () => ({
  useScriptList: vi.fn(),
}));

vi.mock("../../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

import { useScriptList } from "../../hooks/script";
import { useAppStore } from "../../stores/appStore";

const PAGE_SIZE = 500;

const mockScripts = [
  makeScriptRow({ id: 1, fm_id: 1, name: "Save Record", step_count: 5 }),
  makeScriptRow({ id: 2, fm_id: 2, name: "Print Report", run_with_full_access: true, step_count: 12 }),
];

const fullPage = Array.from({ length: PAGE_SIZE }, (_, i) =>
  makeScriptRow({ id: i + 1, fm_id: i + 1, name: `Script${i}`, step_count: 0 })
);

const mockSelectElement = vi.fn();

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(useAppStore).mockReturnValue({
    selectElement: mockSelectElement,
  } as unknown as ReturnType<typeof useAppStore>);
});

describe("AllScriptsPanel", () => {
  it("row_click_navigates_to_script", () => {
    vi.mocked(useScriptList).mockReturnValue(
      { data: mockScripts, isLoading: false } as unknown as ReturnType<typeof useScriptList>
    );
    render(<AllScriptsPanel projectId={1} />);
    fireEvent.click(screen.getByText("Save Record"));
    expect(mockSelectElement).toHaveBeenCalledWith({
      kind: "script",
      projectId: 1,
      id: 1,
      name: "Save Record",
    });
  });

  it("prev_button_disabled_on_first_page", () => {
    vi.mocked(useScriptList).mockReturnValue(
      { data: mockScripts, isLoading: false } as unknown as ReturnType<typeof useScriptList>
    );
    render(<AllScriptsPanel projectId={1} />);
    expect(screen.getByRole("button", { name: /前/ })).toBeDisabled();
  });

  it("next_button_disabled_when_last_page", () => {
    vi.mocked(useScriptList).mockReturnValue(
      { data: mockScripts, isLoading: false } as unknown as ReturnType<typeof useScriptList>
    );
    render(<AllScriptsPanel projectId={1} />);
    expect(screen.getByRole("button", { name: /次/ })).toBeDisabled();
  });

  it("next_button_enabled_when_full_page", () => {
    vi.mocked(useScriptList).mockReturnValue(
      { data: fullPage, isLoading: false } as unknown as ReturnType<typeof useScriptList>
    );
    render(<AllScriptsPanel projectId={1} />);
    expect(screen.getByRole("button", { name: /次/ })).not.toBeDisabled();
  });

  it("filter_resets_page_to_zero", () => {
    vi.mocked(useScriptList).mockReturnValue(
      { data: fullPage, isLoading: false } as unknown as ReturnType<typeof useScriptList>
    );
    render(<AllScriptsPanel projectId={1} />);
    fireEvent.click(screen.getByRole("button", { name: /次/ }));
    fireEvent.change(screen.getByPlaceholderText(/絞り込み/), { target: { value: "x" } });
    const calls = vi.mocked(useScriptList).mock.calls;
    expect(calls[calls.length - 1][2]).toBe(0);
  });
});
