import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { SecurityPanel } from "../components/detail/SecurityPanel";
import type { AccountRow, PrivilegeSetRow } from "../types/ddr";

vi.mock("../hooks/useTauriCommand", () => ({
  useAccountList: vi.fn(),
  usePrivilegeSetList: vi.fn(),
}));

import { useAccountList, usePrivilegeSetList } from "../hooks/useTauriCommand";

const mockAccounts: AccountRow[] = [
  { id: 1, fm_id: 1, name: "Admin", privilege_set: "[Full Access]", enabled: true },
  { id: 2, fm_id: 2, name: "Guest", privilege_set: "[Read-Only Access]", enabled: false },
];

const mockPrivilegeSets: PrivilegeSetRow[] = [
  { id: 1, fm_id: 1, name: "[Full Access]", comment: "Full access" },
  { id: 2, fm_id: 2, name: "[Read-Only Access]", comment: null },
];

beforeEach(() => {
  vi.clearAllMocks();
});

describe("SecurityPanel", () => {
  it("renders_loading_state", () => {
    vi.mocked(useAccountList).mockReturnValue(
      { data: undefined, isLoading: true } as unknown as ReturnType<typeof useAccountList>
    );
    vi.mocked(usePrivilegeSetList).mockReturnValue(
      { data: undefined, isLoading: true } as unknown as ReturnType<typeof usePrivilegeSetList>
    );
    render(<SecurityPanel projectId={1} />);
    expect(screen.getByText(/読み込み中/)).toBeInTheDocument();
  });

  it("renders_account_list", () => {
    vi.mocked(useAccountList).mockReturnValue(
      { data: mockAccounts, isLoading: false } as unknown as ReturnType<typeof useAccountList>
    );
    vi.mocked(usePrivilegeSetList).mockReturnValue(
      { data: mockPrivilegeSets, isLoading: false } as unknown as ReturnType<typeof usePrivilegeSetList>
    );
    render(<SecurityPanel projectId={1} />);
    expect(screen.getByText("Admin")).toBeInTheDocument();
    expect(screen.getByText("Guest")).toBeInTheDocument();
    // privilege_set はアカウント行と権限セット行の両方に表示されるため getAllByText を使用
    expect(screen.getAllByText("[Full Access]").length).toBeGreaterThan(0);
  });

  it("renders_enabled_disabled_status", () => {
    vi.mocked(useAccountList).mockReturnValue(
      { data: mockAccounts, isLoading: false } as unknown as ReturnType<typeof useAccountList>
    );
    vi.mocked(usePrivilegeSetList).mockReturnValue(
      { data: mockPrivilegeSets, isLoading: false } as unknown as ReturnType<typeof usePrivilegeSetList>
    );
    render(<SecurityPanel projectId={1} />);
    // 有効・無効の表示
    expect(screen.getAllByText(/有効|無効/).length).toBeGreaterThan(0);
  });

  it("renders_privilege_set_list", () => {
    vi.mocked(useAccountList).mockReturnValue(
      { data: mockAccounts, isLoading: false } as unknown as ReturnType<typeof useAccountList>
    );
    vi.mocked(usePrivilegeSetList).mockReturnValue(
      { data: mockPrivilegeSets, isLoading: false } as unknown as ReturnType<typeof usePrivilegeSetList>
    );
    render(<SecurityPanel projectId={1} />);
    // privilege_set はアカウント行と権限セット行の両方に表示されるため getAllByText を使用
    expect(screen.getAllByText("[Read-Only Access]").length).toBeGreaterThan(0);
    expect(screen.getByText("Full access")).toBeInTheDocument();
  });

  it("renders_empty_state_when_no_accounts", () => {
    vi.mocked(useAccountList).mockReturnValue(
      { data: [], isLoading: false } as unknown as ReturnType<typeof useAccountList>
    );
    vi.mocked(usePrivilegeSetList).mockReturnValue(
      { data: [], isLoading: false } as unknown as ReturnType<typeof usePrivilegeSetList>
    );
    render(<SecurityPanel projectId={1} />);
    expect(screen.getByText(/アカウントなし/)).toBeInTheDocument();
    expect(screen.getByText(/権限セットなし/)).toBeInTheDocument();
  });
});
