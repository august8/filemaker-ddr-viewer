# Codex Guide

この文書は Codex 固有の運用ルールです。
共通ルールは `docs/agent/README.md` を正本とします。

## 読み込み順

1. `AGENTS.md`
2. `docs/agent/README.md`
3. `docs/agent/CODEX.md`
4. `ARCHITECTURE.md`

## Claude Code コマンドの代替

Claude Code 専用の slash command は Codex では直接実行できないため、同等の確認を通常コマンドと目視で行います。

| Claude Code | Codex での代替 |
|---|---|
| `EnterPlanMode` | ユーザーに作業範囲とタスクリストを提示し、承認後に編集する |
| `/commit-ja` | `git diff --cached` を確認し、Conventional Commits 形式の日本語コミットメッセージ本文だけを提案する |
| `/test-spec` | 追加するテスト観点を会話内または対象テストファイルに明示する |
| `/tdd-guard` | `npm run agent:tdd-guard` を実行し、必要に応じて `git diff` も確認する |
| `/pre-pr` | `npm run agent:pre-pr` を実行する |
| `/bump` | バージョン更新対象 3 ファイルだけを変更する。必要なら別途スクリプト化する |
| `/changelog` | git tag と commit subject から `CHANGELOG.md` を更新する。コミット文面は翻訳しない |
| `/release` | `main` 上で changelog と version を確認し、release commit と annotated tag を作る。push は自動実行しない |

## ブランチ作成

Codex では作業開始前に `git switch -c <branch>` を実行します。
Windows の権限や Git refs の制約でスラッシュ付きブランチ名が作れない場合は、理由を報告してスラッシュなしの名前を使います。

## 進め方

- 実装前に何を変更するかを短く説明し、ユーザー承認を得る。
- ファイル編集前に編集対象を明示する。
- 進捗は `✅ ステップ名 完了` で報告する。
- 既存差分はユーザーまたは他ツール由来として扱い、勝手に戻さない。
- PR 前には `/pre-pr` の代替として `npm run agent:pre-pr` を実行する。
- セッション開始時の自動確認が必要な場合は `npm run agent:session-start` を使う。
- 状態確認には `npm run agent:status` を使う。
- コミットメッセージを提案・作成するときは `/commit-ja` 相当として日本語 description を使う。

## コミット

Codex では Claude Code の `/commit-ja` を直接実行できません。
代わりに次の手順を必ず守ります。

1. `git diff --cached` でステージ済み変更を確認する。
2. ステージ済み変更がない場合は、コミットメッセージを作らずユーザーにステージを促す。
3. 変更内容に合う Conventional Commits の `type` を選ぶ。
4. description は自然な日本語で 50 文字以内を目安にする。
5. 複数 type にまたがる場合は、コミット分割を提案する。
6. ユーザーが明示しない限り、英語コミットメッセージは使わない。

## `.codex/` の扱い

`.codex/` は Codex のローカル設定・hook を置く場所として扱います。
個人環境差が出やすいため、原則としてリポジトリにはコミットしません。

共有したい Codex 運用ルールは `.codex/` ではなく、この `docs/agent/CODEX.md` に記載します。
チームで共有する Codex hook が必要になった場合は、別途 ADR を作成してから追跡対象にします。
