# Claude Code Guide

この文書は Claude Code 固有の運用ルールです。
共通ルールは `docs/agent/README.md` を正本とします。

## 読み込み順

1. `CLAUDE.md`
2. `docs/agent/README.md`
3. `docs/agent/CLAUDE.md`
4. `ARCHITECTURE.md`

## Claude Code 専用フロー

- 実装前に `EnterPlanMode` で作業範囲を列挙し、ユーザー承認を得る。
- プランの末尾は `## タスクリスト` セクションにする。
- タスクリストの先頭に作業ブランチ名を具体的に明記する。
- コミットメッセージ作成時は `/commit-ja` を使う。
- 必要に応じて `/test-spec` でテスト仕様を生成する。
- 実装前に `/tdd-guard` または `npm run agent:tdd-guard` でテスト変更の有無と履歴上の順序を確認し、未コミット差分の順序は目視でも確認する。
- PR 前に `/pre-pr` を実行する。必要に応じて同じ確認内容を `npm run agent:pre-pr` でも実行できる。

## タスクリストテンプレート

```markdown
## タスクリスト

- [ ] ブランチ作成: `main` → `docs/example-task`
- [ ] `/test-spec` 実行（必要な場合）
- [ ] テスト追加（Red確認）
- [ ] `/tdd-guard` 実行
- [ ] 実装（Green確認）
- [ ] PR 前チェック: `/pre-pr` または `npm run agent:pre-pr` 実行・全チェック通過
- [ ] コミット作成: `/commit-ja` で日本語 Conventional Commits メッセージ生成
- [ ] PR 作成
```

## Claude Code コマンド

| コマンド | 用途 |
|---|---|
| `/commit-ja` | ステージ済み変更から日本語 Conventional Commits メッセージを生成する |
| `/test-spec` | 実装前にテスト仕様を生成する |
| `/tdd-guard` | テスト変更の有無と履歴上の順序を確認する。共通代替は `npm run agent:tdd-guard` |
| `/pre-pr` | PR 前のスコープ完全性・テスト通過を確認する。共通代替は `npm run agent:pre-pr` |
| `/bump` | version を関連 3 ファイルで同期更新する |
| `/changelog` | git log から `CHANGELOG.md` を更新する |
| `/release` | release commit と tag を作成する。push は手動 |

## ローカル設定

- 個人設定は `CLAUDE.local.md` または `.claude/settings.local.json` に置く。
- これらは `.gitignore` 済みで、リポジトリにはコミットしない。
- 共有すべき Claude Code 運用ルールはこの文書に記載する。
