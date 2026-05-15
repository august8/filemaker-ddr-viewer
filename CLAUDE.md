# Claude Code Entry

このファイルは Claude Code 用の薄い入口です。
詳細な共通ルールは `docs/agent/` 配下を正本とします。

## 最初に読む文書

1. `docs/agent/README.md`
2. `docs/agent/CLAUDE.md`
3. `ARCHITECTURE.md`

## 基本ルール

- ユーザーとのやり取りは日本語で行う。
- コード、コマンド、ログ、エラーメッセージ、ファイル名、API 名は原文を維持する。
- 実装前に `ARCHITECTURE.md` を読み、実コードと照合する。
- ファイル編集前に作業ブランチを作成する。
- ユーザーや Codex 由来の未コミット変更を勝手に戻さない。
- 実装前に `EnterPlanMode` で作業範囲を確認し、ユーザー承認を得る。
- タスクリストの各ステップ完了時に `✅ ステップ名 完了` と報告する。

## Claude Code 専用コマンド

- `EnterPlanMode`: 作業範囲と確認箇所を列挙する。
- `/commit-ja`: 日本語 Conventional Commits メッセージを生成する。
- `/test-spec`: 必要に応じてテスト仕様を生成する。
- `/tdd-guard`: 実装前にテストが先に書かれているか確認する。
- `/pre-pr`: PR 前チェックリストを通す。

## セッション開始報告

```text
### セッション開始チェック完了
- ARCHITECTURE.md: 確認済み（最終更新: YYYY-MM-DD）
- テスト: X passed / X failed
- 気になった点: なし
```

## ローカル設定

個人設定は `CLAUDE.local.md` または `.claude/settings.local.json` に置きます。
共有すべき Claude Code 運用ルールは `docs/agent/CLAUDE.md` に記載します。
