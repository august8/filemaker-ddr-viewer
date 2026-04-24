# Contributing to FileMaker DDR Viewer

## 前提条件

- [Rust](https://www.rust-lang.org/tools/install)（stable、`rustup` 経由を推奨）
- [Node.js](https://nodejs.org/) v18 以上
- [Tauri の依存関係](https://v2.tauri.app/start/prerequisites/)（Windows の場合は WebView2 が必要）

## 開発環境のセットアップ

```bash
git clone https://github.com/august8/filemaker-ddr-viewer.git
cd filemaker-ddr-viewer
npm install        # 依存関係インストール + lefthook（pre-commit フック）を自動設定
npm run tauri dev  # アプリ起動（ホットリロード）
```

## ブランチ運用

```
main          ← 常にリリース可能な状態
  ├── feat/xxx    機能追加
  ├── fix/xxx     バグ修正
  └── refactor/xxx リファクタリング
```

- `main` への直接コミット・プッシュは禁止（バージョンバンプ・typo 修正等の軽微な変更を除く）
- 作業はブランチを切って PR を出す

## コミットメッセージ

[Conventional Commits](https://www.conventionalcommits.org/) に従います。

```
feat: 新機能の追加
fix: バグ修正
refactor: 動作を変えないリファクタリング
test: テストの追加・修正
docs: ドキュメントのみの変更
chore: ビルド設定・依存関係等の変更
ci: CI 設定の変更
```

## コーディング規約

### Rust

- エラー処理は `thiserror` で独自エラー型を定義する。`unwrap()` は本番コードで禁止（テストのみ許可）
- フロントエンドに渡す型は `#[derive(Serialize, Deserialize)]` を付ける
- `cargo clippy -- -D warnings` と `cargo fmt` を必ずパスすること（lefthook で自動チェックされる）

### TypeScript / React

- 関数コンポーネント + hooks のみ。class component は使わない
- `invoke()` の呼び出しは `src/hooks/` のドメイン別ファイルにまとめ、コンポーネントから直接呼ばない
- グローバル状態は zustand、サーバー状態は @tanstack/react-query を使う

## テスト

新規コードにはテストを書いてください。

### Rust

```bash
cd src-tauri
cargo test              # 全テスト
cargo test parser::     # モジュール指定
```

- 単体テストは各 `.rs` ファイル末尾の `#[cfg(test)] mod tests { ... }` に書く
- 統合テストは `src-tauri/tests/` に配置する
- DB を使うテストは `Connection::open_in_memory()` を使う

### フロントエンド

```bash
npm run test             # Vitest（全テスト）
npm run test:watch       # ウォッチモード
npm run test -- --coverage  # カバレッジ
```

- コンポーネントテストは React Testing Library + Vitest で書く
- Tauri IPC のモックは `vi.mock("@tauri-apps/api/core")` を使う
- ロジック（状態遷移・条件分岐）を優先してテストする。UI の描画確認テストは書かない

## PR の出し方

1. ブランチを切って実装・テストを行う
2. `cargo clippy`・`cargo fmt`・`npm run test`・`npm run lint` が通ることを確認する（lefthook が pre-commit で自動実行）
3. PR を作成する。タイトルはコミットメッセージと同じ形式で書く
4. CI が通ったらメンテナーがレビュー・マージする

大きな変更を加える場合は、先に Issue で設計を議論してから実装することを推奨します。

## 設計判断の記録（ADR）

複雑な判断を行った場合は `docs/decisions/NNNN-slug.md` に ADR（Architecture Decision Record）を残してください。

**書くべき場合:**
- 複数の実装案を比較検討した（却下した案がある）
- 将来「なぜこうなっているのか？」と疑問を持たれそうな非自明な決定
- DB スキーマ・IPC 設計など後から覆しにくい決定

**書かなくてよい場合:** バグ修正、ライブラリ更新、コードを読めば自明な実装方針

ファイル名は `NNNN-slug.md`（連番）。`docs/decisions/README.md` のインデックスも更新してください。
