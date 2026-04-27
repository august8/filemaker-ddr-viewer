---
description: 実装ファイルを編集する前に対応するテストが先に書かれているかを確認し、TDD順序を守らせる
---

# TDD Guard

React コンポーネントや Rust モジュールの実装ファイルを編集しようとする前に、
対応するテストが先に存在・更新されているかを確認する。

## トリガー条件

以下のファイルを新規作成または編集しようとするとき:
- `src/components/**/*.tsx`
- `src-tauri/src/commands/**/*.rs`
- `src-tauri/src/parser/**/*.rs`
- `src-tauri/src/analyzer/**/*.rs`

## 動作手順

1. **対応テストファイルの特定**

   | 実装ファイル | 対応テストファイル |
   |---|---|
   | `src/components/detail/Foo.tsx` | `src/__tests__/detail/Foo.test.tsx` |
   | `src/components/Foo.tsx` | `src/__tests__/Foo.test.tsx` |
   | `src-tauri/src/commands/foo.rs` | 同ファイル末尾の `#[cfg(test)] mod tests { ... }` |
   | `src-tauri/src/parser/foo_parser.rs` | 同ファイル末尾の `#[cfg(test)] mod tests { ... }` |

2. **テストの先行確認**

   - テストファイル（または `#[cfg(test)]` ブロック）が存在しない場合:
     → 「テストファイルを先に作成してください」と報告し、テストファイルのスケルトンを提示する
   
   - テストファイルは存在するが、変更する機能に対応する `it()` / `#[test]` がない場合:
     → 「対応するテストケースを先に追加してください」と報告する
   
   - テストが先に書かれている場合:
     → そのまま実装を進める

3. **順序が逆転していた場合の対応**

   実装ファイルを先に変更してしまった後でテストを書いている場合（TDD違反を検出）:
   - 現状を報告する（「TDD の順序が逆転しています」）
   - 実装を一旦 `git stash` してテストを先に書き直すことを提案する
   - ユーザーが続行を選択した場合はそのまま進める（強制はしない）

## 注意事項

- `src/components/RightPanel.tsx`（純ルーティング）と `src/components/detail/RelationshipGraphPanel.tsx`（SVG描画）は coverage 除外対象のため、テストなしでも警告しない
- バグ修正で既存テストが失敗している場合は、テスト追加より先に既存テストの修正を優先する
- リファクタリング（動作変更なし）の場合は、既存テストのパスを確認してから進める
