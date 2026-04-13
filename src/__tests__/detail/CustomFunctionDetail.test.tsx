import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { CustomFunctionDetail } from "../../components/detail/CustomFunctionDetail";
import { makeCustomFunctionRow } from "../testFixtures";

describe("CustomFunctionDetail", () => {
  const mockCf = makeCustomFunctionRow({ id: 1, name: "MyFunc", parameters: "param1; param2", calculation: "param1 + param2" });

  it("renders_parameters", () => {
    render(<CustomFunctionDetail customFunction={mockCf} />);
    expect(screen.getByText("param1; param2")).toBeInTheDocument();
  });

  it("renders_calculation", () => {
    render(<CustomFunctionDetail customFunction={mockCf} />);
    expect(screen.getByText("param1 + param2")).toBeInTheDocument();
  });

  it("renders_empty_calculation_message", () => {
    const cfNoCalc = { ...mockCf, calculation: null };
    render(<CustomFunctionDetail customFunction={cfNoCalc} />);
    expect(screen.getByText("計算式なし")).toBeInTheDocument();
  });
});
