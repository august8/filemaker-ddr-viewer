import { describe, expect, it, beforeEach } from "vitest";
import { render, screen, fireEvent, act, waitFor } from "@testing-library/react";
import { SearchBar } from "../components/SearchBar";
import { useAppStore } from "../stores/appStore";

beforeEach(() => {
  useAppStore.setState({ searchQuery: "" });
});

describe("SearchBar", () => {
  it("renders_search_input", () => {
    render(<SearchBar />);
    expect(screen.getByRole("textbox")).toBeInTheDocument();
  });

  it("clear_button_appears_when_query_not_empty", () => {
    render(<SearchBar />);
    const input = screen.getByRole("textbox");
    fireEvent.change(input, { target: { value: "test" } });
    expect(screen.getByRole("button", { name: "クリア" })).toBeInTheDocument();
  });

  it("clear_button_resets_query", () => {
    render(<SearchBar />);
    const input = screen.getByRole("textbox");
    fireEvent.change(input, { target: { value: "hello" } });
    const clearButton = screen.getByRole("button", { name: "クリア" });
    fireEvent.click(clearButton);
    expect(input).toHaveValue("");
  });

  it("input_clears_when_searchQuery_is_externally_reset", async () => {
    // selectElement() が searchQuery を "" にクリアしたとき、
    // SearchBar の input も "" に同期されることを確認する（debounce 復元バグの回帰テスト）
    //
    // 実際のシナリオ:
    //   1. debounce が発火して searchQuery = "foo" に設定済み
    //   2. ユーザーが検索結果をクリック → selectElement が searchQuery = "" にクリア
    //   3. SearchBar の input も "" に同期される（これがないと 300ms 後に "foo" が復元される）

    // Step 1: searchQuery = "foo" で描画（debounce 発火後を模倣）
    useAppStore.setState({ searchQuery: "foo" });
    render(<SearchBar />);
    const input = screen.getByRole("textbox");
    expect(input).toHaveValue("foo");

    // Step 2: 外部から searchQuery がクリアされる（selectElement による遷移を模倣）
    act(() => {
      useAppStore.setState({ searchQuery: "" });
    });

    // Step 3: input が "" に同期されること（sync effect が localValue を更新）
    await waitFor(() => expect(input).toHaveValue(""));
  });
});
