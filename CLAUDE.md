# FileMaker DDR Viewer — プロジェクトコンテキスト

## プロジェクト概要

FileMaker DDR (Database Design Report) XMLを解析・可視化する軽量デスクトップツール。
Tauri 2.x + Rust バックエンド + React フロントエンド構成。
OSS として公開中: https://github.com/august8/filemaker-ddr-viewer

## 技術スタック

- **デスクトップ**: Tauri 2.x（OSネイティブWebView）
- **バックエンド**: Rust（全てのビジネスロジック）
- **フロントエンド**: React 19 + TypeScript + Vite + TailwindCSS
- **XML解析**: quick-xml + serde
- **全文検索**: FTS5（SQLite 組み込み）※ `search/` に tantivy 実装が残っているが未使用
- **データ保存**: rusqlite (SQLite, bundled)
- **グラフ解析**: petgraph
- **並列処理**: rayon
- **フロントエンド可視化**: D3.js + dagre-d3
- **状態管理**: zustand
- **API通信**: @tanstack/react-query（Tauri IPC経由）

## プロジェクト構造

```
filemaker-ddr-viewer/
├── src-tauri/                      # Rust バックエンド
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs                 # エントリポイント
│       ├── lib.rs                  # Tauriアプリ初期化・プラグイン登録
│       ├── commands/               # Tauri IPCコマンド（#[tauri::command]）
│       │   ├── mod.rs
│       │   ├── error.rs            # CommandError 型定義
│       │   ├── import.rs           # DDRインポート
│       │   ├── search.rs           # 検索API（FTS5）
│       │   ├── analysis.rs         # 参照解析・壊れた参照
│       │   ├── catalog.rs          # エンティティ一覧取得（list_*コマンド群）
│       │   ├── field_refs.rs       # フィールド参照解析
│       │   ├── callchain.rs        # コールチェーン
│       │   └── diff.rs             # DDR差分比較
│       ├── parser/                 # DDR XMLパーサー
│       │   ├── mod.rs
│       │   ├── ddr_reader.rs       # quick-xmlストリーミングリーダー
│       │   ├── models.rs           # パース結果の型定義
│       │   ├── table_parser.rs     # BaseTableCatalog
│       │   ├── script_parser.rs    # ScriptCatalog
│       │   ├── layout_parser.rs    # LayoutCatalog
│       │   ├── relationship_parser.rs
│       │   ├── catalog_parser.rs   # ValueList/Account/Privilege等
│       │   ├── version.rs          # FMバージョン検出・正規化
│       │   ├── helpers.rs          # パース共通ユーティリティ
│       │   └── summary_parser.rs   # 概要.xml サマリーパーサー
│       ├── analyzer/               # 解析エンジン
│       │   ├── mod.rs
│       │   ├── reference_graph.rs  # petgraph参照グラフ構築
│       │   ├── broken_refs.rs      # 壊れた参照検出
│       │   ├── orphans.rs          # 未使用要素検出
│       │   ├── call_chain.rs       # スクリプト呼び出しチェーン
│       │   ├── report_card.rs      # システム健全性レポート
│       │   └── diff_engine.rs      # DDR差分比較
│       └── db/                     # SQLiteデータ層
│           ├── mod.rs
│           ├── schema.rs           # テーブル定義・マイグレーション
│           └── repository/         # CRUD操作
│               ├── mod.rs
│               ├── import.rs       # DDRデータ挿入
│               ├── catalog.rs      # エンティティ取得
│               ├── search.rs       # FTS5検索
│               └── solution.rs     # ソリューション・プロジェクト管理
│
├── src/                            # React フロントエンド
│   ├── App.tsx
│   ├── main.tsx
│   ├── components/
│   │   ├── navigation/CategoryTree.tsx   # サイドバーツリー
│   │   ├── MainContent.tsx         # メインエリアルーティング
│   │   ├── SearchBar.tsx
│   │   ├── SearchResults.tsx
│   │   ├── SolutionList.tsx
│   │   ├── ImportButton.tsx
│   │   ├── ProjectSummaryCard.tsx
│   │   ├── ReportCard.tsx
│   │   ├── BrokenRefsList.tsx
│   │   ├── OrphanScriptsList.tsx
│   │   ├── UnusedFieldsList.tsx
│   │   ├── DiffView.tsx
│   │   ├── DiffCard.tsx
│   │   ├── RightPanel.tsx
│   │   ├── StatusBar.tsx
│   │   ├── Spinner.tsx
│   │   ├── ErrorBoundary.tsx
│   │   └── detail/
│   │       ├── TableDetail.tsx
│   │       ├── FieldDetail.tsx
│   │       ├── field/              # FieldDetail のサブコンポーネント群
│   │       │   ├── FieldAutoEnter.tsx
│   │       │   ├── FieldBasicProperties.tsx
│   │       │   ├── FieldCalcReferences.tsx
│   │       │   ├── FieldLayoutReferences.tsx
│   │       │   ├── FieldRelationshipReferences.tsx
│   │       │   ├── FieldScriptReferences.tsx
│   │       │   ├── FieldStorage.tsx
│   │       │   └── FieldValidationRules.tsx
│   │       ├── ScriptDetail.tsx
│   │       ├── LayoutDetail.tsx
│   │       ├── LayoutObjectDetail.tsx
│   │       ├── AllTablesPanel.tsx
│   │       ├── AllFieldsPanel.tsx
│   │       ├── AllScriptsPanel.tsx
│   │       ├── AllLayoutsPanel.tsx
│   │       ├── AllTableOccurrencesPanel.tsx
│   │       ├── AllRelationshipsPanel.tsx
│   │       ├── AllValueListsPanel.tsx
│   │       ├── AllCustomFunctionsPanel.tsx
│   │       ├── ValueListDetail.tsx
│   │       ├── CustomFunctionDetail.tsx
│   │       ├── RelationshipGraphPanel.tsx  # D3/dagre グラフ（coverage除外）
│   │       ├── SecurityPanel.tsx
│   │       ├── UpgradeCheckPanel.tsx
│   │       ├── UpgradeSettingsPanel.tsx
│   │       ├── CallChainTree.tsx
│   │       └── WhereUsed.tsx
│   ├── hooks/
│   │   ├── useTauriCommand.ts
│   │   └── useSearchFiltering.ts
│   ├── styles/tokens.ts
│   ├── stores/appStore.ts
│   └── types/ddr.ts
│
├── tests/
│   ├── ddr/                        # バージョン別DDR XMLサンプル（FM17〜22、統合テスト用）
│   │   ├── 17.0.7.700/
│   │   ├── 18.0.3.317/
│   │   ├── 19.6.3.302/
│   │   ├── 20.3.2.201/
│   │   ├── 21.1.2.200/
│   │   └── 22.0.6.601/
│   └── fixtures/                   # 小さな単体テスト用XML（minimal.xml等）
│
├── src-tauri/tests/
│   └── integration_ddr.rs          # 統合テスト
│
├── package.json
├── vite.config.ts
├── tsconfig.json
└── CLAUDE.md                       # このファイル
```

## セッション開始時の必須チェック

**新しいセッションを開始したら、何も実装・提案する前に以下を必ず実行する。**

```
1. ARCHITECTURE.md を読む
   - 実装済み機能・未実装機能の一覧を確認する
   - 既知の制約・注意点を頭に入れる

2. ARCHITECTURE.md の内容を実際のコードで検証する
   - 「実装済み」と書いてある機能が実際にコードに存在するか確認する
   - 型定義・関数シグネチャが記載と一致しているか確認する
   - ドキュメントが古い場合はその場で更新してから進む

3. テスト状況を確認する
   - cargo test / npm run test を実行してパス状態を把握する
```

**このチェックを省略すると、古い情報をもとに実装して手戻りが発生する。**

---

## 開発ワークフロー（必須・厳守）

**すべての機能追加・変更は以下の手順で進める。**

```
1. main から作業ブランチを切る（必須）
   - **ファイルに一切触れる前に** ブランチを切ること。編集してからブランチを切るのは禁止
   - ブランチ名は feat/xxx、fix/xxx、refactor/xxx 等
   - ドキュメント修正・小さな変更であっても例外なし
   - main への直接コミット・プッシュは絶対禁止

2. 必要な場合のみ ADR を作成する（docs/decisions/NNNN-slug.md）
   - **書くべき条件**（いずれか一つを満たす場合）:
     - 複数の実装案を比較検討した（却下した案がある）
     - 将来「なぜこうなっているのか？」と疑問を持たれそうな非自明な決定
     - DB スキーマ・IPC 設計・パーサー構造など後から覆しにくい決定
   - **書かなくてよいもの**: バグ修正・ライブラリ更新・コードを読めば自明な実装方針
   - ファイル名は `NNNN-slug.md`（連番）。NNNN は docs/decisions/ の最大番号 + 1
   - 外部との設計議論が必要な場合は gh issue create で Issue も立てる

3. テストを書いてから実装する（TDD）
   - テストコードで仕様を表現する
   - cargo test / npm run test を実行して Green を確認する

4. ADR を作成した場合、実装完了後にステータスを Proposed → Accepted に更新する
   - docs/decisions/README.md のインデックスも追加する

5. ARCHITECTURE.md を最終確認・更新する
   - 実装で判明した追加の制約・注意点を反映する

6. PR を作成して完了とする
   - CI（fmt / clippy / test）が通ることを確認してから PR を作成する
   - main へのマージはユーザーが CI 確認後に実施する
```

---

## コーディング規約

### Rust

- **エラー処理**: `thiserror` で独自エラー型を定義。`unwrap()` は本番コードで禁止（テストのみ許可）
- **命名規則**: Rust標準（snake_case 関数/変数、PascalCase 型、SCREAMING_SNAKE_CASE 定数）
- **モジュール**: 各モジュールは `mod.rs` で公開インターフェースを明示。内部実装は非公開
- **型安全**: `String` ではなくnewtypeパターンを活用（例: `struct ElementId(i64)`）
- **シリアライズ**: フロントエンド向けの型は必ず `#[derive(Serialize, Deserialize)]`
- **clippy**: `cargo clippy -- -D warnings` を必ずパスすること（lefthook で自動チェック）
- **fmt**: `cargo fmt --check` を必ずパスすること（lefthook で自動チェック）

### TypeScript/React

- **strictモード**: tsconfig.json で `strict: true`
- **コンポーネント**: 関数コンポーネント + hooks のみ。class component 禁止
- **状態管理**: グローバル状態は zustand、サーバー状態は @tanstack/react-query
- **Tauri IPC**: `invoke()` 呼び出しは必ず hooks/ にラップし、コンポーネントから直接呼ばない
- **lint**: ESLint + Prettier

## テスト規約（必須）

### テストは必ず書く

全ての新規コードにはテストが必須。テストなしのPRはマージしない。

### Rust テスト

```bash
cargo test                    # 全テスト実行
cargo test -- --nocapture     # println出力表示
cargo test parser::           # パーサーモジュールのみ
```

**単体テスト**: 各 `.rs` ファイル末尾に `#[cfg(test)] mod tests { ... }` を配置

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn test_basic() { /* ... */ }

    // rstest でパラメータ化テスト
    #[rstest]
    #[case("tests/fixtures/minimal.xml", 3)]
    #[case("tests/fixtures/minimal_summary.xml", 5)]
    fn test_parse_scripts(#[case] path: &str, #[case] expected_count: usize) {
        let content = std::fs::read_to_string(path).unwrap();
        let result = parse_scripts(&content).unwrap();
        assert_eq!(result.len(), expected_count);
    }
}
```

**統合テスト**: `src-tauri/tests/` に配置

**テスト用SQLite**: 必ず `Connection::open_in_memory()` を使用（ファイルI/O不要で高速）

**スナップショットテスト**: `insta` クレートでパース結果のスナップショットを保存

**テスト用クレート（dev-dependencies）**:
- `rstest` — フィクスチャ・パラメータ化テスト
- `insta` — スナップショットテスト
- `proptest` — プロパティベーステスト（XMLバリエーション自動生成）
- `assert_matches` — パターンマッチアサーション

### フロントエンド テスト

```bash
npm run test                  # Vitest実行
npm run test:watch            # ウォッチモード
npm run test -- --coverage    # カバレッジ
```

**コンポーネントテスト**: React Testing Library + Vitest

**Tauri IPC モック**: `vi.mock("@tauri-apps/api/core")` を使用

### カバレッジ目標

| レイヤー | 目標 | 重点領域 |
|---------|------|---------|
| parser/ | 90%+ | 全XMLセクション、バージョン差異、エッジケース |
| analyzer/ | 85%+ | 壊れた参照検出、循環参照、コールチェーン |
| db/ | 80%+ | CRUD操作、マイグレーション |
| commands/ | 70%+ | IPC入出力の型チェック |
| stores/ + hooks/ | 80%+ | ロジック層（状態遷移・enabled ガード・スコープ計算） |
| UIコンポーネント | 60%+ | インタラクション・条件分岐のみ。描画確認テストは不要 |
| D3/純表示 | 除外 | RelationshipGraphPanel 等は coverage exclude 設定済み |

### テスト分類

| テスト種別 | 場所 | 実行頻度 |
|-----------|------|---------|
| 単体テスト | 各 `.rs` ファイル内 | 毎コミット |
| 統合テスト | `src-tauri/tests/` | 毎コミット |
| フロントエンドテスト | `src/__tests__/` | 毎コミット |
| スナップショットテスト | `src-tauri/src/**/snapshots/` | 毎コミット |

## DDR XML 基本情報

### ルート要素
```xml
<FMPReport type="..." version="12.0v1" creationdate="..." creationtime="...">
  <File name="...">
    <BaseTableCatalog>...</BaseTableCatalog>
    <RelationshipGraph>...</RelationshipGraph>
    <LayoutCatalog>...</LayoutCatalog>
    <ScriptCatalog>...</ScriptCatalog>
    <ValueListCatalog>...</ValueListCatalog>
    <AccountCatalog>...</AccountCatalog>
    <PrivilegesCatalog>...</PrivilegesCatalog>
    <ExtendedPrivilegeCatalog>...</ExtendedPrivilegeCatalog>
    <CustomFunctionCatalog>...</CustomFunctionCatalog>
    <Options>...</Options>
  </File>
</FMPReport>
```

### バージョン対応
- FM17〜最新（Claris FileMaker）のDDR XMLに対応
- `<FMPReport>` の `version` 属性からバージョン判定
- `parser/version.rs` の `VersionAdapter` でバージョン差異を正規化

### 重要な参照タイプ
- `ScriptCall` — Perform Script ステップ
- `FieldReference` — Set Field, If条件等
- `LayoutField` — レイアウト上のフィールド配置
- `ScriptTrigger` — レイアウト/オブジェクトのトリガー
- `CalculationField` — 計算式内の参照
- `RelationshipField` — リレーションキーフィールド
- `CustomFunctionCall` — カスタム関数呼び出し

## 作業進行時の注意

- 新しいモジュールを追加したら、必ず `mod.rs` にエクスポートを追加する
- Tauri IPCコマンドを追加したら、`lib.rs` の `invoke_handler` に登録する
- フロントエンドの型定義（`types/ddr.ts`）はRust側の型と同期を維持する
- テストフィクスチャ（DDR XMLサンプル）は `tests/fixtures/` に集約する
