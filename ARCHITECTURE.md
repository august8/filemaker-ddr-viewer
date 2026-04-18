# アーキテクチャ

## 技術スタック

| レイヤー | 技術 |
|---------|------|
| デスクトップ | Tauri 2.x（OS ネイティブ WebView） |
| バックエンド | Rust（全ビジネスロジック） |
| フロントエンド | React 19 + TypeScript + Vite + TailwindCSS |
| XML 解析 | quick-xml + serde |
| 全文検索 | FTS5（SQLite 組み込み） |
| データ保存 | rusqlite (SQLite, bundled) |
| グラフ解析 | petgraph |
| フロントエンド可視化 | D3.js + dagre-d3 |
| 状態管理 | zustand |
| サーバー状態 | @tanstack/react-query（Tauri IPC 経由） |

---

## プロジェクト構成

```
filemaker-ddr-viewer/
├── src/                            # React フロントエンド
│   ├── App.tsx
│   ├── components/
│   │   ├── navigation/CategoryTree.tsx
│   │   └── detail/                 # 各詳細パネル
│   ├── hooks/useTauriCommand.ts    # 全 invoke() はここに集約
│   ├── stores/appStore.ts          # グローバル状態（zustand）
│   └── types/ddr.ts                # 型定義
├── src-tauri/src/
│   ├── commands/                   # Tauri IPC コマンド
│   ├── parser/                     # DDR XML パーサー
│   ├── db/                         # SQLite データ層
│   ├── analyzer/                   # 解析エンジン
│   └── search/                     # tantivy 全文検索（未使用、FTS5 で代替）
├── tests/fixtures/                 # テスト用 DDR XML サンプル（FM17〜22）
└── docs/decisions/                 # 設計判断記録（ADR）
```

---

## データフロー

```
概要.xml（DDR）
  → parser::parse_ddr()              quick-xml ストリーミングパース
  → DdrFile（インメモリ構造体）
  → db::repository::insert_ddr_file() SQLite 永続化 + FTS5 インデックス構築
  → Tauri IPC（invoke）
  → React コンポーネント
```

---

## バックエンドモジュール

| モジュール | 役割 |
|-----------|------|
| `parser/` | DDR XML をストリーミングパースして `DdrFile` 構造体に変換。FM17〜最新に対応 |
| `db/schema.rs` | SQLite スキーマ定義・マイグレーション |
| `db/repository/` | CRUD・FTS5 検索・solution/project 管理 |
| `commands/import.rs` | `概要.xml` 起点の一括インポート・単体 DDR ファイルのインポート |
| `commands/search.rs` | FTS5 全文検索（name + content 全カラム） |
| `commands/analysis.rs` | ProjectSummary・BrokenRefs・ReportCard |
| `commands/catalog.rs` | エンティティ一覧取得（`list_*` コマンド群） |
| `commands/field_refs.rs` | フィールド参照解析 |
| `commands/callchain.rs` | コールチェーン・Callers・孤立スクリプト検出 |
| `commands/diff.rs` | 2 プロジェクト間の差分比較 |
| `analyzer/broken_refs.rs` | PerformScript / ScriptTrigger の壊れた参照検出 |
| `analyzer/orphans.rs` | 未使用スクリプト検出 |
| `analyzer/call_chain.rs` | DFS + 循環参照検出 |
| `analyzer/report_card.rs` | 健全性レポート（Info / Warning / Error） |
| `analyzer/diff_engine.rs` | 名前ベース差分比較（テーブルはフィールド単位で詳細比較） |

---

## フロントエンドモジュール

| コンポーネント / ファイル | 役割 |
|--------------------------|------|
| `App.tsx` | 3 ペインレイアウト・リサイズ・ナビゲーション |
| `MainContent.tsx` | メインエリアのルーティング（`selectedElement.kind` で切り替え） |
| `navigation/CategoryTree.tsx` | サイドバーツリー |
| `SearchBar.tsx` / `SearchResults.tsx` | 全文検索・カテゴリフィルター・クリック遷移 |
| `SolutionList.tsx` | ソリューション一覧・削除 |
| `ImportButton.tsx` | `概要.xml` インポート |
| `ProjectSummaryCard.tsx` | 要素数サマリー |
| `ReportCard.tsx` | 健全性レポート |
| `BrokenRefsList.tsx` | 壊れた参照一覧 |
| `OrphanScriptsList.tsx` | 孤立スクリプト一覧 |
| `detail/TableDetail.tsx` | テーブル詳細・フィールド一覧 |
| `detail/FieldDetail.tsx` | フィールド詳細・Where Used |
| `detail/ScriptDetail.tsx` | スクリプト詳細・ステップ一覧・diff表示 |
| `detail/LayoutDetail.tsx` | レイアウト詳細・トリガー・オブジェクト |
| `detail/LayoutObjectDetail.tsx` | レイアウトオブジェクト詳細 |
| `detail/All*Panel.tsx` | 各エンティティの横断一覧（AllTablesPanel / AllFieldsPanel / AllScriptsPanel / AllLayoutsPanel / AllTableOccurrencesPanel / AllRelationshipsPanel / AllValueListsPanel / AllCustomFunctionsPanel） |
| `detail/RelationshipGraphPanel.tsx` | リレーショングラフ（dagre + SVG、pan/zoom 対応） |
| `detail/CallChainTree.tsx` | コールチェーンツリー |
| `detail/WhereUsed.tsx` | 参照元一覧 |
| `detail/SecurityPanel.tsx` | アカウント・権限セット |
| `detail/UpgradeCheckPanel.tsx` / `UpgradeSettingsPanel.tsx` | アップグレードチェック |
| `DiffView.tsx` | 差分比較ビュー |
| `stores/appStore.ts` | グローバル状態（zustand） |
| `hooks/useTauriCommand.ts` | 全 Tauri IPC フック |

---

## 状態管理

`src/stores/appStore.ts`（zustand）が以下のグローバル状態を管理する。

| state | 型 | 用途 |
|-------|----|------|
| `selectedProject` | `ProjectRow \| null` | 現在選択中のプロジェクト |
| `selectedElement` | `SelectedElement \| null` | メインパネルに表示する要素 |
| `searchQuery` | `string` | 検索バーのテキスト |
| `rightPanel` | `RightPanelState \| null` | 右パネルに表示する要素 |
| `navHistory` | `SelectedElement[]` | ←→ ナビゲーション履歴 |
| `navIndex` | `number` | 履歴内の現在位置 |
| `fontSize` | `number` | フォントサイズ（localStorage 永続化） |

`selectedElement` は `searchQuery` より表示優先度が高い。検索結果をクリックして詳細に遷移した後、`←` ボタンで `selectedElement` を null にすると自然に検索結果画面に戻れる。

---

## DB 設計のポイント

### ID の扱い

`search_index.element_id` には SQLite の auto-increment ID（DB ID）を保存する。FileMaker が XML に付与する内部 ID（`fm_id`）とは別物で一致しない。`list_scripts` 等が返す `ScriptRow.id` は DB ID であり、検索結果のナビゲーション時に使用する。

### FTS5 search_index

```sql
CREATE VIRTUAL TABLE search_index USING fts5(
    project_id UNINDEXED,
    element_type UNINDEXED,  -- "table" | "field" | "script" | "layout" | ...
    element_id UNINDEXED,    -- DB auto-increment ID
    name,                    -- 検索対象: 要素名
    content,                 -- 検索対象: 計算式・ステップ内容・述語等
    tokenize='unicode61'
);
```

`content` に格納するもの:

| element_type | content の内容 |
|---|---|
| `script` | 全ステップの `step_text` を結合 |
| `field` | `comment` + `calculation` |
| `layout` | `table_occurrence_name` |
| `relationship` | 述語文字列（`A::x = B::y` 形式） |
| `table_occurrence` | `base_table_name` |
| `custom_function` | `parameters` + `calculation` |
| `value_list` | カスタム値の全テキスト |

検索クエリはカラム指定なしの前方一致（`"word"*`）で `name` と `content` を横断検索する。日本語は `unicode61` トークナイザで文字単位に分割される。

### フィールド検索結果の親テーブル解決

フィールドの検索結果クリック時に親テーブルを特定するため、`search()` クエリで `fields` テーブルと `base_tables` テーブルを LEFT JOIN して `table_id` と `table_name` を付与している。

---

## 既知の制約

- **インポートの重複チェックなし**: 同一の `概要.xml` を複数回インポートすると別エントリとして追加される
- **検索の日本語分割**: `unicode61` トークナイザは文字単位で分割するため、「顧客登録」で検索すると「顧客」「登録」を両方含む名前がヒットする
- **区切り線スクリプト**: FM のスクリプト区切り線は `name="-"` として取り込まれる。壊れた参照・孤立スクリプトの検出対象から除外済み

---

## Tauri IPC コマンド一覧

| コマンド名 | ファイル | 引数 | 戻り値 |
|---|---|---|---|
| `import_solution` | import.rs | `summary_path: String` | `SolutionWithProjects` |
| `import_ddr` | import.rs | `file_path: String` | `ProjectRow` |
| `list_solutions` | analysis.rs | — | `Vec<SolutionRow>` |
| `get_solution_projects` | analysis.rs | `solution_id` | `Vec<ProjectRow>` |
| `delete_solution` | analysis.rs | `solution_id` | `()` |
| `delete_project` | analysis.rs | `project_id` | `()` |
| `get_project_summary` | analysis.rs | `project_id` | `ProjectSummary` |
| `get_broken_refs` | analysis.rs | `project_id` | `Vec<BrokenRef>` |
| `get_report_card` | analysis.rs | `project_id` | `ReportCard` |
| `search_elements` | search.rs | `project_id, query, limit?` | `Vec<SearchResult>` |
| `list_tables` | catalog.rs | `project_id` | `Vec<TableRow>` |
| `list_table_fields` | catalog.rs | `project_id, table_id` | `Vec<FieldRow>` |
| `list_all_fields` | catalog.rs | `project_id` | `Vec<AllFieldRow>` |
| `list_scripts` | catalog.rs | `project_id` | `Vec<ScriptRow>` |
| `list_script_steps` | catalog.rs | `script_id` | `Vec<ScriptStepRow>` |
| `list_layouts` | catalog.rs | `project_id` | `Vec<LayoutRow>` |
| `list_layout_objects` | catalog.rs | `layout_id` | `Vec<LayoutObjectRow>` |
| `list_layout_object_conditions` | catalog.rs | `object_id` | `Vec<ConditionRow>` |
| `list_layout_triggers` | catalog.rs | `layout_id` | `Vec<TriggerRow>` |
| `list_table_occurrences` | catalog.rs | `project_id` | `Vec<TableOccurrenceRow>` |
| `list_relationships` | catalog.rs | `project_id` | `Vec<RelationshipRow>` |
| `list_value_lists` | catalog.rs | `project_id` | `Vec<ValueListRow>` |
| `list_value_list_items` | catalog.rs | `value_list_id` | `Vec<String>` |
| `list_custom_functions` | catalog.rs | `project_id` | `Vec<CustomFunctionRow>` |
| `list_accounts` | catalog.rs | `project_id` | `Vec<AccountRow>` |
| `list_privilege_sets` | catalog.rs | `project_id` | `Vec<PrivilegeSetRow>` |
| `get_field_refs` | field_refs.rs | `project_id, table_name, field_name` | `Vec<FieldRefScript>` |
| `get_field_layout_refs` | field_refs.rs | `project_id, table_name, field_name` | `Vec<FieldRefLayout>` |
| `get_call_chain` | callchain.rs | `project_id, script_id` | `CallChainNode` |
| `get_callers` | callchain.rs | `project_id, script_id` | `Vec<i64>` |
| `get_orphan_scripts` | callchain.rs | `project_id` | `Vec<OrphanScript>` |
| `compare_projects` | diff.rs | `project_id_a, project_id_b` | `DiffResult` |
| `compare_solutions` | diff.rs | `solution_id_a, solution_id_b` | `DiffResult` |
| `list_all_projects` | diff.rs | — | `Vec<ProjectWithSolution>` |
| `get_upgrade_check` | analysis.rs | `solution_id, check_items` | `Vec<UpgradeHit>` |
