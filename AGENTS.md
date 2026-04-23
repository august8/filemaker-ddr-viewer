# FileMaker DDR Viewer — Project Guidelines (SSoT)

このファイルはプロジェクト固有の技術ガイドラインを記述する場所です。
エージェントの行動規範は `.agent/rules/` を参照してください。

---

## プロジェクト概要

**FileMaker DDR Viewer** は FileMaker Database Design Report (DDR) XML を解析・可視化する
Tauri 2.x 製デスクトップツールです。

- **リポジトリ**: https://github.com/august8/filemaker-ddr-viewer
- **ライセンス**: MIT
- **対象 OS**: Windows 10/11 (64-bit)
- **スタック**: Rust（バックエンド） + React/TypeScript（フロントエンド）/ Tauri 2.x

---

## アーキテクチャ

```
Tauri 2.x
  ├── Rust バックエンド (src-tauri/src/)
  │     parser/ → db/ → analyzer/ → commands/
  └── React フロントエンド (src/)
        types/ → hooks/ → stores/ → components/
```

### データフロー

```
DDR XML
  → parser::parse_ddr()                    # quick-xml ストリーミングパース
  → DdrFile (インメモリ)
  → db::repository::insert_ddr_file()      # SQLite 永続化
  → search_index (FTS5)                    # 全文検索インデックス構築
  → Tauri IPC (invoke)
  → React コンポーネント
```

---

## 設計上の重要な判断（ADR なしに変更禁止）

| 決定事項 | 内容 |
|---------|------|
| `search_index.element_id` | DB auto-increment ID を保存。FM 内部 ID ではない |
| FTS5 検索 | `name` + `content` 全カラム検索。カラム指定なし |
| 表示優先順位 | `selectedElement` > `searchQuery` > ダッシュボード |
| リストデータ | `projectId` が非 null なら常時フェッチ（フラッシュ防止） |
| IPC 呼び出し | すべて `src/hooks/` 配下のドメイン別ファイルに集約。コンポーネントから直接 `invoke()` しない |

---

## コーディング規約

### Rust

- エラー処理は `thiserror` で独自エラー型を定義する
- **本番コードで `unwrap()` 禁止**（テストのみ許可）
- 型安全のため `String` やプリミティブ型ではなく newtype パターンを適宜活用する（例: `struct ElementId(i64)`）
- 各モジュールは `mod.rs` で公開インターフェースを明示し、内部実装は非公開にする
- フロントエンドに渡す型は `#[derive(Serialize, Deserialize)]` を付ける
- `cargo clippy -- -D warnings` と `cargo fmt --check` を必ずパスすること

### TypeScript / React

- 関数コンポーネント + hooks のみ。class component は使わない
- グローバル状態: Zustand (`appStore`)、サーバー状態: `@tanstack/react-query`
- TypeScript strict mode。`any` 禁止（`unknown` または型定義を使う）
- named export のみ（default export を components から使わない）

---

## 開発コマンド

```bash
# 開発
npm run tauri dev           # アプリ起動（ホットリロード）

# テスト
npm run test                # React テスト（Vitest）
npm run test:watch          # ウォッチモード
cd src-tauri && cargo test --lib   # Rust 単体テスト

# ビルド
npm run build               # フロントエンドビルド（型チェック含む）
npm run tauri build         # プロダクションビルド

# コード品質
cd src-tauri
cargo clippy -- -D warnings
cargo fmt --check
```

---

## テスト方針

### Rust

- 単体テスト: 各 `.rs` ファイル末尾の `#[cfg(test)] mod tests { ... }`
- 統合テスト: `src-tauri/tests/`
- DB テスト: `Connection::open_in_memory()` を使う

### フロントエンド

- React Testing Library + Vitest
- Tauri IPC モック: `vi.mock("@tauri-apps/api/core")`
- ロジック（状態遷移・条件分岐）を優先。UI 描画確認テストは書かない
- **カバレッジ目標**: statements 55%, branches 48%, functions 50%, lines 55%（`vite.config.ts` の thresholds 設定）

---

## ブランチ運用・コミット規約

```
main          ← 常にリリース可能な状態
  ├── feat/xxx
  ├── fix/xxx
  └── refactor/xxx
```

- `main` への直接コミット・プッシュは禁止（バージョンバンプ・typo 修正等の軽微な変更を除く）
- Conventional Commits に従う: `feat:` / `fix:` / `refactor:` / `test:` / `docs:` / `chore:` / `ci:`

---

## 変更時に承認が必要な箇所

以下を変更する場合は、実装前に必ず計画を提示してユーザーの承認を得ること：

1. `src-tauri/src/db/schema.rs` — スキーマ変更は不可逆
2. Tauri IPC コマンドの追加・改名 — フロントエンド/バックエンド契約に影響
3. `src/hooks/` 配下のファイル群 — 全 IPC の集約点
4. `src/stores/appStore.ts` の state 型 — 全コンポーネントに影響

---

## 参照先

- **行動規範**: `.agent/rules/`
- **ワークフロー**: `.agent/workflows/`
- **スキル**: `.agent/skills/`
- **アーキテクチャ詳細**: `ARCHITECTURE.md`
- **コントリビューション**: `CONTRIBUTING.md`
