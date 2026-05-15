---
description: 実装ファイルを編集する前に対応するテスト変更の有無と履歴上の順序を確認する
---

# TDD Guard

React コンポーネントや Rust モジュールの実装ファイルを編集しようとする前に、
対応するテストが先に存在・更新されているかを確認する。

Claude Code と Codex の確認内容を揃えるため、共通の npm script を使う。
未コミット差分内の実作業順序は Git から復元できないため、Red 確認と作業順序は目視でも確認する。

## 実行手順

```bash
npm run agent:tdd-guard
```

## 対象

- `src/components/**/*.tsx`
- `src-tauri/src/commands/**/*.rs`
- `src-tauri/src/parser/**/*.rs`
- `src-tauri/src/analyzer/**/*.rs`

## 注意事項

- `src/components/RightPanel.tsx` と `src/components/detail/RelationshipGraphPanel.tsx` は coverage 除外対象のため、テストなしでも警告しない。
- バグ修正で既存テストが失敗している場合は、テスト追加より先に既存テストの修正を優先する。
- リファクタリング（動作変更なし）の場合は、既存テストのパスを確認してから進める。
