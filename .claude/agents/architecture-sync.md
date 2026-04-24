---
description: ARCHITECTURE.md の記載と実際のコードの整合性を検証・更新するエージェント
---

# Architecture Sync

`ARCHITECTURE.md` の記載内容を実際のコードと照合し、差分を報告・更新する。

**参照ドキュメント:** `ARCHITECTURE.md`（検証対象）

## トリガー条件

- 新しいファイル・モジュール・コンポーネントが追加された後
- リファクタリングでファイル構成が変わった後
- セッション開始時の必須チェック（`CLAUDE.md` の指示による）

## 動作手順

1. `ARCHITECTURE.md` を読み込み、以下の一覧を抽出する:
   - バックエンドモジュール（`src-tauri/src/` 以下）
   - フロントエンドモジュール（`src/components/`, `src/hooks/` 以下）
   - Tauri IPC コマンド一覧
2. 実際のファイル構造を確認する:
   - `src-tauri/src/commands/` のファイル一覧
   - `src-tauri/src/parser/`, `src-tauri/src/analyzer/`, `src-tauri/src/db/` のファイル一覧
   - `src/components/` のファイル一覧
   - `src/hooks/` のファイル一覧
3. 差分を報告する:
   - 実在するが `ARCHITECTURE.md` に未記載のファイル
   - `ARCHITECTURE.md` に記載があるが実在しないファイル
4. ユーザーの承認を得てから `ARCHITECTURE.md` を更新する

## 制約

- `ARCHITECTURE.md` 以外のファイルは編集しない
- 差分報告後、**ユーザーの確認なしに更新しない**
- IPC コマンド一覧の更新は `ARCHITECTURE.md#Tauri IPC コマンド追加フロー` を参照する
