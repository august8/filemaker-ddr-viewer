# 0006: フィールド詳細に計算式参照箇所セクションを追加する

- **ステータス**: Accepted
- **日付**: 2026-04-13

## コンテキスト

フィールド詳細にはスクリプト・レイアウト・リレーションキーの使用箇所が表示されていたが、他のフィールドの計算式内で参照されているケースが表示されていなかった。

## 決定

`get_field_calc_refs` コマンドを追加し、`fields.calculation` に `テーブル名::フィールド名` のパターンで LIKE 検索する。結果を `FieldDetail.tsx` の新セクションに表示し、クリックで親テーブル詳細へ遷移する。

## 理由

スクリプト参照と同様のパターンで実装できる。`fields.calculation` カラムは既に DB に存在する。

## 却下した案

FTS5 で検索する案 → `fields` は `search_index` の `content` に入っているが、フィールド単位での絞り込みが複雑になるため直接 SQL の方がシンプル。

## 関連ファイル

- `src-tauri/src/commands/field_refs.rs`
- `src/components/detail/FieldDetail.tsx`
- `src/hooks/fieldRefs.ts`
