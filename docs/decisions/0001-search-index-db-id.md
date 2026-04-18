# 0001: search_index.element_id に DB auto-increment ID を使う

- **ステータス**: Accepted
- **日付**: 2026-01

## コンテキスト

検索インデックスに要素の ID を持たせる必要があった。FileMaker が XML に付与する内部 ID（`fm_id`）と SQLite の auto-increment ID のどちらを使うかを決める必要があった。

## 決定

`search_index.element_id` には SQLite の auto-increment ID（DB ID）を使う。FM 内部 ID は使わない。

## 理由

FM 内部 ID と DB ID は一致しない。FM ID でインデックスを作ると、フロントエンドの `scripts.find(s => s.id === element_id)` が失敗する。`list_scripts` 等が返す各 Row の `.id` は DB ID であるため、検索インデックスも DB ID に揃える必要がある。

## 関連ファイル

- `src-tauri/src/db/repository/import.rs`
- `src-tauri/src/db/repository/search.rs`
