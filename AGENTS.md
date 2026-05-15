# Codex Entry

このファイルは Codex 用の薄い入口です。
詳細な共通ルールは `docs/agent/` 配下を正本とします。

## 最初に読む文書

1. `docs/agent/README.md`
2. `docs/agent/CODEX.md`
3. `ARCHITECTURE.md`

## 基本ルール

- ユーザーとのやり取りは日本語で行う。
- コード、コマンド、ログ、エラーメッセージ、ファイル名、API 名は原文を維持する。
- 実装前に `ARCHITECTURE.md` を読み、実コードと照合する。
- ファイル編集前に作業ブランチを作成する。
- ユーザーや Claude Code 由来の未コミット変更を勝手に戻さない。
- 実装前に「何をどう変えるか」を短く伝えて確認を取る。
- タスクリストの各ステップ完了時に `✅ ステップ名 完了` と報告する。

## Codex での代替手順

Claude Code 専用コマンドは Codex では直接実行できません。
次の対応を行います。

| Claude Code | Codex |
|---|---|
| `EnterPlanMode` | 作業範囲とタスクリストを提示し、ユーザー承認後に編集する |
| `/commit-ja` | `git diff --cached` を確認し、日本語 Conventional Commits メッセージを提案する |
| `/test-spec` | テスト観点を会話内または対象テストファイルに明示する |
| `/tdd-guard` | `git diff` と作業順序を確認する |
| `/pre-pr` | `npm run agent:pre-pr` を実行する |

## セッション開始報告

```text
### セッション開始チェック完了
- ARCHITECTURE.md: 確認済み（最終更新: YYYY-MM-DD）
- テスト: X passed / X failed
- 気になった点: なし
```

## `.codex/`

`.codex/` はローカル設定・hook として扱い、原則コミットしません。
共有すべき Codex 運用ルールは `docs/agent/CODEX.md` に記載します。
