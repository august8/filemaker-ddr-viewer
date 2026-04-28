# 設計判断記録（ADR）

このディレクトリには Architecture Decision Records（ADR）を格納しています。

| ADR | タイトル | ステータス |
|-----|---------|-----------|
| [0001](0001-search-index-db-id.md) | search_index.element_id に DB auto-increment ID を使う | Accepted |
| [0002](0002-fts5-full-column-search.md) | FTS5 検索を name + content 全カラム検索にする | Accepted |
| [0003](0003-field-search-parent-table-join.md) | フィールド検索結果に親テーブル情報を LEFT JOIN で付与する | Accepted |
| [0004](0004-selected-element-display-priority.md) | selectedElement を searchQuery より表示優先にする | Accepted |
| [0005](0005-always-fetch-list-data.md) | リストデータをプロジェクト選択時点で常時フェッチする | Accepted |
| [0006](0006-field-calc-refs.md) | フィールド詳細に計算式参照箇所セクションを追加する | Accepted |
| [0007](0007-unused-fields-extended-detection.md) | 未参照フィールド検出をスクリプト・計算式・バリューリストまで拡張する | Accepted |
| [0008](0008-unused-fields-bare-ref.md) | 未参照フィールド検出に同テーブル内ベア参照チェックを追加する | Accepted |
| [0009](0009-remove-upsert-solution.md) | upsert_solution を削除しインポートは常に新規追加とする | Accepted |
| [0010](0010-utf16-lossy-fallback.md) | UTF-16 デコード失敗時に lossy フォールバックを追加する | Accepted |
| [0011](0011-e2e-testing.md) | E2E テスト基盤の技術選定（WebdriverIO + tauri-driver） | Accepted |
