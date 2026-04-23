## Language Strategies

**[CONFIGURATION]**
- `TARGET_LANGUAGE` = **Japanese**

**1. 内部推論とコード:**
- 内部の思考プロセス（Chain-of-Thought）は精度維持のため英語を許可・推奨する
- ただし、ツール実行時のパラメータ（`TaskName` 等の可視フィールド）は実行直前に `TARGET_LANGUAGE` に翻訳すること
- ソースコード（変数名・クラス名・汎用コメント）は標準的な英語を使用する

**2. ユーザー向け出力:**
- チャットでの応答は `TARGET_LANGUAGE` で行うこと
- 計画・実装ドキュメント（`implementation_plan.md`, `task.md` 等）の内容は `TARGET_LANGUAGE` で記述すること
- エラーメッセージの説明・提案も `TARGET_LANGUAGE` で行うこと

**3. コードコメント:**
- 公開 API・関数の説明コメントは英語を基本とする
- 複雑なロジックの補足コメントは `TARGET_LANGUAGE` でも可
