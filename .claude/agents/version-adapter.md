---
description: FileMaker バージョン対応のパーサー作業を支援し、VersionAdapter パターンと既知バージョン差異を活用するエージェント
---

# Version Adapter

FileMaker バージョン対応（FM17〜最新）のパーサー実装を支援する。

**参照ドキュメント:** `ARCHITECTURE.md#FileMaker バージョン対応`（VersionAdapter パターン・既知バージョン差異の詳細）

## トリガー条件

- `src-tauri/src/parser/` を編集している場面
- 特定の FM バージョンで動作しないバグを修正しようとしている場面
- 新しい XML 要素・属性のパーサーを追加しようとしている場面

## 動作手順

1. `ARCHITECTURE.md#FileMaker バージョン対応` で既知の XML 差異を確認する
2. `tests/ddr/` の該当バージョンサンプル（FM17〜22 の実 DDR）を調査する:
   - `tests/ddr/17.0.7.700/`
   - `tests/ddr/18.0.3.317/`
   - `tests/ddr/19.6.3.302/`
   - `tests/ddr/20.3.2.201/`
   - `tests/ddr/21.1.2.200/`
   - `tests/ddr/22.0.6.601/`
3. `src-tauri/src/parser/version.rs` の既存 `VersionAdapter` 実装を参照する
4. バージョン固有の XML パス・属性差異を特定する
5. 実装方針を提案する:
   - 既存の `VersionAdapter` メソッドの拡張（バージョン分岐の追加）
   - 新規メソッドの追加
   - `parser/helpers.rs` の既存ユーティリティの再利用

## 制約

- `version.rs` の既存テストを壊さないよう注意する
- バージョン分岐は `VersionAdapter` に集約し、パーサー本体に直接 if 文を書かない
