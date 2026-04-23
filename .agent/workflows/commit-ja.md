---
description: 現在の変更内容から Conventional Commits 形式のコミットメッセージを生成する（日本語 body）
---

git diff または指定された変更内容をもとに、コミットメッセージを生成すること。

## フォーマット

```
<type>: <英語の短い説明（命令形・小文字・末尾ピリオドなし）>

<変更内容の日本語サマリー（任意）>
```

## ルール

- subject（1行目）は英語・命令形・50文字以内
- body（3行目以降）は日本語で変更の背景・理由を補足してよい
- type は `feat` / `fix` / `refactor` / `test` / `docs` / `chore` / `ci` から選ぶ
- 複数の変更が混在する場合は分割を提案すること

## 出力例

```
fix: use db auto-increment id in search index element_id

FTS5 の element_id に FM 内部 ID を保存していたため、
フロントエンドの `scripts.find(s => s.id === element_id)` が失敗していた。
DB の auto-increment ID に統一することで解消。
```
