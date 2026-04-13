import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { ValueListDetail } from "../../components/detail/ValueListDetail";
import { makeValueListRow } from "../testFixtures";

vi.mock("../../hooks/useTauriCommand", () => ({
  useValueListItems: vi.fn(),
}));

import { useValueListItems } from "../../hooks/useTauriCommand";

const mockValueList = makeValueListRow({ id: 1, name: "Status", item_count: 2 });
const mockFieldValueList = makeValueListRow({ id: 2, fm_id: 2, name: "Projects", source: "Field" });

beforeEach(() => {
  vi.clearAllMocks();
});

describe("ValueListDetail", () => {
  it("renders_source_type", () => {
    vi.mocked(useValueListItems).mockReturnValue({
      data: ["Active", "Inactive"],
      isLoading: false,
    } as unknown as ReturnType<typeof useValueListItems>);
    render(<ValueListDetail valueList={mockValueList} />);
    expect(screen.getByText("Custom")).toBeInTheDocument();
  });

  it("renders_custom_values", () => {
    vi.mocked(useValueListItems).mockReturnValue({
      data: ["Active", "Inactive"],
      isLoading: false,
    } as unknown as ReturnType<typeof useValueListItems>);
    render(<ValueListDetail valueList={mockValueList} />);
    expect(screen.getByText("Active")).toBeInTheDocument();
    expect(screen.getByText("Inactive")).toBeInTheDocument();
  });

  it("renders_field_source_without_items", () => {
    vi.mocked(useValueListItems).mockReturnValue({
      data: [],
      isLoading: false,
    } as unknown as ReturnType<typeof useValueListItems>);
    render(<ValueListDetail valueList={mockFieldValueList} />);
    expect(screen.getByText("Field")).toBeInTheDocument();
  });
});
