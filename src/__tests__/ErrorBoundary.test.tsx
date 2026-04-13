import { describe, expect, it, vi, afterEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { ErrorBoundary } from "../components/ErrorBoundary";

// エラーを throw する子コンポーネント
const Bomb = () => {
  throw new Error("テストエラー");
};

// console.error を抑制（React が Error Boundary のエラーを出力するため）
const suppressConsoleError = () =>
  vi.spyOn(console, "error").mockImplementation(() => {});

afterEach(() => {
  vi.restoreAllMocks();
});

describe("ErrorBoundary", () => {
  it("shows_error_ui_when_child_throws", () => {
    suppressConsoleError();
    render(
      <ErrorBoundary>
        <Bomb />
      </ErrorBoundary>
    );
    expect(screen.getByText("エラーが発生しました")).toBeInTheDocument();
    expect(screen.getByText(/テストエラー/)).toBeInTheDocument();
  });

  it("shows_recovery_message", () => {
    suppressConsoleError();
    render(
      <ErrorBoundary>
        <Bomb />
      </ErrorBoundary>
    );
    expect(screen.getByText(/別の項目を選択すると自動的に回復/)).toBeInTheDocument();
  });

  it("shows_copy_button", () => {
    suppressConsoleError();
    render(
      <ErrorBoundary>
        <Bomb />
      </ErrorBoundary>
    );
    expect(screen.getByRole("button", { name: /コピー/ })).toBeInTheDocument();
  });

  it("renders_children_when_no_error", () => {
    render(
      <ErrorBoundary>
        <div>正常コンテンツ</div>
      </ErrorBoundary>
    );
    expect(screen.getByText("正常コンテンツ")).toBeInTheDocument();
    expect(screen.queryByText("エラーが発生しました")).not.toBeInTheDocument();
  });

  it("resets_when_resetKey_changes", () => {
    suppressConsoleError();
    const { rerender } = render(
      <ErrorBoundary resetKey="a">
        <Bomb />
      </ErrorBoundary>
    );
    expect(screen.getByText("エラーが発生しました")).toBeInTheDocument();

    // resetKey が変わると自動リセット
    rerender(
      <ErrorBoundary resetKey="b">
        <div>回復しました</div>
      </ErrorBoundary>
    );
    expect(screen.getByText("回復しました")).toBeInTheDocument();
    expect(screen.queryByText("エラーが発生しました")).not.toBeInTheDocument();
  });
});
