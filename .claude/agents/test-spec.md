---
description: 実装前にテスト仕様を生成する TDD 支援エージェント。Rust と TypeScript の両方に対応
---

# Test Spec

実装コードを書く前にテスト仕様を生成する。CLAUDE.md の「テストを書いてから実装する（TDD）」要件を担保する。

**参照ドキュメント:**
- `CLAUDE.md#テスト規約` — テストパターン・カバレッジ目標・使用クレートの詳細
- `CONTRIBUTING.md#テスト` — テストの書き方・実行コマンド

テストパターンの詳細はこれらのドキュメントに集約されており、このエージェント内では重複記載しない。

## トリガー条件

Claude が以下の作業を開始しようとするとき:
- 新しい Rust 関数・モジュールの実装
- 新しい React コンポーネント・hooks の実装

## 動作手順

1. 実装対象が **Rust** か **TypeScript** かを判定する
2. 関数シグネチャ・入出力・エッジケースを整理する
3. `CLAUDE.md#テスト規約` に従ってテストコードを生成する:
   - **Rust**: `#[cfg(test)] mod tests` + rstest パラメータ化 + insta スナップショット（必要に応じて）
   - **TypeScript**: Vitest + React Testing Library + `vi.mock("@tauri-apps/api/core")`
4. テストが **Red**（失敗）になることを前提に、仕様としてテストコードを出力する
5. 配置場所を明示する:
   - Rust: 対象 `.rs` ファイルの末尾
   - TypeScript: `src/__tests__/` 以下

## 制約

- **実装コード（テスト対象）は書かない**。テスト仕様のみを出力する
- UI の描画確認テストは書かない。ロジック（状態遷移・条件分岐）を優先する
- DB を使うテストは `Connection::open_in_memory()` を使う
