# 0004: selectedElement を searchQuery より表示優先にする

- **ステータス**: Accepted
- **日付**: 2026-02

## コンテキスト

`App.tsx` の表示切り替えで `selectedElement`（詳細画面）と `searchQuery`（検索結果）のどちらを優先するかを決める必要があった。

## 決定

`selectedElement` → `searchQuery` → ダッシュボード の順で優先する。

## 理由

検索結果をクリックして詳細に遷移した後、`searchQuery` を消さずに残す。`←` ボタンで `selectedElement` を null にすると自然に検索結果画面へ戻れる。

## 却下した案

`searchQuery` を優先する → 詳細画面が表示されず、検索結果が常に上書き表示される。

## 関連ファイル

- `src/App.tsx`
