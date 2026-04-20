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
- **全文検索**: FTS5（SQLite 組み込み）
- **データ保存**: rusqlite (SQLite, bundled)
- **グラフ解析**: petgraph
- **フロントエンド可視化**: dagre（レイアウト計算）+ ネイティブ SVG
- **差分表示**: diff2html
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

## エージェントワークフロー

このプロジェクトでは `.claude/agents/` の3エージェントで作業を進める。
各エージェントの詳細ルール（禁止事項・チェック手順等）はそれぞれの `.md` を参照。

| エージェント | ファイル | 役割 |
|-------------|---------|------|
| plan-agent | `.claude/agents/plan-agent.md` | 調査・設計・承認（読み取り専用） |
| impl-agent | `.claude/agents/impl-agent.md` | ブランチ作成・TDD実装（commit/push禁止） |
| check-agent | `.claude/agents/check-agent.md` | CI確認・commit/push/PR作成 |

### 典型的なフロー

> **注意**: エージェントワークフローを使うときは**プランモードで会話を開始しない**。
> プランモードはサブエージェントにも伝播し、impl-agent・check-agentが動けなくなる。
> 各エージェントのsystem promptが役割制限を担うため、オーケストレーター側のプランモードは不要。

```
1. @plan-agent <タスク内容>
   → プラン（ブランチ名・変更ファイル・テスト仕様）を出力

2. プランを確認・承認する

3. @impl-agent <プランの内容をそのまま渡す>
   → 完了報告（変更ファイル・テスト結果）を出力

4. @check-agent
   → PASS: PR作成まで実行 / FAIL: 差し戻しレポートを出力
   → FAILの場合は @impl-agent <差し戻しレポート> で再実行
```

### 最重要ルール（エージェント共通）

- **main への直接 commit/push は絶対禁止**
- **ブランチ名**: `feat/`, `fix/`, `refactor/`, `docs/`, `test/` のいずれかで始める
- **TDD**: テストを先に書いてREDを確認してから実装する
- **ファイルに触れる前にブランチを切る**（ブランチを切る前の編集は禁止）

---

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
1. プランモードで作業範囲を確認・承認する（必須）
   → plan-agent に委任するか、EnterPlanMode で手動で行う
   - 調査が必要なものはここで全て洗い出し、漏れなく確認する
   - ユーザーの承認を得てからプランモードを抜ける
   - 承認なしにファイル編集・実装に進むことは禁止

2. main から作業ブランチを切る（必須）
   → impl-agent が自動で行う（手動の場合も同じルールに従う）
   - ファイルに一切触れる前にブランチを切ること
   - main への直接コミット・プッシュは絶対禁止

3. 必要な場合のみ ADR を作成する（docs/decisions/NNNN-slug.md）
   - **書くべき条件**（いずれか一つを満たす場合）:
     - 複数の実装案を比較検討した（却下した案がある）
     - 将来「なぜこうなっているのか？」と疑問を持たれそうな非自明な決定
     - DB スキーマ・IPC 設計・パーサー構造など後から覆しにくい決定
   - **書かなくてよいもの**: バグ修正・ライブラリ更新・コードを読めば自明な実装方針
   - ファイル名は `NNNN-slug.md`（連番）。NNNN は docs/decisions/ の最大番号 + 1

4. テストを書いてから実装する（TDD）
   → impl-agent が自動で行う（詳細は `.claude/agents/impl-agent.md` 参照）

5. ADR を作成した場合、実装完了後にステータスを Proposed → Accepted に更新する
   - docs/decisions/README.md のインデックスも追加する

6. ARCHITECTURE.md を最終確認・更新する
   → check-agent が更新漏れを検出する

7. PR を作成して完了とする
   → check-agent が CI確認後にPR作成まで実行する
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
- **型チェック**: `tsc --noEmit`（lefthook で自動チェック）

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

フロントエンドの実際の thresholds（`vite.config.ts` で設定）:

| 指標 | 閾値 |
|------|------|
| statements | 55% |
| branches | 48% |
| functions | 50% |
| lines | 55% |

coverage exclude 設定済みのファイル:
- `src/components/detail/RelationshipGraphPanel.tsx`（SVG描画）
- `src/components/RightPanel.tsx`（純ルーティング）

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
- テストフィクスチャは用途で分けて管理する（`tests/ddr/` にバージョン別DDRサンプル、`tests/fixtures/` に小さな単体テスト用XML）
