# ADR 0012: Claude Code と Codex の入口ドキュメントを分離する

- **Status**: Accepted
- **Date**: 2026-05-14

## Context

このプロジェクトは当初 Claude Code を前提に `CLAUDE.md` へ詳細な作業ルールを集約していた。
Codex へ開発環境を移行したことで `AGENTS.md` も必要になり、同じ内容を 2 つの入口ファイルに複製する状態になった。

複製された入口ファイルには次の問題がある。

- 共通ルールを更新するときに `AGENTS.md` と `CLAUDE.md` の同期漏れが起きる。
- Claude Code 専用の `EnterPlanMode`、`/commit-ja`、`/test-spec`、`/tdd-guard`、`/pre-pr`、release 系コマンドが Codex では直接実行できない。
- Codex 固有の `.codex/` 設定や hook をどこまで追跡するかが曖昧になる。
- 新しいエージェントを追加すると、巨大な入口ファイルの複製がさらに増える。

## Decision

ツール非依存のルールを `docs/agent/` に集約し、各エージェントの入口ファイルは薄く保つ。

- `AGENTS.md`: Codex 用の入口。
- `CLAUDE.md`: Claude Code 用の入口。
- `docs/agent/README.md`: プロジェクト概要、技術スタック、共通開発規約、作業フロー。
- `docs/agent/CODEX.md`: Codex 固有の代替手順と `.codex/` の扱い。
- `docs/agent/CLAUDE.md`: Claude Code 固有の slash command と Plan mode 手順。
- `scripts/agent/*.mjs`: Claude Code の主要 workflow を Codex でも実行できる cross-platform チェック。

`.codex/` は個人環境差が出やすいローカル設定として扱い、原則コミットしない。
共有すべき Codex 運用ルールは `.codex/` ではなく `docs/agent/CODEX.md` に記載する。

## Consequences

共通ルールの更新先が明確になり、Claude Code と Codex の入口ファイルの重複を減らせる。
一方で、各エージェントは入口ファイルから `docs/agent/` の追加文書を読む必要がある。

ツール固有のワークフロー差分は残るが、差分は `docs/agent/CODEX.md` と `docs/agent/CLAUDE.md` に閉じ込める。
実行漏れを防ぎたい PR 前チェックは、ドキュメント上の代替手順だけでなく `npm run agent:pre-pr` として実行可能にする。

release 系コマンドの npm script 化など、頻度の低い自動化は必要になった時点で追加する。

## Related Files

- `AGENTS.md`
- `CLAUDE.md`
- `docs/agent/README.md`
- `docs/agent/CODEX.md`
- `docs/agent/CLAUDE.md`
- `scripts/agent/*.mjs`
- `package.json`
- `.gitignore`
