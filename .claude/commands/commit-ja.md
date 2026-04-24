---
description: ステージ済みの変更から Conventional Commits 形式の日本語コミットメッセージを生成する
---

# /commit-ja — 日本語コミットメッセージ生成

## 手順

1. `git diff --cached` を実行してステージ済みの変更を確認する
   - ステージ済みの変更がない場合: 「ステージされた変更がありません。`git add` でファイルをステージしてください。」と返して終了する
2. 変更内容を分析して最も適切な type を選択する:
   `feat` / `fix` / `refactor` / `perf` / `docs` / `style` / `test` / `chore` / `build` / `ci` / `revert`
3. 複数の type にまたがる変更の場合: コミット分割を提案してからメッセージを生成する
4. Conventional Commits 形式の日本語コミットメッセージを生成する:
   - **type**: 英語のまま
   - **scope**（任意）: 変更対象モジュール（例: `parser`, `search`, `db`, `ui`）
   - **description**: 自然な日本語、50文字以内
   - **body**（任意）: 空行の後、変更の意図・影響を簡潔に記述

**出力ルール:**
- コミットメッセージ本文のみを直接出力する（コピー用）
- 前置き・解説・締めは一切書かない

**出力例:**
```
feat(parser): FM22 の ScriptTrigger 新属性に対応

withFewerFolders 属性を VersionAdapter 経由で正規化するよう修正。
FM17〜21 との後方互換性を維持。
```

$ARGUMENTS
