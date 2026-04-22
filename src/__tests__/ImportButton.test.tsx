import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ImportButton } from "../components/ImportButton";

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("../hooks/solutions", () => ({
  useImportSolution: vi.fn(),
}));

import { useImportSolution } from "../hooks/solutions";

function wrapper({ children }: { children: React.ReactNode }) {
  const qc = new QueryClient();
  return <QueryClientProvider client={qc}>{children}</QueryClientProvider>;
}

beforeEach(() => {
  vi.mocked(useImportSolution).mockReturnValue(
    { mutate: vi.fn(), isPending: false } as unknown as ReturnType<typeof useImportSolution>
  );
});

describe("ImportButton", () => {
  it("renders_import_button", () => {
    render(<ImportButton />, { wrapper });
    expect(screen.getByText("DDR をインポート")).toBeInTheDocument();
  });

  it("shows_loading_state", () => {
    vi.mocked(useImportSolution).mockReturnValue(
      { mutate: vi.fn(), isPending: true } as unknown as ReturnType<typeof useImportSolution>
    );
    render(<ImportButton />, { wrapper });
    expect(screen.getByText("インポート中...")).toBeInTheDocument();
  });
});
