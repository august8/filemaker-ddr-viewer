# FM DDR Analyzer — アーキテクチャ

---

## 1. システム全体構成

```
Tauri 2.x
  ├── Rust バックエンド (src-tauri/src/)
  │     parser/ → db/ → analyzer/ → commands/
  └── React フロントエンド (src/)
        types/ → hooks/ → stores/ → components/
```

### データフロー

```
DDR XML ファイル
  → parser::parse_ddr()          # quick-xml ストリーミングパース
  → DdrFile (インメモリ)
  → db::repository::insert_ddr_file()  # SQLite 永続化
  → search_index (FTS5)          # 検索インデックス構築
  → Tauri IPC (invoke)
  → React コンポーネント
```

---

## 2. DB スキーマ設計の重要な判断

### 2.1 ID の扱い（バグ修正履歴あり）

**決定**: `search_index.element_id` には **DBのauto-increment ID** を保存する。FM内部IDは使わない。

**理由**:
- FM内部ID（`table.id.0` 等）は FM が XML 内に付与する整数
- DB auto-increment ID はSQLiteが付与する整数
- 両者は一致しない。FM ID でindexを作ると、フロントエンドの `scripts.find(s => s.id === element_id)` が失敗する
- `list_scripts` が返す `ScriptRow.id` は DB ID

**実装**:
- `insert_field_inner` → `Result<i64>` (DB ID返却)
- `insert_custom_function_inner` → `Result<i64>` (DB ID返却)
- `insert_ddr_file` 内で `Vec<SearchEntry>` に DB ID を収集してからバルク INSERT

### 2.2 FTS5 search_index の設計

```sql
CREATE VIRTUAL TABLE search_index USING fts5(
    project_id UNINDEXED,
    element_type UNINDEXED,  -- "table","field","script","layout","value_list","custom_function"
    element_id UNINDEXED,    -- DB auto-increment ID（FM IDではない）
    name,                    -- 検索対象: 要素名
    content,                 -- 検索対象: 計算式・コメント等
    tokenize='unicode61'
);
```

**重要な決定: 全カラム検索（name + content）**

`build_fts_query` はカラム指定なしの `"word"*` 形式で生成する。
`name`（要素名）と `content`（計算式・ステップ内容・TO名・リレーション述語等）の両方を検索する。

**経緯**:
- 初版では `name:"word"*` で name のみ検索していた
- スクリプトステップ内容・カスタム関数の計算式・TO名・リレーション述語を `content` に格納し、
  横断検索できるようにしたため全カラム検索に変更

```rust
// 現在の実装: 全カラム検索（name + content）
format!("\"{escaped}\"*")
```

**search_index の content に格納するもの**:
- `script`: 全ステップの `step_text` を結合
- `field`: `comment` + `calculation`
- `layout`: `table_occurrence_name`
- `relationship`: predicates の述語文字列
- `table_occurrence`: `base_table_name`
- `custom_function`: `parameters` + `calculation`
- `value_list`: `custom_values` の全値

### 2.3 フィールド検索結果の親テーブル解決

検索結果でフィールドをクリックした際、そのフィールドが属するテーブルを特定するため、
`search()` の SQL に LEFT JOIN を追加している。

```sql
WITH fts AS (SELECT ... FROM search_index WHERE ...)
SELECT f.*,
       fld.table_id,          -- parent_id: base_tables の DB ID
       bt.name AS table_name  -- parent_name: テーブル名
FROM fts f
LEFT JOIN fields fld ON f.element_type = 'field' AND fld.id = f.element_id
LEFT JOIN base_tables bt ON bt.id = fld.table_id
```

- `fld.id = f.element_id`：DB ID 同士の JOIN（FM IDではない）
- フィールド以外の要素では `parent_id`/`parent_name` は NULL

---

## 3. 検索・ナビゲーションの設計

### 3.1 SearchResults コンポーネントのフィルター

```
検索結果 → カテゴリフィルターチップ → 絞り込み表示
```

- `activeType: string | null` のローカル state で管理
- フィルターは DB に問い合わせず、取得済み結果をフロントエンドで絞り込む
- `filteredResults = activeType ? results.filter(r => r.element_type === activeType) : results`

### 3.2 検索結果クリック時のナビゲーション

| element_type | 動作 |
|---|---|
| table | `selectElement({ kind:"table", id })` |
| script | `selectElement({ kind:"script", id })` |
| layout | `selectElement({ kind:"layout", id })` |
| value_list | `selectElement({ kind:"value_list", id })` |
| custom_function | `selectElement({ kind:"custom_function", id })` |
| field | `selectElement({ kind:"table", id: parent_id })` + `setRightPanel({ kind:"field", fieldId, tableId })` |

フィールドはメインパネルに親テーブル詳細を表示しつつ、右パネルにフィールド詳細を出す。

**重要**: `projectId` は `result.project_id`（`SearchResult` が保持）から取得する。
`selectedProject?.id` は使わない。全体・ソリューションスコープでは選択プロジェクトと
異なるプロジェクトの結果が含まれるため、結果自身が持つ `project_id` が正確。

### 3.3 App.tsx の表示優先順位

```
selectedElement が設定されている → 詳細画面
searchQuery が空でない         → SearchResults
それ以外                       → ダッシュボード（ProjectSummary + ReportCard 等）
```

**決定**: `selectedElement` を `searchQuery` より優先する

**理由**:
- 検索結果をクリックして詳細に遷移した後、検索クエリを消さずに残す
- ブラウザバック相当の「←」ボタンで `selectedElement` を null に戻すと
  自然に検索結果画面に戻れる

### 3.4 リストデータの常時フェッチ

`useScriptList`, `useLayoutList`, `useValueListList`, `useCustomFunctionList` は
`projectId` が非 null なら **常時フェッチ**する（`selectedElement.kind` による条件なし）。

**理由**:
- 検索結果クリック直後、データがロードされる前に `scripts.find(...)` が `[]` を返すと
  詳細画面が一瞬表示されず SearchResults に戻る「フラッシュ」が発生する
- プロジェクト選択時点でデータをキャッシュしておくことで即時表示できる
- `isLoading` フラグで「読み込み中...」を表示し、ロード完了まで fallthrough しない

---

## 4. フロントエンドの状態管理

### 4.1 appStore（zustand）

| state | 型 | 用途 |
|---|---|---|
| `selectedProject` | `ProjectRow \| null` | 現在選択中のプロジェクト |
| `selectedElement` | `SelectedElement \| null` | メインパネルに表示する要素 |
| `searchQuery` | `string` | 検索バーのテキスト（常時同期）|
| `rightPanel` | `RightPanelState \| null` | 右パネルに表示する要素 |
| `navHistory` | `SelectedElement[]` | ←→ナビゲーション履歴 |
| `navIndex` | `number` | 現在位置 |
| `fontSize` | `number` | フォントサイズ（localStorage 永続化）|

### 4.2 SelectedElement の種類

```typescript
type SelectedElement =
  | { kind: "all_fields"; projectId }              // フィールド横断一覧
  | { kind: "table"; projectId; id; name }         // テーブル詳細
  | { kind: "script"; projectId; id; name }        // スクリプト詳細
  | { kind: "layout"; projectId; id; name }        // レイアウト詳細
  | { kind: "value_list"; projectId; id; name }    // バリューリスト詳細
  | { kind: "custom_function"; projectId; id; name } // カスタム関数詳細
  | { kind: "all_tables"; projectId }              // テーブル一覧パネル
  | { kind: "all_scripts"; projectId }             // スクリプト一覧パネル
  | { kind: "all_layouts"; projectId }             // レイアウト一覧パネル
  | { kind: "all_value_lists"; projectId }         // バリューリスト一覧パネル
  | { kind: "all_custom_functions"; projectId }    // カスタム関数一覧パネル
  | { kind: "all_table_occurrences"; projectId }   // TOパネル（TO/Rel はパネル方式）
  | { kind: "all_relationships"; projectId }       // リレーション一覧パネル
  | { kind: "dashboard" }                          // ダッシュボード（ProjectSummary等）
  | { kind: "diff" }                               // 差分比較ビュー
  | { kind: "search"; query: string }              // 検索状態
  | { kind: "security"; projectId }                // セキュリティ
  | { kind: "relationship_graph"; projectId }      // リレーショングラフ
  | { kind: "upgrade_check"; solutionId }          // アップグレードチェック
  | { kind: "upgrade_settings" }                  // アップグレードチェック設定
  | null
```

---

## 5. 実装済み機能一覧

### バックエンド（Rust）

| モジュール | 機能 | 状態 |
|---|---|---|
| `parser/` | DDR XML ストリーミングパース（FM14〜最新） | ✅ |
| `db/schema.rs` | SQLite スキーマ定義・マイグレーション | ✅ |
| `db/repository/` | CRUD・FTS5検索・solution/project管理（サブモジュール構成）| ✅ |
| `db/repository/solution.rs` | SolutionRow/ProjectRow型・solution/project CRUD | ✅ |
| `db/repository/import.rs` | insert_ddr_file・insert_layout_object_condition | ✅ |
| `db/repository/catalog.rs` | list_* クエリ群・全Row型定義 | ✅ |
| `db/repository/search.rs` | SearchResult型・FTS5全カラム検索 | ✅ |
| `commands/import.rs` | 概要.xml 起点の一括インポート・単体インポート | ✅ |
| `commands/search.rs` | FTS5 全文検索（name+content 全カラム） | ✅ |
| `commands/analysis.rs` | ProjectSummary・BrokenRefs・ReportCard・solution/project管理 | ✅ |
| `commands/catalog.rs` | list_* エンティティ一覧取得コマンド群 | ✅ |
| `commands/field_refs.rs` | フィールド参照解析（get_field_refs等） | ✅ |
| `commands/callchain.rs` | CallChain・Callers・OrphanScripts | ✅ |
| `commands/diff.rs` | 2プロジェクト間の差分比較 | ✅ |
| `analyzer/broken_refs.rs` | PerformScript/ScriptTrigger の壊れた参照検出 | ✅ |
| `analyzer/orphans.rs` | 未使用スクリプト検出 | ✅ |
| `analyzer/call_chain.rs` | DFS + 循環検出 | ✅ |
| `analyzer/report_card.rs` | 健全性レポート（Info/Warning/Error） | ✅ |
| `analyzer/diff_engine.rs` | 名前ベース差分比較（テーブルはフィールド単位 detail） | ✅ |

### フロントエンド（React）

| コンポーネント / ファイル | 機能 | 状態 |
|---|---|---|
| `App.tsx` | 3ペイン レイアウト・リサイズ・ナビゲーション | ✅ |
| `components/MainContent.tsx` | メインエリアのルーティング（15 case switch） | ✅ |
| `navigation/CategoryTree.tsx` | サイドバーツリー | ✅ |
| `SearchBar.tsx` | 検索バー | ✅ |
| `SearchResults.tsx` | 検索結果・カテゴリフィルター・クリック遷移 | ✅ |
| `SolutionList.tsx` | ソリューション一覧・削除 | ✅ |
| `ImportButton.tsx` | 概要.xml インポート | ✅ |
| `ProjectSummaryCard.tsx` | 要素数サマリー（数値クリックで対応パネルへ遷移） | ✅ |
| `ReportCard.tsx` | 健全性レポート（クリックで詳細遷移） | ✅ |
| `BrokenRefsList.tsx` | 壊れた参照一覧（クリックで詳細遷移） | ✅ |
| `OrphanScriptsList.tsx` | 孤立スクリプト一覧 | ✅ |
| `detail/TableDetail.tsx` | テーブル詳細・フィールド一覧 | ✅ |
| `detail/FieldDetail.tsx` | フィールド詳細・Where Used | ✅ |
| `detail/ScriptDetail.tsx` | スクリプト詳細・ステップ一覧・CallChain | ✅ |
| `detail/LayoutDetail.tsx` | レイアウト詳細・トリガー・オブジェクト | ✅ |
| `detail/LayoutObjectDetail.tsx` | レイアウトオブジェクト詳細 | ✅ |
| `detail/AllFieldsPanel.tsx` | 全フィールド横断一覧 | ✅ |
| `detail/AllTablesPanel.tsx` | テーブル一覧パネル（CategoryTree → クリック遷移） | ✅ |
| `detail/AllScriptsPanel.tsx` | スクリプト一覧パネル | ✅ |
| `detail/AllLayoutsPanel.tsx` | レイアウト一覧パネル | ✅ |
| `detail/AllValueListsPanel.tsx` | バリューリスト一覧パネル | ✅ |
| `detail/AllCustomFunctionsPanel.tsx` | カスタム関数一覧パネル | ✅ |
| `detail/AllTableOccurrencesPanel.tsx` | テーブルオカレンス一覧パネル | ✅ |
| `detail/AllRelationshipsPanel.tsx` | リレーション一覧パネル | ✅ |
| `detail/SecurityPanel.tsx` | セキュリティ（アカウント・権限セット）| ✅ |
| `detail/UpgradeCheckPanel.tsx` | アップグレードチェック結果一覧 | ✅ |
| `detail/UpgradeSettingsPanel.tsx` | アップグレードチェック設定（ビルトイン/カスタム項目） | ✅ |
| `StatusBar.tsx` | ステータスバー（要素数・検索時間）| ✅ |
| `detail/RelationshipGraphPanel.tsx` | リレーショングラフ（dagre + SVG + pan/zoom）| ✅ |
| `detail/ValueListDetail.tsx` | バリューリスト詳細 | ✅ |
| `detail/CustomFunctionDetail.tsx` | カスタム関数詳細 | ✅ |
| `detail/CallChainTree.tsx` | コールチェーン ツリー表示 | ✅ |
| `detail/WhereUsed.tsx` | 参照元一覧 | ✅ |
| `RightPanel.tsx` | 右パネルコンテナ | ✅ |
| `DiffView.tsx` | 変更差分フルパネルビュー（サイドバー「差分比較」から表示） | ✅ |

---

## 6. 未実装・今後の課題

現在すべて実装済み。

---

## 7. 既知の制約・注意点

### インポート

- インポートは常に `insert_solution` で新規追加される（同一 `summary_path` でも上書きではなく別エントリとして追加）
- `summary_path = None` のインポートも同様に新規追加（重複チェックなし）

### 検索

- `name` + `content` の全カラムが検索対象（スクリプトステップ内容・計算式・TO名・述語等も含む）
- FTS5 は前方一致プレフィックス検索（`"word"*`）
- 日本語は `tokenize='unicode61'` で文字単位分割される
  → 「顧客登録」で検索すると「顧客」「登録」に分割されて両方含む名前がヒットする

### 区切り線スクリプト

- FM のスクリプト区切り線は `name="-"` として取り込まれる
- `broken_refs` / `orphans` の検出対象から除外済み

---

## 8. Tauri IPC コマンド一覧

| コマンド名 | ファイル | 引数 | 戻り値 |
|---|---|---|---|
| `import_solution` | import.rs | `summary_path: String` | `SolutionWithProjects` |
| `import_ddr` | import.rs | `file_path: String` | `ProjectRow` |
| `list_solutions` | analysis.rs | - | `Vec<SolutionRow>` |
| `get_solution_projects` | analysis.rs | `solution_id` | `Vec<ProjectRow>` |
| `delete_solution` | analysis.rs | `solution_id` | `()` |
| `list_projects` | analysis.rs | - | `Vec<ProjectRow>` |
| `delete_project` | analysis.rs | `project_id` | `()` |
| `get_project_summary` | analysis.rs | `project_id` | `ProjectSummary` |
| `get_broken_refs` | analysis.rs | `project_id` | `Vec<BrokenRef>` |
| `get_report_card` | analysis.rs | `project_id` | `ReportCard` |
| `resolve_element_by_name` | analysis.rs | `project_id, element_type, name` | `ElementRef?` |
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
| `resolve_layout_field` | field_refs.rs | `project_id, occurrence_name, field_name` | `FieldLocation?` |
| `get_layout_ref_debug_info` | field_refs.rs | `project_id` | `LayoutRefDebugInfo` |
| `get_call_chain` | callchain.rs | `project_id, script_id` | `CallChainNode` |
| `get_callers` | callchain.rs | `project_id, script_id` | `Vec<i64>` |
| `get_orphan_scripts` | callchain.rs | `project_id` | `Vec<OrphanScript>` |
| `compare_projects` | diff.rs | `project_id_a, project_id_b` | `DiffResult` |
| `compare_solutions` | diff.rs | `solution_id_a, solution_id_b` | `DiffResult` |
| `list_all_projects` | diff.rs | - | `Vec<ProjectWithSolution>` |
| `get_upgrade_check` | analysis.rs | `solution_id, check_items: Vec<CheckItemConfig>` | `Vec<UpgradeHit>` |

---
