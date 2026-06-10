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
| フロントエンド可視化 | dagre（レイアウト計算）+ ネイティブ SVG |
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
│   ├── hooks/                      # ドメイン別 IPC フック
│   ├── stores/appStore.ts          # グローバル状態（zustand）
│   └── types/ddr.ts                # 型定義
├── src-tauri/src/
│   ├── commands/                   # Tauri IPC コマンド
│   ├── parser/                     # DDR XML パーサー
│   ├── db/                         # SQLite データ層
│   └── analyzer/                   # 解析エンジン
├── src-tauri/tests/
│   └── integration_ddr.rs          # 統合テスト
├── tests/ddr/                      # バージョン別 DDR XML サンプル（FM17〜22、統合テスト用）
├── tests/fixtures/                 # 小さな単体テスト用 XML（minimal.xml 等）
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
| `analyzer/reference_graph.rs` | petgraph で参照グラフを構築 |
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
| `navigation/CategoryTree.tsx` | サイドバーツリー。カテゴリ展開時のみ IPC クエリを発行（遅延ロード）。カウントは `useProjectSummary` から取得し未展開時も正確な値を表示 |
| `SearchBar.tsx` / `SearchResults.tsx` | 全文検索・カテゴリフィルター・クリック遷移 |
| `SolutionList.tsx` | ソリューション一覧・削除・ソリューション名変更（ホバー鉛筆ボタン＋インライン編集） |
| `ImportButton.tsx` | `概要.xml` インポート |
| `ProjectSummaryCard.tsx` | 要素数サマリー（プロジェクト単位） |
| `SolutionDashboard.tsx` | ソリューション選択時のサマリー（配下プロジェクト一覧） |
| `ReportCard.tsx` | 健全性レポート |
| `BrokenRefsList.tsx` | 壊れた参照一覧 |
| `OrphanScriptsList.tsx` | 孤立スクリプト一覧 |
| `UnusedFieldsList.tsx` | 未使用フィールド一覧 |
| `DiffCard.tsx` | 差分比較カード（DiffView のサブコンポーネント） |
| `StatusBar.tsx` | 下部ステータスバー |
| `Spinner.tsx` | ローディングスピナー |
| `ErrorBoundary.tsx` | React エラーバウンダリ |
| `detail/TableDetail.tsx` | テーブル詳細・フィールド一覧。タイトルを `shrink-0` ヘッダーに固定し、フィールドテーブルは `flex-1 overflow-auto` 内でスクロール。`useColumnResize` でカラム幅ドラッグ変更可能 |
| `detail/FieldDetail.tsx` | フィールド詳細・Where Used |
| `detail/ScriptDetail.tsx` | スクリプト詳細・ステップ一覧・diff表示 |
| `detail/LayoutDetail.tsx` | レイアウト詳細・トリガー・オブジェクト。タイトル・テーブルオカレンス・トリガーテーブルを `shrink-0` ヘッダーに固定し、オブジェクト一覧は `flex-1 overflow-auto` 内でスクロール。`useColumnResize` でカラム幅ドラッグ変更可能 |
| `detail/LayoutObjectDetail.tsx` | レイアウトオブジェクト詳細 |
| `detail/All*Panel.tsx` | 各エンティティの横断一覧（AllTablesPanel / AllFieldsPanel / AllScriptsPanel / AllLayoutsPanel / AllTableOccurrencesPanel / AllRelationshipsPanel / AllValueListsPanel / AllCustomFunctionsPanel / AllExternalDataSourcesPanel）。全パネル共通でタイトル・フィルター・ページネーションを `shrink-0` ヘッダーに固定し、テーブルは `flex-1 overflow-auto` 内でスクロール。`useColumnResize` でカラム幅ドラッグ変更可能。AllFieldsPanel は `@tanstack/react-virtual` の `useVirtualizer` で仮想スクロール化済み |
| `detail/RelationshipGraphPanel.tsx` | リレーショングラフ（dagre + SVG、pan/zoom 対応） |
| `detail/CallChainTree.tsx` | コールチェーンツリー |
| `detail/WhereUsed.tsx` | 参照元一覧 |
| `detail/SecurityPanel.tsx` | アカウント・権限セット |
| `detail/UpgradeCheckPanel.tsx` / `UpgradeSettingsPanel.tsx` | アップグレードチェック（DBスキャン項目 + 壊れた参照セクション）。壊れた参照は `useSolutionBrokenRefs` で全プロジェクトを集約して表示 |
| `detail/field/FieldBasicProperties.tsx` 他 | FieldDetail のサブコンポーネント群（FieldAutoEnter / FieldCalcReferences / FieldLayoutReferences / FieldRelationshipReferences / FieldScriptReferences / FieldStorage / FieldValidationRules） |
| `DiffView.tsx` | 差分比較ビュー |
| `stores/appStore.ts` | グローバル状態（zustand） |
| `hooks/` | ドメイン別 Tauri IPC フック（analysis / catalog / diff / fieldRefs / layout / script / search / security / solutions / table）。`hooks/analysis.ts` の `useSolutionBrokenRefs` はソリューション全プロジェクトの壊れた参照を `Promise.all` で集約。`useResolveElementByName` は差分ビューのナビゲーション用に要素名から ID を解決する命令型関数を返す hook |
| `hooks/useColumnResize.ts` | テーブルのカラム幅ドラッグリサイズフック。`table-layout:fixed` + `<colgroup>` と組み合わせて使用。ドラッグ開始時に全列幅を DOM から実測して固定し、N 列を広げると N+1 列が縮む（合計幅一定）。最終列のみ単独変更 |
| `hooks/useSearchFiltering.ts` | 検索フィルタリングロジック |
| `styles/tokens.ts` | デザイントークン定数 |

---

## 状態管理

`src/stores/appStore.ts`（zustand）が以下のグローバル状態を管理する。

| state | 型 | 用途 |
|-------|----|------|
| `solutions` | `SolutionRow[]` | ソリューション一覧 |
| `selectedSolution` | `SolutionRow \| null` | 現在選択中のソリューション |
| `selectedProject` | `ProjectRow \| null` | 現在選択中のプロジェクト |
| `selectedElement` | `SelectedElement` | メインパネルに表示する要素（`null` を含むユニオン型） |
| `searchQuery` | `string` | 検索バーのテキスト |
| `rightPanel` | `RightPanelState` | 右パネル1に表示する要素（`null` を含むユニオン型）。`setRightPanel` 呼び出し時に `rightPanel2` も自動クリア |
| `rightPanel2` | `RightPanelState` | 右パネル2（フィールド詳細専用）。`LayoutObjectDetail` / `FieldCalcReferences` でフィールドリンクをクリックすると開く。パネル1が閉じると連動してクリア |
| `navHistory` | `SelectedElement[]` | ←→ ナビゲーション履歴 |
| `navIndex` | `number` | 履歴内の現在位置 |
| `fontSize` | `number` | フォントサイズ（localStorage 永続化） |
| `showAbout` | `boolean` | バージョン情報ダイアログ表示フラグ |
| `showUpgradeSettings` | `boolean` | アップグレード設定ダイアログ表示フラグ |
| `diffState` | `DiffStateData` | 差分比較の選択状態 |
| `diffContext` | `{ compareProjectId: number } \| null` | 差分からのナビゲーション時の比較元 project_id |
| `searchDuration` | `number \| null` | 直近検索の所要時間（ms） |
| `searchContains` | `boolean` | 部分一致検索モード |
| `searchScope` | `"all" \| "solution" \| "project"` | 検索スコープ |
| `checkItems` | `CheckItem[]` | アップグレードチェック設定（localStorage 永続化） |

`selectedElement` は `searchQuery` より表示優先度が高い。検索結果をクリックして詳細に遷移した後、`←` ボタンで `selectedElement` を null にすると自然に検索結果画面に戻れる。

---

## DB 設計のポイント

### ID の扱い

`search_index.element_id` には SQLite の auto-increment ID（DB ID）を保存する。FileMaker が XML に付与する内部 ID（`fm_id`）とは別物で一致しない。`list_scripts` 等が返す `ScriptRow.id` は DB ID であり、検索結果のナビゲーション時に使用する。

### FTS5 search_index

```sql
CREATE VIRTUAL TABLE search_index USING fts5(
    project_id   UNINDEXED,
    element_type,            -- "table" | "field" | "script" | "layout" | ...（インデックス対象）
    element_id   UNINDEXED,  -- DB auto-increment ID
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
| `layout` | `table_occurrence_name` + レイアウト上オブジェクトの `object_name` / `button_label` / `field_name` / `tooltip` / `hide_condition`（壊れたフィールドプレースメントがある場合は `フィールドが見つかりません` も追記） |
| `relationship` | 述語文字列（`A::x = B::y` 形式） |
| `table_occurrence` | `base_table_name` |
| `custom_function` | `parameters` + `calculation` |
| `value_list` | カスタム値の全テキスト |

検索クエリはカラム指定なしの前方一致（`"word"*`）で `name` と `content` を横断検索する。日本語は `unicode61` トークナイザで文字単位に分割される。

### フィールド検索結果の親テーブル解決

フィールドの検索結果クリック時に親テーブルを特定するため、`search()` クエリで `fields` テーブルと `base_tables` テーブルを LEFT JOIN して `table_id` と `table_name` を付与している。

### 分離モデル（プログラムファイル + データファイル）対応

FileMaker の分離モデルでは、レイアウト・スクリプトを持つ「プログラムファイル」と、テーブル・フィールドを持つ「データファイル」が別々の project として同一 solution にインポートされる。

**`fetch_occ_names` のソリューションスコープ化**

`table_occurrences` の検索範囲を同一 `solution_id` の全プロジェクトに拡張することで、データファイルの `project_id` を起点にプログラムファイル側のオカレンス名も取得できる。

```sql
WHERE project_id IN (
  SELECT id FROM projects
  WHERE solution_id = (SELECT solution_id FROM projects WHERE id = ?1)
)
```

これにより `get_field_refs`, `get_field_calc_refs`, `get_field_layout_refs`, `get_field_relationship_keys` がすべてソリューション全体を対象に検索する。

**`resolve_layout_field` のクロスプロジェクト検索**

`resolve_layout_field_inner` は以下の 2 段階で解決する:
1. 同一プロジェクト内で `(occurrence_name, field_name)` を検索
2. 見つからない場合、`table_occurrences.source_file` → 外部プロジェクト名 → `projects.name` で外部プロジェクトを特定してフィールドを検索

**返却型の `project_id` / `field_project_id`**

クロスプロジェクトナビゲーションのため、各返却型にプロジェクト ID を追加している:

| 型 | 追加フィールド | 用途 |
|----|--------------|------|
| `FieldLocation` | `field_project_id` | フィールド定義が属するプロジェクト（外部ファイルの場合に異なる） |
| `FieldRefScript` | `project_id` | スクリプトが属するプロジェクト |
| `FieldRefLayout` | `project_id` | レイアウトが属するプロジェクト |
| `FieldCalcRef` | `project_id` | 参照元フィールドが属するプロジェクト |
| `FieldRelKeyRef` | `project_id` | リレーションが属するプロジェクト |

フロントエンドは `FieldPanelInner` で `fieldProjectId ?? projectId` を使い、`useTableFields` の呼び出しとサブコンポーネントへの `projectId` 渡しを正しいプロジェクトで行う。各参照リストコンポーネントは `ref.project_id` を使って `selectElement` / `setRightPanel` を呼ぶ。

---

## Tauri IPC コマンド追加フロー

新しい IPC コマンドを追加するときは以下の 6 ステップを全て完了すること。
漏れがあるとコンパイルエラーまたはランタイムエラーになる。

| # | 作業 | ファイル |
|---|------|---------|
| 1 | `#[tauri::command]` 関数を定義する | `src-tauri/src/commands/<domain>.rs` |
| 2 | モジュールにエクスポートを追加する | `src-tauri/src/commands/mod.rs` |
| 3 | `invoke_handler` に登録する | `src-tauri/src/lib.rs` |
| 4 | `invoke()` ラッパー hooks を作成する | `src/hooks/<domain>.ts` |
| 5 | TypeScript 型定義を追加する | `src/types/ddr.ts` |
| 6 | Rust 関数のユニットテストを追加する | 同じ `.rs` ファイルの末尾 |

コマンドを追加したら `ARCHITECTURE.md#Tauri IPC コマンド一覧` のテーブルも更新する。

---

## FileMaker バージョン対応

### バージョン命名規則

| 形式 | 例 | 世代 |
|---|---|---|
| `X.Yv<patch>` | `21.0v1`, `19.6v2` | FileMaker Pro 14〜21 |
| `X.Y.Z` | `20.3.2`, `22.0.6` | Claris FileMaker 20〜 |

`parser/version.rs` の `FmVersion::parse()` が両形式を統一的に解析する。

### VersionAdapter パターン

`VersionAdapter` (`parser/version.rs`) はバージョン間の XML 差異を正規化するアダプタ層。
パーサー本体にバージョン分岐を直接書かず、差異は全て `VersionAdapter` のメソッドに集約する。

```rust
let adapter = VersionAdapter::new(version);
let tag = adapter.field_catalog_tag(); // バージョンによって異なる可能性のある値
```

現在実装されているメソッド:

| メソッド | 返す値 | バージョン差異 |
|---|---|---|
| `field_catalog_tag()` | `"FieldCatalog"` | FM17〜22 で変化なし |
| `script_step_tag()` | `"Step"` | FM17〜22 で変化なし |
| `step_list_tag()` | `"StepList"` | FM17〜22 で変化なし |

### 既知のバージョン差異

FM17〜22 の実 DDR サンプルを調査した結果、タグ名・構造に変化はなく追加属性のみ。

| バージョン | 追加された属性・要素 | 対応状況 |
|---|---|---|
| FM19〜（Claris 世代） | `withFewerFolders`（ScriptStep）, `messageCalc` | パーサーで属性を無視（影響なし） |
| FM20〜 | バージョン文字列が `X.Y.Z` 形式に変更 | `FmVersion::parse()` で対応済み |

新しいバージョン差異を発見した場合は `VersionAdapter` にメソッドを追加し、このテーブルを更新する。

### テストサンプル

`tests/ddr/` に各バージョンの実 DDR XML サンプルを格納している（統合テスト用）:

| ディレクトリ | FM バージョン |
|---|---|
| `tests/ddr/17.0.7.700/` | FileMaker Pro 17.0.7 |
| `tests/ddr/18.0.3.317/` | FileMaker Pro 18.0.3 |
| `tests/ddr/19.6.3.302/` | FileMaker Pro 19.6.3 |
| `tests/ddr/20.3.2.201/` | Claris FileMaker 20.3.2 |
| `tests/ddr/21.1.2.200/` | Claris FileMaker 21.1.2 |
| `tests/ddr/22.0.6.601/` | Claris FileMaker 22.0.6 |

単体テスト用の小さな XML は `tests/fixtures/` に配置する。

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
| `write_text_file` | import.rs | `path: String, content: String` | `()` |
| `list_solutions` | analysis.rs | — | `Vec<SolutionRow>` |
| `get_solution_projects` | analysis.rs | `solution_id` | `Vec<ProjectRow>` |
| `delete_solution` | analysis.rs | `solution_id` | `()` |
| `list_projects` | analysis.rs | — | `Vec<ProjectRow>` |
| `delete_project` | analysis.rs | `project_id` | `()` |
| `rename_solution` | analysis.rs | `solution_id, new_name` | `()` |
| `get_project_summary` | analysis.rs | `project_id` | `ProjectSummary` |
| `list_solution_project_summaries` | analysis.rs | `solution_id` | `Vec<ProjectSummary>` |
| `get_broken_refs` | analysis.rs | `project_id` | `Vec<BrokenRef>` |
| `get_report_card` | analysis.rs | `project_id` | `ReportCard` |
| `resolve_element_by_name` | analysis.rs | `project_id, element_type, name` | `Option<ElementRef>` |
| `get_upgrade_check` | analysis.rs | `solution_id, check_items` | `Vec<UpgradeHit>` |
| `search_elements` | search.rs | `project_id?, solution_id?, query, limit?, contains?` | `Vec<SearchResult>` |
| `list_tables` | catalog.rs | `project_id, limit?, offset?` | `Vec<TableRow>` |
| `list_table_fields` | catalog.rs | `project_id, table_id, limit?, offset?` | `Vec<FieldRow>` |
| `list_all_fields` | catalog.rs | `project_id, limit?, offset?` | `Vec<AllFieldRow>` |
| `list_scripts` | catalog.rs | `project_id, limit?, offset?` | `Vec<ScriptRow>` |
| `list_script_steps` | catalog.rs | `script_id, limit?, offset?` | `Vec<ScriptStepRow>` |
| `list_layouts` | catalog.rs | `project_id, limit?, offset?` | `Vec<LayoutRow>` |
| `list_layout_objects` | catalog.rs | `layout_id, limit?, offset?` | `Vec<LayoutObjectRow>` |
| `list_layout_object_conditions` | catalog.rs | `object_id, limit?, offset?` | `Vec<ConditionRow>` |
| `list_layout_triggers` | catalog.rs | `layout_id, limit?, offset?` | `Vec<TriggerRow>` |
| `list_table_occurrences` | catalog.rs | `project_id, limit?, offset?` | `Vec<TableOccurrenceRow>` |
| `list_relationships` | catalog.rs | `project_id, limit?, offset?` | `Vec<RelationshipRow>` |
| `list_value_lists` | catalog.rs | `project_id, limit?, offset?` | `Vec<ValueListRow>` |
| `list_value_list_items` | catalog.rs | `value_list_id, limit?, offset?` | `Vec<String>` |
| `list_custom_functions` | catalog.rs | `project_id, limit?, offset?` | `Vec<CustomFunctionRow>` |
| `list_accounts` | catalog.rs | `project_id, limit?, offset?` | `Vec<AccountRow>` |
| `list_privilege_sets` | catalog.rs | `project_id, limit?, offset?` | `Vec<PrivilegeSetRow>` |
| `list_external_data_sources` | catalog.rs | `project_id, limit?, offset?` | `Vec<ExternalDataSourceRow>` |
| `get_field_refs` | field_refs.rs | `project_id, table_name, field_name` | `Vec<FieldRefScript>` |
| `get_field_calc_refs` | field_refs.rs | `project_id, table_name, field_name` | `Vec<FieldCalcRef>` |
| `get_field_layout_refs` | field_refs.rs | `project_id, table_name, field_name` | `Vec<FieldRefLayout>` |
| `resolve_layout_field` | field_refs.rs | `project_id, occurrence_name, field_name` | `Option<FieldLocation>` |
| `get_field_relationship_keys` | field_refs.rs | `project_id, table_name, field_name` | `Vec<FieldRelKeyRef>` |
| `list_unused_fields` | field_refs.rs | `project_id` | `Vec<UnusedFieldRow>` |
| `get_layout_ref_debug_info` | field_refs.rs | `project_id, layout_id` | デバッグ情報 |
| `get_call_chain` | callchain.rs | `project_id, script_id` | `CallChainNode` |
| `get_callers` | callchain.rs | `project_id, script_id` | `Vec<i64>` |
| `get_orphan_scripts` | callchain.rs | `project_id` | `Vec<OrphanScript>` |
| `compare_projects` | diff.rs | `project_id_a, project_id_b` | `DiffResult` |
| `compare_solutions` | diff.rs | `solution_id_a, solution_id_b` | `DiffResult` |
| `list_all_projects` | diff.rs | — | `Vec<ProjectWithSolution>` |
| `import_ddr_from_path` ⚠️ test-utils | test_utils.rs | `summary_path: String` | `SolutionWithProjects` |

---

## E2E テスト

### 技術選定

Playwright + WebView2 CDP 直接接続を採用（ADR 0011 参照）。
WebView2 は `--remote-debugging-port=9222` で CDP を公開し、
Playwright の `chromium.connectOverCDP()` で接続する。
tauri-driver・msedgedriver 等の外部バイナリは不要。

### ファイルダイアログのバイパス

OS ネイティブのファイルピッカーは自動化ツールから操作できない。Cargo feature フラグ `test-utils` で
テスト専用 IPC コマンド `import_ddr_from_path` を追加し、パスを直接受け取る方式で回避する。

`npx tauri build --debug --features test-utils` でビルドしたバイナリのみにこのコマンドが含まれる。
リリースバイナリ（feature なし）には一切含まれない。

CDP も同様に `test-utils` feature のときのみ有効化される（`main.rs` で env var を設定）。

### ローカル実行手順

```bash
# E2E バイナリのビルド（Rust コード変更後に必要）
npm run build:e2e
# 展開: npx tauri build --debug --features test-utils

# E2E テスト実行（バイナリが既にある場合）
npx playwright test

# ビルドからテストまで一括実行
npm run test:e2e
```

### テスト構成

| ファイル | 内容 |
|---|---|
| `playwright.config.ts` | Playwright 設定 |
| `tests/e2e/global-setup.ts` | アプリ起動・CDP 待機 |
| `tests/e2e/global-teardown.ts` | アプリ終了 |
| `tests/e2e/fixtures.ts` | `connectOverCDP()` による Page fixture |
| `tests/e2e/golden-path.spec.ts` | ゴールデンパス 4 ステップ（インポート → サイドバー表示 → 検索 → 詳細表示） |
| `src-tauri/src/commands/test_utils.rs` | `import_ddr_from_path` コマンド（`test-utils` feature でゲート） |

### 安定したセレクタ

E2E テストは CSS クラス名ではなく以下を優先して使う:

| 用途 | セレクタ |
|---|---|
| 検索結果アイテム | `[data-testid="search-result-item"]` |
| 検索バー | `page.getByPlaceholder(/検索/)` |
| テキスト内容による要素検索 | `page.locator("aside").getByText(name).first()` |
| メインコンテンツエリア | `[data-testid="main-content"]` |
| 詳細パネル（table/script/layout/value_list/custom_function） | `[data-testid="detail-panel"]` |

### CI への追加（現フェーズは対象外）

Tauri バイナリのビルドに約 10 分かかるため、現時点では CI に含めない。
CI 追加手順のコメントは `.github/workflows/ci.yml` に記載済み。
