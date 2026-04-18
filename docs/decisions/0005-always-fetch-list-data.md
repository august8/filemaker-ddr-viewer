# 0005: リストデータをプロジェクト選択時点で常時フェッチする

- **ステータス**: Accepted
- **日付**: 2026-02

## コンテキスト

検索結果クリック直後、データがロードされる前に `scripts.find(...)` が空配列を返し、詳細画面が一瞬表示されず SearchResults に戻る「フラッシュ」が発生した。

## 決定

`useScriptList`・`useLayoutList`・`useValueListList`・`useCustomFunctionList` は `projectId` が非 null なら `selectedElement.kind` に関係なく常時フェッチする。

## 理由

プロジェクト選択時点でキャッシュしておくことで、検索結果クリック時に即時表示できる。

## 却下した案

`selectedElement.kind === "script"` のときだけフェッチ → フラッシュが解消できない。

## 関連ファイル

- `src/hooks/useTauriCommand.ts`
