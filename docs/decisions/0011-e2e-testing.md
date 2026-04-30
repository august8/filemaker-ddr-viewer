# ADR 0011: E2E テスト基盤の技術選定

- **Status**: Accepted
- **Date**: 2026-04-28

## Context

現状のテストカバレッジ:
- Rust 単体テスト（各モジュール内 `#[cfg(test)]`）
- Rust 統合テスト（`src-tauri/tests/integration_ddr.rs`）
- Vitest コンポーネントテスト 36 ファイル（Tauri IPC モック経由）

Tauri アプリ全体のエンドツーエンド（実 WebView + 実 IPC + 実 SQLite）の統合テストは存在せず、
リリース前の UI 回帰検知が手動目視に依存している。

Issue #20 の要件:
- ゴールデンパス 1 本（DDR インポート → 検索 → 詳細表示）を自動化する
- `npm run test:e2e` で実行できる
- ローカル実行手順を `ARCHITECTURE.md` に記載する

## 候補技術の比較

### 候補 1: Playwright + WebView2 CDP 直接接続 ✅（最終採用）

Playwright は CDP ネイティブ。WebView2 は `--remote-debugging-port` を渡すことで
CDP エンドポイントを公開する。`chromium.connectOverCDP()` で接続可能。
tauri-driver・msedgedriver が不要でセットアップが単純。

### 候補 2: WebdriverIO + tauri-driver（当初採用・後に変更）

WebdriverIO は WebDriver プロトコルネイティブ。tauri-driver + msedgedriver との
公式ペアリングで、Tauri 公式ドキュメントに手順が記載されている。

**変更理由**: Windows + WebView2 環境で `waitForDisplayed()` が誤動作する問題が発覚。
`isDisplayed()` が要素の存在を正しく検出できず、精度が低い workaround が必要になった。
Playwright + CDP に切り替えることで `toBeVisible()` が正しく機能することを確認。

### 候補 3: Vite プレビュー + Playwright + IPC モック

デスクトップアプリの起動を諦め、Vite の dev server に対して Playwright を実行する軽量案。

**問題点**: Rust 層（XML パース・FTS5 全文検索・SQLite インポートパイプライン）をモックアウトするため、
既存 Vitest コンポーネントテストと重複する。E2E として最も重要な「実 IPC ラウンドトリップ」が
検証できない。

## 評価マトリクス

| 評価軸 | **Playwright + CDP** | WebdriverIO + tauri-driver | Vite + Playwright + モック |
|---|---|---|---|
| 実 Rust IPC を検証する | ✅ | ✅ | ❌（モック） |
| プロトコル互換性（Windows） | ✅（CDP ネイティブ） | △（WD 経由、DPI 問題あり） | ✅ |
| 要素可視性の信頼性 | ✅（`toBeVisible()` 実績） | △（`isDisplayed()` 誤動作） | ✅ |
| 追加バイナリ不要 | ✅ | ❌（tauri-driver + msedgedriver） | ✅ |
| TypeScript サポート | ✅ | ✅ | ✅ |
| テストの価値 | **高** | 高 | 低 |

## Decision

**Playwright + WebView2 CDP 直接接続を採用する。**

Rust バックエンド（XML パース・FTS5・SQLite）は既存の Vitest テストでは一切検証されない。
真の E2E 価値は実バイナリ・実 IPC・実 SQLite を通じた統合テストにある。
当初は WebdriverIO + tauri-driver で実装したが、Windows + WebView2 環境での
`isDisplayed()` 誤動作により Playwright + CDP に移行した。
WebView2 は `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222` で
CDP を公開し、Playwright の `chromium.connectOverCDP()` で接続する。

## ファイルダイアログ問題の解決

DDR インポートは `@tauri-apps/plugin-dialog` のネイティブファイルピッカーを使うため
WebDriver からは操作できない。Cargo feature フラグ `test-utils` でテスト専用 IPC コマンドを追加する:

```rust
// src-tauri/src/commands/test_utils.rs
#[tauri::command]
pub async fn import_ddr_from_path(
    state: tauri::State<'_, AppState>,
    summary_path: String,
) -> Result<SolutionWithProjects, CommandError> {
    super::import::import_solution(state, summary_path).await
}
```

- `Cargo.toml` に `[features] test-utils = []` を追加
- `commands/mod.rs` で `#[cfg(feature = "test-utils")] pub mod test_utils;`
- `lib.rs` の `generate_handler!` で `#[cfg(feature = "test-utils")] commands::test_utils::import_ddr_from_path`
- ビルド時: `npx tauri build --features test-utils`
- リリースバイナリ（feature なし）には一切含まれない

## CI への追加（今フェーズは対象外）

Tauri バイナリのビルドには約 10 分かかる。`tauri-driver` + `msedgedriver` のセットアップも
GitHub Actions 上で未検証のため、今フェーズはローカル実行のみとする。

CI に追加する場合の手順は `.github/workflows/ci.yml` にコメントで記載する。

## 関連ファイル

- `src-tauri/Cargo.toml`（test-utils feature）
- `src-tauri/src/main.rs`（CDP env var）
- `src-tauri/src/commands/test_utils.rs`
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/lib.rs`
- `playwright.config.ts`
- `tests/e2e/global-setup.ts`（アプリ起動・CDP 待機）
- `tests/e2e/global-teardown.ts`（アプリ終了）
- `tests/e2e/fixtures.ts`（CDP 接続 fixture）
- `tests/e2e/golden-path.spec.ts`
- `ARCHITECTURE.md`
