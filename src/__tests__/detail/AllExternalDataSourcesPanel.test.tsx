import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { AllExternalDataSourcesPanel } from "../../components/detail/AllExternalDataSourcesPanel";
import { makeExternalDataSourceRow } from "../testFixtures";
import { PAGE_SIZE } from "../../constants";

vi.mock("../../hooks/catalog", () => ({
  useExternalDataSourceList: vi.fn(),
}));

import { useExternalDataSourceList } from "../../hooks/catalog";

const mockSources = [
  makeExternalDataSourceRow({ id: 1, fm_id: 2, name: "BaseFile", path_list: "file:BaseFile", link: "BaseFile_fmp12.xml" }),
  makeExternalDataSourceRow({ id: 2, fm_id: 3, name: "ExternalFile", path_list: "file:ExternalFile", link: "ExternalFile_fmp12.xml" }),
];

const fullPage = Array.from({ length: PAGE_SIZE }, (_, i) =>
  makeExternalDataSourceRow({ id: i + 1, fm_id: i + 2, name: `File${i}`, path_list: `file:File${i}`, link: `File${i}.xml` })
);

beforeEach(() => {
  vi.clearAllMocks();
});

describe("AllExternalDataSourcesPanel", () => {
  it("shows_list_of_external_sources", () => {
    vi.mocked(useExternalDataSourceList).mockReturnValue(
      { data: mockSources, isLoading: false } as unknown as ReturnType<typeof useExternalDataSourceList>
    );
    render(<AllExternalDataSourcesPanel projectId={1} />);
    expect(screen.getByText("BaseFile")).toBeInTheDocument();
    expect(screen.getByText("ExternalFile")).toBeInTheDocument();
    expect(screen.getByText("file:BaseFile")).toBeInTheDocument();
  });

  it("shows_empty_message_when_no_sources", () => {
    vi.mocked(useExternalDataSourceList).mockReturnValue(
      { data: [], isLoading: false } as unknown as ReturnType<typeof useExternalDataSourceList>
    );
    render(<AllExternalDataSourcesPanel projectId={1} />);
    expect(screen.getByText("外部データソースなし")).toBeInTheDocument();
  });

  it("shows_loading_spinner", () => {
    vi.mocked(useExternalDataSourceList).mockReturnValue(
      { data: undefined, isLoading: true } as unknown as ReturnType<typeof useExternalDataSourceList>
    );
    render(<AllExternalDataSourcesPanel projectId={1} />);
    expect(screen.getByText("読み込み中...")).toBeInTheDocument();
  });

  it("filter_narrows_results_by_name", () => {
    vi.mocked(useExternalDataSourceList).mockReturnValue(
      { data: mockSources, isLoading: false } as unknown as ReturnType<typeof useExternalDataSourceList>
    );
    render(<AllExternalDataSourcesPanel projectId={1} />);
    fireEvent.change(screen.getByPlaceholderText(/絞り込み/), { target: { value: "External" } });
    expect(screen.getByText("ExternalFile")).toBeInTheDocument();
    expect(screen.queryByText("BaseFile")).not.toBeInTheDocument();
  });

  it("prev_button_disabled_on_first_page", () => {
    vi.mocked(useExternalDataSourceList).mockReturnValue(
      { data: mockSources, isLoading: false } as unknown as ReturnType<typeof useExternalDataSourceList>
    );
    render(<AllExternalDataSourcesPanel projectId={1} />);
    expect(screen.getByRole("button", { name: /前/ })).toBeDisabled();
  });

  it("next_button_disabled_when_last_page", () => {
    vi.mocked(useExternalDataSourceList).mockReturnValue(
      { data: mockSources, isLoading: false } as unknown as ReturnType<typeof useExternalDataSourceList>
    );
    render(<AllExternalDataSourcesPanel projectId={1} />);
    expect(screen.getByRole("button", { name: /次/ })).toBeDisabled();
  });

  it("next_click_increments_offset", () => {
    vi.mocked(useExternalDataSourceList)
      .mockReturnValueOnce({ data: fullPage, isLoading: false } as unknown as ReturnType<typeof useExternalDataSourceList>)
      .mockReturnValue({ data: mockSources, isLoading: false } as unknown as ReturnType<typeof useExternalDataSourceList>);
    render(<AllExternalDataSourcesPanel projectId={1} />);
    fireEvent.click(screen.getByRole("button", { name: /次/ }));
    expect(vi.mocked(useExternalDataSourceList).mock.lastCall?.[2]).toBe(PAGE_SIZE);
  });
});
