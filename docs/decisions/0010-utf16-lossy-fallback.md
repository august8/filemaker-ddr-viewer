# 0010: UTF-16 デコード失敗時に lossy フォールバックを追加する

- **ステータス**: Accepted
- **日付**: 2026-04-18

## コンテキスト

`decode_ddr_bytes` の UTF-16 LE/BE パスは `String::from_utf16()` を使っており、孤立サロゲート等の不正なシーケンスを含むファイルを渡すとインポート全体がエラーになる。FileMaker DDR は通常 well-formed だが、ファイルが破損・編集された場合に全体が失敗するのは過剰。

## 決定

`String::from_utf16()` が失敗した場合に `String::from_utf16_lossy()` で再試行し、不正シーケンスを U+FFFD（置換文字）に置換してパースを継続する。UTF-8 パスは変更しない。

## 理由

DDR XML の大半は ASCII/BMP 範囲内であり、不正サロゲートが少数含まれても XML 構造には影響しないことが多い。完全失敗よりも部分的な読み取りの方がユーザーにとって有益。

## 却下した案

UTF-8 も lossy にする → XML パーサー（quick-xml）が不正バイトでエラーを出すため、二重にエラーを隠すことになり混乱しやすい。

## 関連ファイル

- `src-tauri/src/commands/import.rs`
