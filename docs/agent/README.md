# Agent Guide

この文書は Claude Code と Codex の両方で共有する、ツール非依存の開発ルールと作業フローです。
各ツール固有の入口は `AGENTS.md` と `CLAUDE.md` に置き、ツール固有の差分は `docs/agent/CODEX.md` と `docs/agent/CLAUDE.md` に置きます。

## プロジェクト概要

FileMaker DDR (Database Design Report) XML を解析・可視化する軽量デスクトップツール。
Tauri 2.x + Rust バックエンド + React フロントエンドで構成します。

OSS: https://github.com/august8/filemaker-ddr-viewer

## 技術スタック

| レイヤー | 技術 |
|---|---|
| デスクトップ | Tauri 2.x |
| バックエンド | Rust |
| フロントエンド | React 19 + TypeScript + Vite + TailwindCSS |
| XML 解析 | quick-xml + serde |
| 全文検索 | FTS5 |
| データ保存 | rusqlite (SQLite, bundled) |
| グラフ解析 | petgraph |
| フロントエンド可視化 | dagre + SVG |
| 差分表示 | diff2html |
| 状態管理 | zustand |
| サーバー状態 | @tanstack/react-query |

## 主要ディレクトリ

| パス | 役割 |
|---|---|
| `src-tauri/src/commands/` | Tauri IPC コマンド |
| `src-tauri/src/parser/` | DDR XML パーサー |
| `src-tauri/src/analyzer/` | 参照解析・差分・健全性レポート |
| `src-tauri/src/db/` | SQLite スキーマ・Repository |
| `src/components/` | React UI |
| `src/hooks/` | Tauri IPC ラッパーと query hooks |
| `src/types/ddr.ts` | フロントエンド型定義 |
| `tests/ddr/` | バージョン別 DDR サンプル |
| `tests/fixtures/` | 小さな単体テスト用 XML |
| `docs/decisions/` | ADR |
| `docs/agent/` | エージェント共通・ツール別運用ルール |

詳細な実装状況と制約は `ARCHITECTURE.md` を正本とします。

## 共通開発原則

- ユーザーとのやり取りは日本語で行う。
- コード、コマンド、ログ、エラーメッセージ、ファイル名、API 名は原文を維持する。
- 実装前に `ARCHITECTURE.md` を読み、記載内容を実コードで確認する。
- 既存パターンを優先し、不要な抽象化や広いリファクタリングを避ける。
- ユーザーや他ツールの未コミット変更を勝手に戻さない。
- ドキュメント修正を含むすべての変更は `main` から作業ブランチを切って行う。
- `main` への直接コミット・直接プッシュは禁止。

## セッション開始チェック

新しいセッションでは、実装・提案の前に次を確認します。

1. `ARCHITECTURE.md` を読む。
2. 記載された実装済み機能が実コードに存在するか確認する。
3. 型定義・関数シグネチャ・IPC 登録が記載と大きくずれていないか確認する。
4. `npm run test` を実行する。
5. Rust ファイルを触る予定がある場合、または全体状態を確認する場合は `cargo test` も実行する。

報告形式:

```text
### セッション開始チェック完了
- ARCHITECTURE.md: 確認済み（最終更新: YYYY-MM-DD）
- テスト: X passed / X failed
- 気になった点: なし
```

## 実装前確認

バグ修正、レビュー指摘対応、小さなリファクタリングでも、実装に入る前に「何をどう変えるか」を 1〜2 文でユーザーに伝えて確認します。

例:

```text
`useProjectSummary` からカウントを取得することで 0 件表示を解消します。よいですか？
```

## ブランチ

- ファイル編集前に `main` から作業ブランチを切る。
- ブランチ名は `feat/<slug>`、`fix/<slug>`、`refactor/<slug>`、`docs/<slug>` など変更内容に合わせる。
- `<slug>` は英小文字・数字・ハイフンで、変更内容が分かる短い名前にする。
- 環境上スラッシュ付きブランチ名が作れない場合は、理由を報告して `docs-...` のような衝突しない名前を使う。
- 既存の未コミット変更はユーザーまたは他ツール由来として扱い、勝手に戻さない。

## コミット

コミットメッセージは Conventional Commits 形式を使い、description は日本語にします。
英語のコミットメッセージは、ユーザーが明示した場合のみ使います。

形式:

```text
type(scope): 日本語の説明

必要に応じて本文
```

ルール:

- `type` は英語のまま使う。
- 使用できる `type`: `feat` / `fix` / `refactor` / `perf` / `docs` / `style` / `test` / `chore` / `build` / `ci` / `revert`
- `scope` は任意。例: `parser`, `search`, `db`, `ui`, `agent`
- description は自然な日本語で 50 文字以内を目安にする。
- 複数の `type` にまたがる変更は、コミット分割を提案してからメッセージを作る。
- コミット前に `git diff --cached` を確認し、ステージ済みの変更だけを対象にする。

例:

```text
docs(agent): Codex と Claude の運用ルールを整理
```

## TDD

機能追加・挙動変更では、テストを先に追加してから実装します。

1. テスト追加（Red 確認）
2. 実装（Green 確認）
3. 必要なリファクタリング
4. 関連ドキュメント更新

ドキュメントのみの変更は TDD 対象外です。

## テスト仕様

機能追加・挙動変更では、実装前にテスト観点を整理します。

- Rust: 対象 `.rs` ファイル末尾の `#[cfg(test)] mod tests`、または `src-tauri/tests/`
- TypeScript / React: `src/__tests__/` 以下
- DB を使う Rust テストは `Connection::open_in_memory()` を使う。
- UI の大量 DOM レンダーより、hook の呼び出し引数や状態遷移の検証を優先する。
- テストは Red になることを確認してから実装へ進む。

## 進捗報告

タスクリストの各ステップ完了時に、次の形式で報告します。

```text
✅ ステップ名 完了
```

## Rust 規約

- エラー処理は `thiserror` の独自エラー型を使う。
- 本番コードで `unwrap()` は使わない。テストのみ許可。
- Rust 標準の命名規則を守る。
- 新しいモジュールを追加したら `mod.rs` に公開インターフェースを追加する。
- フロントエンドへ渡す型は `Serialize` / `Deserialize` を derive する。
- `cargo fmt --check` と `cargo clippy -- -D warnings` を通す。

## TypeScript / React 規約

- `strict: true` を前提に型を崩さない。
- 関数コンポーネント + hooks のみを使う。
- グローバル状態は zustand、サーバー状態は @tanstack/react-query を使う。
- `invoke()` は必ず `src/hooks/` にラップし、コンポーネントから直接呼ばない。
- Rust 側の返却型と `src/types/ddr.ts` の同期を保つ。

## Tauri IPC 追加フロー

新しい IPC コマンドを追加するときは次をすべて完了します。

1. `src-tauri/src/commands/<domain>.rs` に `#[tauri::command]` 関数を定義する。
2. `src-tauri/src/commands/mod.rs` に必要な module export を追加する。
3. `src-tauri/src/lib.rs` の `invoke_handler` に登録する。
4. `src/hooks/<domain>.ts` に `invoke()` ラッパーを追加する。
5. `src/types/ddr.ts` に TypeScript 型を追加する。
6. Rust のユニットテストを追加する。
7. `ARCHITECTURE.md` の IPC コマンド一覧を更新する。

## 専門チェック

Claude Code の専門 agent で担っていた観点は、Codex でも以下を明示的に確認します。

| 領域 | 確認内容 |
|---|---|
| ADR | 非自明な設計判断では `docs/decisions/NNNN-slug.md` と `docs/decisions/README.md` を更新する |
| Architecture Sync | 新規ファイル・モジュール・IPC・構成変更後に `ARCHITECTURE.md` と実コードの差分を確認する |
| Schema Guard | `src-tauri/src/db/schema.rs` 変更時はカラム削除、型変更、NOT NULL 追加、テーブル削除/リネーム、FTS5 構造変更を危険操作として確認する |
| Version Adapter | `src-tauri/src/parser/` のバージョン差異は `VersionAdapter` に集約し、パーサー本体に直接バージョン分岐を書かない |
| Tauri IPC | IPC 追加時は Rust command、`commands/mod.rs`、`lib.rs`、frontend hook、型定義、テスト、`ARCHITECTURE.md` をセットで確認する |

## テスト方針

新規コードにはテストを追加します。ドキュメントのみの変更では実装テストは不要ですが、PR 前の既存テスト確認は行います。

| 種別 | コマンド |
|---|---|
| フロントエンド | `npm run test` |
| Rust | `cargo test` in `src-tauri/` |
| Rust fmt | `cargo fmt --check` in `src-tauri/` |
| Rust clippy | `cargo clippy -- -D warnings` in `src-tauri/` |
| TypeScript build | `npm run build` または `tsc --noEmit` 相当 |
| E2E | `npm run test:e2e` |
| 状態確認 | `npm run agent:status` |
| セッション開始チェック | `npm run agent:session-start` |
| TDD 順序チェック | `npm run agent:tdd-guard` |
| agent docs 参照チェック | `npm run agent:docs-check` |
| PR 前一括チェック | `npm run agent:pre-pr` |

大量データを扱うフロントエンドテストでは、DOM に大量レンダーせず hook の呼び出し引数を検証します。

## ADR 方針

次のいずれかに当てはまる場合は `docs/decisions/NNNN-slug.md` を追加します。

- 複数の実装案を比較した。
- 将来「なぜこうしたのか」が疑問になりそうな非自明な決定。
- DB スキーマ、IPC 設計、パーサー構造など後から覆しにくい決定。

単純なバグ修正、ライブラリ更新、コードを読めば自明な実装方針では ADR は不要です。

## ARCHITECTURE.md 更新

実装で以下が変わった場合は `ARCHITECTURE.md` を更新します。

- 機能の実装状況
- IPC コマンド
- DB スキーマ
- パーサー仕様
- 既知の制約
- E2E や開発運用の重要な手順

## PR 前チェック

PR 前に少なくとも次を確認します。

- `npm run agent:pre-pr`
- 必要に応じて `npm run agent:tdd-guard`
- agent 運用ドキュメントを変更した場合は `npm run agent:docs-check`
- ADR を追加した場合は `docs/decisions/README.md` を更新
- `ARCHITECTURE.md` の更新要否を確認

## PR 作成

- PR は作業ブランチから作成する。
- CI が通ることを確認する。
- `main` へのマージはユーザーが CI 確認後に行う。

## リリース運用

リリース作業は通常の開発 PR とは分けて扱います。

- バージョン更新は `package.json`、`src-tauri/Cargo.toml` の `[package] version`、`src-tauri/tauri.conf.json` の 3 ファイルを同期する。
- `CHANGELOG.md` は git tag と Conventional Commits を元に更新する。
- リリースコミットは `chore: release vX.Y.Z` とする。
- リリースタグは `vX.Y.Z` の annotated tag とする。
- `git push` はユーザー確認後に手動で行う。
- `git push --force` は実行しない。
