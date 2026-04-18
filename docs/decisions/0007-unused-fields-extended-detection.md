# 0007: 未参照フィールド検出をスクリプト・計算式・バリューリストまで拡張する

- **ステータス**: Accepted
- **日付**: 2026-04-17

## コンテキスト

`list_unused_fields()` はレイアウト配置とリレーション結合キーしか確認しないため、スクリプトや計算式・バリューリストで使われているフィールドが誤って「未参照」と表示されていた。

## 決定

- **スクリプト・計算式**: `OccName::FieldName` パターンのみを Rust ポスト処理で検出する。`script_steps.calculation / step_text` と `fields.calculation / auto_enter_calc / val_calc` を対象にする。ベア参照は誤検出リスクがあるため除外する。
- **バリューリスト**: DDR XML の `<PrimaryField>/<SecondaryField>` を新規パースして `value_list_field_refs` テーブルに保存し、SQL UNION で `used_fields` CTE に追加する。

## 理由

- スクリプト・計算式: データは既に DB に存在するため、SQL LIKE クロスジョインを避けて Rust ポスト処理が最もシンプルかつ高速。
- バリューリスト: テーブルオカレンス名とフィールド名が XML に明示されるため、構造的参照として SQL で処理できる。

## 却下した案

- ベア参照の検出: 同名フィールドが複数テーブルに存在する場合に誤検出が発生するため対象外。
- SQL LIKE クロスジョイン: `script_steps × table_occurrences × fields` の組み合わせになり著しく遅いため不採用。

## 関連ファイル

- `src-tauri/src/parser/models.rs`（`ValueListFieldRef` 構造体）
- `src-tauri/src/parser/catalog_parser.rs`（PrimaryField/SecondaryField パース）
- `src-tauri/src/db/schema.rs`（`value_list_field_refs` テーブル）
- `src-tauri/src/db/repository/import.rs`（INSERT）
- `src-tauri/src/commands/field_refs.rs`（`list_unused_fields` 拡張）
