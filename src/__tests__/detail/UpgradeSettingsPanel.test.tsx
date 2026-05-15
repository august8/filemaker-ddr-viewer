import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { UpgradeSettingsPanel } from "../../components/detail/UpgradeSettingsPanel";

vi.mock("../../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

import { useAppStore } from "../../stores/appStore";

const baseStore = {
  checkItems: [],
  setCheckItems: vi.fn(),
  showBrokenRefsInUpgradeCheck: true,
  setShowBrokenRefsInUpgradeCheck: vi.fn(),
};

describe("UpgradeSettingsPanel - 解析セクション", () => {
  it("broken_refs_toggle_is_checked_when_enabled", () => {
    vi.mocked(useAppStore).mockImplementation(
      () => ({ ...baseStore } as unknown as ReturnType<typeof useAppStore>)
    );
    render(<UpgradeSettingsPanel onClose={vi.fn()} />);
    const checkbox = screen.getByRole("checkbox", { name: /壊れた参照/ });
    expect(checkbox).toBeChecked();
  });

  it("broken_refs_toggle_calls_setter_with_false", async () => {
    const setShowBrokenRefsInUpgradeCheck = vi.fn();
    vi.mocked(useAppStore).mockImplementation(
      () => ({ ...baseStore, setShowBrokenRefsInUpgradeCheck } as unknown as ReturnType<typeof useAppStore>)
    );
    const user = userEvent.setup();
    render(<UpgradeSettingsPanel onClose={vi.fn()} />);
    await user.click(screen.getByRole("checkbox", { name: /壊れた参照/ }));
    expect(setShowBrokenRefsInUpgradeCheck).toHaveBeenCalledWith(false);
  });

  it("broken_refs_toggle_is_unchecked_when_disabled", () => {
    vi.mocked(useAppStore).mockImplementation(
      () => ({ ...baseStore, showBrokenRefsInUpgradeCheck: false } as unknown as ReturnType<typeof useAppStore>)
    );
    render(<UpgradeSettingsPanel onClose={vi.fn()} />);
    const checkbox = screen.getByRole("checkbox", { name: /壊れた参照/ });
    expect(checkbox).not.toBeChecked();
  });
});
