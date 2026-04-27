---
description: PR作成前のスコープ完全性・テスト通過を確認するチェックリストを実行する
---

# /pre-pr — PR作成前チェックリスト

`gh pr create` を実行する前に必ずこのコマンドを実行する。
全チェックを通過してから PR を作成すること。

## 実行手順

### 1. 変更ファイルの列挙

```bash
git diff origin/main...HEAD --name-only
```

変更ファイルを列挙し、以下のチェックに使用する。

### 2. スコープ完全性チェック

変更ファイルを以下の観点で確認する:

**バックエンド → フロントエンドの対応:**
- `src-tauri/src/commands/` が変更されている場合、対応する `src/hooks/` の変更が存在するか
- バックエンドに新しい IPC コマンドが追加された場合、フロントエンドの hook と UI が実装されているか

**実装 → テストの対応:**
- `src/components/**/*.tsx` が変更されている場合、`src/__tests__/**/*.test.tsx` も変更されているか
- `src-tauri/src/**/*.rs` が変更されている場合、`#[cfg(test)]` または `src-tauri/tests/` にテストが追加されているか

**TDD 順序の確認:**
- `git log --oneline origin/main...HEAD` でコミット履歴を確認し、テストコミットが実装コミットより後になっていないか

### 3. 未完了マーカーの確認

```bash
git diff origin/main...HEAD
```

以下のマーカーが含まれていないか確認する:
- `TODO` / `FIXME` / `HACK`
- Rust: `todo!()` / `unimplemented!()`
- コメントアウトされた未実装箇所

### 4. テスト実行

```bash
npm run test
```

Rust ファイルが変更されている場合は追加で:
```bash
cargo test
```

全テストがグリーンであることを確認する。

### 5. 結果レポート

全チェックが通過した場合:
```
✅ PR作成可能です
```

問題がある場合:
```
❌ 以下を修正してからPRを作成してください:
- [具体的な問題点を列挙]
```

$ARGUMENTS
