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

### 候補 1: Playwright + tauri-driver

Playwright は Chrome DevTools Protocol（CDP）ネイティブのブラウザ自動化ツール。
tauri-driver は WebDriver プロトコルを話す。

**問題点**: プロトコルの不一致。
Playwright の `connectOverCDP()` は CDP エンドポイントを必要とするが、
tauri-driver が直接公開するのは WebDriver エンドポイントである。
WebView2 の CDP ポートを手動で抽出する追加実装が必要になり、
公式ドキュメントが存在しない。

### 候補 2: WebdriverIO + tauri-driver ✅（採用）

WebdriverIO は WebDriver プロトコルネイティブ。tauri-driver + msedgedriver との
公式ペアリングで、Tauri 公式ドキュメントに手順が記載されている。
`@wdio/tauri-service` パッケージが tauri-driver の起動・終了ライフサイクルを管理する。
TypeScript サポートがある。

### 候補 3: Vite プレビュー + Playwright + IPC モック

デスクトップアプリの起動を諦め、Vite の dev server に対して Playwright を実行する軽量案。

**問題点**: Rust 層（XML パース・FTS5 全文検索・SQLite インポートパイプライン）をモックアウトするため、
既存 Vitest コンポーネントテストと重複する。E2E として最も重要な「実 IPC ラウンドトリップ」が
検証できない。

## 評価マトリクス

| 評価軸 | Playwright + tauri-driver | **WebdriverIO + tauri-driver** | Vite + Playwright + モック |
|---|---|---|---|
| 実 Rust IPC を検証する | ✅ | ✅ | ❌（モック） |
| プロトコル互換性 | ❌（CDP/WD 不一致） | ✅（WD ネイティブ） | ✅ |
| 公式ドキュメント | ❌ | ✅ | N/A |
| TypeScript サポート | ✅ | ✅ | ✅ |
| CI 実行可能性 | △（要ブリッジ） | △（ビルド時間） | ✅（高速） |
| テストの価値 | 高 | **高** | 低 |

## Decision

**WebdriverIO + tauri-driver を採用する。**

Rust バックエンド（XML パース・FTS5・SQLite）は既存の Vitest テストでは一切検証されない。
真の E2E 価値は実バイナリ・実 IPC・実 SQLite を通じた統合テストにある。
WebdriverIO はプロトコル互換性・公式ドキュメント・エコシステム品質のすべての点で
tauri-driver との組み合わせに最適である。

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

- `src-tauri/Cargo.toml`
- `src-tauri/src/commands/test_utils.rs`（新規）
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/lib.rs`
- `wdio.conf.ts`（新規）
- `tests/e2e/golden-path.spec.ts`（新規）
- `ARCHITECTURE.md`
