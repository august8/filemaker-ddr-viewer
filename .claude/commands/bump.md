---
description: バージョンを patch/minor/major または指定バージョンに一括更新する
---

# /bump — バージョン一括更新

**引数:** `patch` | `minor` | `major` | `X.Y.Z`（例: `0.2.0`）

引数が指定されていない場合は、以下の使い方を表示して終了する。

```
使い方: /bump <patch|minor|major|X.Y.Z>
例:
  /bump patch   → 0.1.1 → 0.1.2
  /bump minor   → 0.1.1 → 0.2.0
  /bump major   → 0.1.1 → 1.0.0
  /bump 0.2.0   → 0.1.1 → 0.2.0
```

## 手順

1. `package.json` を読み取り、現在のバージョンを取得する
2. 引数に応じて新バージョンを計算する（Semantic Versioning 準拠）
   - `patch`: Z を +1（X.Y.Z → X.Y.Z+1）
   - `minor`: Y を +1、Z を 0 にリセット（X.Y.Z → X.Y+1.0）
   - `major`: X を +1、Y・Z を 0 にリセット（X.Y.Z → X+1.0.0）
   - 明示バージョン（例: `0.2.0`）:
     - `X.Y.Z` 形式（X・Y・Z が非負整数）であることを確認する。形式が不正な場合はエラーを出して終了する。
     - セマンティックバージョン比較で新バージョン > 現在バージョンであることを確認する。ダウングレードになる場合は「バージョンのダウングレードはできません（現在: X.Y.Z → 指定: A.B.C）」と表示して終了する。
3. 以下の **3 ファイル**を更新する（それ以外は変更しない）:
   - `package.json` の `"version"` フィールド
   - `src-tauri/Cargo.toml` の `[package]` セクションの `version` フィールド（**`[dependencies]` 内の version は変更しない**）
   - `src-tauri/tauri.conf.json` の `"version"` フィールド
4. 変更内容を表示して完了を報告する:
   ```
   ✅ バージョンを X.Y.Z → A.B.C に更新しました
   更新ファイル:
     - package.json
     - src-tauri/Cargo.toml
     - src-tauri/tauri.conf.json
   次のステップ: /changelog で CHANGELOG.md を更新してください
   ```

$ARGUMENTS
