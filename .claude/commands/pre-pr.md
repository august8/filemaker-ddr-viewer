---
description: PR作成前のスコープ完全性・テスト通過を確認するチェックリストを実行する
---

# /pre-pr — PR作成前チェックリスト

`gh pr create` を実行する前に必ずこのコマンドを実行する。
Claude Code と Codex の確認内容を揃えるため、共通の npm script を使う。

## 実行手順

```bash
npm run agent:pre-pr
```

このコマンドは以下を確認する:

- 変更ファイルの列挙
- 実装変更とテスト変更の対応
- `ARCHITECTURE.md` 更新要否
- 未完了マーカー（`TODO` / `FIXME` / `HACK` / `todo!()` / `unimplemented!()`）
- `npm run test`
- `npm run build`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`

全チェックを通過してから PR を作成すること。

$ARGUMENTS

