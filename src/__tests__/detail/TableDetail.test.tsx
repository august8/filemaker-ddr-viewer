import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { TableDetail } from "../../components/detail/TableDetail";
import { makeFieldRow } from "../testFixtures";

vi.mock("../../hooks/useTauriCommand", () => ({
  useTableFields: vi.fn(),
  useTableList: vi.fn(() => ({ data: [], isLoading: false })),
}));

vi.mock("../../stores/appStore", () => ({
  useAppStore: vi.fn(() => ({
    setRightPanel: vi.fn(),
    rightPanel: null,
    diffContext: null,
  })),
}));

import { useTableFields } from "../../hooks/useTauriCommand";

const mockFields = [
  makeFieldRow({ id: 1, fm_id: 1, name: "FirstName", data_type: "Text", comment: "First name" }),
  makeFieldRow({ id: 2, fm_id: 2, name: "GlobalFlag", data_type: "Number", is_global: true }),
];

beforeEach(() => {
  vi.clearAllMocks();
});

describe("TableDetail", () => {
  it("renders_field_list", () => {
    vi.mocked(useTableFields).mockReturnValue({
      data: mockFields,
      isLoading: false,
    } as unknown as ReturnType<typeof useTableFields>);
    render(<TableDetail projectId={1} tableId={1} name="Contact" />);
    expect(screen.getByText("FirstName")).toBeInTheDocument();
    expect(screen.getByText("Text")).toBeInTheDocument();
    expect(screen.getByText("GlobalFlag")).toBeInTheDocument();
  });

  it("renders_global_badge", () => {
    vi.mocked(useTableFields).mockReturnValue({
      data: mockFields,
      isLoading: false,
    } as unknown as ReturnType<typeof useTableFields>);
    render(<TableDetail projectId={1} tableId={1} name="Contact" />);
    const badges = screen.getAllByText("G");
    expect(badges.length).toBeGreaterThan(0);
  });

  it("renders_empty_when_no_fields", () => {
    vi.mocked(useTableFields).mockReturnValue({
      data: [],
      isLoading: false,
    } as unknown as ReturnType<typeof useTableFields>);
    render(<TableDetail projectId={1} tableId={1} name="Contact" />);
    expect(screen.getByText("フィールドなし")).toBeInTheDocument();
  });
});
