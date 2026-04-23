## Git Commit Rules

[Conventional Commits](https://www.conventionalcommits.org/) に従うこと。

### フォーマット

```
<type>: <description>

[optional body]
```

### type 一覧

| type | 用途 |
|------|------|
| `feat` | 新機能の追加 |
| `fix` | バグ修正 |
| `refactor` | 動作を変えないリファクタリング |
| `test` | テストの追加・修正 |
| `docs` | ドキュメントのみの変更 |
| `chore` | ビルド設定・依存関係等の変更 |
| `ci` | CI 設定の変更 |

### ルール

- description は英語・命令形・小文字始まり・末尾ピリオドなし
- 例: `feat: add csv export to upgrade check panel`
- 例: `fix: use db id instead of fm id in search index`
- `main` への直接コミットは禁止（バージョンバンプ・typo 修正を除く）
