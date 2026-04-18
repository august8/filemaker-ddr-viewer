# 0003: フィールド検索結果に親テーブル情報を LEFT JOIN で付与する

- **ステータス**: Accepted
- **日付**: 2026-02

## コンテキスト

検索結果でフィールドをクリックした際、親テーブルの詳細画面へ遷移する必要がある。`search_index` にはテーブル情報が含まれない。

## 決定

`search()` の SQL に `LEFT JOIN fields` と `LEFT JOIN base_tables` を追加し、`parent_id`（テーブルの DB ID）と `parent_name`（テーブル名）を検索結果に付与する。

## 理由

フィールド以外の要素では JOIN 結果が NULL になるだけで副作用がない。別途 IPC 呼び出しを追加せずに済む。

## 関連ファイル

- `src-tauri/src/db/repository/search.rs`
