# Contributing to FM DDR Analyzer

## 開発環境のセットアップ

README.md の「開発環境のセットアップ」を参照してください。

## コーディング規約

### Rust

- **エラー処理**: `thiserror` で独自エラー型を定義。`unwrap()` は本番コードで禁止（テストのみ許可）
- **命名規則**: Rust 標準（snake_case 関数/変数、PascalCase 型、SCREAMING_SNAKE_CASE 定数）
- **モジュール**: 各モジュールは `mod.rs` で公開インターフェースを明示。内部実装は非公開
- **型安全**: `String` ではなく newtype パターンを活用（例: `struct ElementId(i64)`）
- **シリアライズ**: フロントエンド向けの型は必ず `#[derive(Serialize, Deserialize)]`
- **clippy**: `cargo clippy -- -D warnings` をパスすること
- **fmt**: `cargo fmt --check` をパスすること

### TypeScript/React

- **strict モード**: tsconfig.json で `strict: true`
- **コンポーネント**: 関数コンポーネント + hooks のみ。class component 禁止
- **状態管理**: グローバル状態は zustand、サーバー状態は @tanstack/react-query
- **Tauri IPC**: `invoke()` 呼び出しは必ず `hooks/` にラップし、コンポーネントから直接呼ばない
- **lint**: ESLint + Prettier

## テスト

全ての新規コードにはテストが必須です。

### Rust テスト

```bash
cd src-tauri
cargo test                    # 全テスト実行
cargo test -- --nocapture     # println 出力表示
cargo test parser::           # パーサーモジュールのみ
```

単体テストは各 `.rs` ファイル末尾に配置します:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() { /* ... */ }
}
```

統合テストは `src-tauri/tests/` に配置してください。

テスト用 SQLite は必ず `Connection::open_in_memory()` を使用してください。

### フロントエンド テスト

```bash
npm run test           # Vitest 実行
npm run test -- --watch   # ウォッチモード
```

Tauri IPC のモックには `@tauri-apps/api/mocks` の `mockIPC()` を使用してください。

### カバレッジ目標

| レイヤー | 目標 |
|---------|------|
| parser/ | 90%+ |
| analyzer/ | 85%+ |
| search/ | 80%+ |
| db/ | 80%+ |
| commands/ | 70%+ |
| React components | 70%+ |

## コード品質チェック

PR を出す前に以下を実行してください:

```bash
# Rust
cd src-tauri
cargo test
cargo clippy -- -D warnings
cargo fmt --check

# フロントエンド
npm run test
npm run lint
```

## 設計判断について

大きな変更を加える場合は、先に Issue を立てて設計を議論してから実装することを推奨します。

## 注意事項

- 新しいモジュールを追加したら、`mod.rs` にエクスポートを追加する
- Tauri IPC コマンドを追加したら、`lib.rs` の `invoke_handler` に登録する
- フロントエンドの型定義（`types/ddr.ts`）は Rust 側の型と同期を維持する
- テストフィクスチャは `tests/fixtures/` に集約する
