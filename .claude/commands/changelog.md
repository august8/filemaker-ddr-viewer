---
description: git log から CHANGELOG.md のエントリを自動生成する
---

# /changelog — CHANGELOG.md 自動生成

`/bump` でバージョンを更新した後に実行する。

## 手順

1. `package.json` から現在バージョン（`CURRENT`）を読み取る
2. `git tag --sort=-version:refname` で直近タグ（`PREV_TAG`）を特定する
   - タグが存在しない場合は初回リリースとして全コミットを対象にする
3. `git log <PREV_TAG>..HEAD --pretty=format:"%s"` でコミット一覧を取得する
4. コミットメッセージを以下のカテゴリに分類する（Conventional Commits 準拠）:
   - `feat:` / `feat(*):` → `### 機能`
   - `fix:` / `fix(*):` → `### 修正`
   - `refactor:` / `perf:` → `### 改善`
   - `docs:` → `### ドキュメント`
   - `chore:` / `test:` / `ci:` / `build:` → 省略（内容が外部向けに意味のある場合のみ `### その他` に含める）
5. コミット本文の末尾にある PR 番号（`(#NN)` 形式）は除去する
6. 同一機能への複数コミット（例: 同じリファクタリングの分割コミット）は 1 エントリにまとめる
7. 以下の形式で CHANGELOG.md に挿入する:
   - 挿入位置: 最初の `## [` の直前（`## [Unreleased]` がある場合はその下）
   - 空のカテゴリ（コミットが 0 件）は省略する

```markdown
## [CURRENT] - YYYY-MM-DD

### 機能
- ...

### 修正
- ...

### 改善
- ...
```

8. ファイル末尾の比較リンクセクションに新エントリを追加する:
   ```
   [CURRENT]: https://github.com/august8/filemaker-ddr-viewer/compare/vPREV...vCURRENT
   ```
   既存の最上位リンクの直前に挿入する。

9. 完了を報告する:
   ```
   ✅ CHANGELOG.md を更新しました（vCURRENT）
   次のステップ: /release でコミットとタグを作成してください
   ```

**注意:** コミットメッセージの日本語訳はしない。英語コミットはそのまま記載する。

$ARGUMENTS
