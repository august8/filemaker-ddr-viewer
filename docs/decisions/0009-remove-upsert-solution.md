# 0009: upsert_solution を削除しインポートは常に新規追加とする

- **ステータス**: Accepted
- **日付**: 2026-04-17

## コンテキスト

`upsert_solution` は「同一 `summary_path` の既存ソリューションを削除してから INSERT する（上書きインポート）」関数として実装されていたが、どこからも呼ばれていない Dead Code だった。実際の `import_solution`・`import_ddr` はいずれも `insert_solution`（毎回新規追加）を呼んでおり、ドキュメントとの齟齬があった。

## 決定

1. `upsert_solution` 関数と付属テストを削除する
2. インポートは常に `insert_solution` で新規追加する（重複チェックなし）

## 理由

上書きインポート機能を将来追加する予定がない。Dead Code を残すとドキュメント誤記と合わさって「実装済み機能」と誤解させるリスクがある。

## 却下した案

コメント付きで残す → 予定のない機能のために Dead Code を保持するメリットがない。

## 関連ファイル

- `src-tauri/src/db/repository/solution.rs`
- `src-tauri/src/db/repository/mod.rs`
