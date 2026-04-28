---
description: バンプ・CHANGELOG 更新・コミット・タグ作成をまとめて行うリリースフロー
---

# /release — リリースフロー

`/bump` と `/changelog` を実行済みの状態で使うか、このコマンドが自動的に補完する。

## 手順

0. 現在のブランチを確認する:
   - `git branch --show-current` を実行する
   - `main` でない場合: 「⚠️ リリースは main ブランチで実行してください（現在: BRANCH）」と表示して終了する

1. `package.json` から現在バージョン（`VERSION`）を読み取る
2. `git tag --sort=-version:refname | head -1` で最新タグを確認する
   - 現在バージョンとタグが一致している場合: すでにリリース済みなので終了
   - バンプされていない場合: 「先に `/bump` を実行してください」と案内して終了
3. `CHANGELOG.md` に `## [VERSION]` セクションが存在するか確認する
   - 存在しない場合: `/changelog` 相当の処理を実行してから続行する
4. ステージ状態を確認する:
   - `git diff --cached --name-only` で既にステージ済みのファイルを確認する
   - `git diff --name-only` で未ステージの変更ファイルを確認する
   - バージョンファイル（`package.json`, `Cargo.toml`, `tauri.conf.json`）と `CHANGELOG.md` 以外にステージ済み/変更済みのファイルがある場合: ユーザーに警告して確認を求める
5. 以下のファイルを git ステージする（変更がある場合のみ）:
   - `package.json`
   - `src-tauri/Cargo.toml`
   - `src-tauri/tauri.conf.json`
   - `CHANGELOG.md`
6. コミットを作成する:
   ```
   chore: release vVERSION
   ```
7. タグを作成する:
   ```
   git tag vVERSION
   ```
8. 完了を報告する:
   ```
   ✅ vVERSION のリリース準備が完了しました
   
   コミット: chore: release vVERSION
   タグ:     vVERSION
   
   次のステップ（手動で実行）:
     git push --follow-tags origin main
   
   push すると GitHub Actions (release.yml) が自動でビルドと
   GitHub Releases へのアップロードを実行します。
   ```

**安全制約:**
- `git push` は自動実行しない（ユーザーが確認してから実行する）
- `git push --force` は絶対に実行しない

$ARGUMENTS
