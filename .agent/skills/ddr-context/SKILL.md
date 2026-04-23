---
name: ddr-context
description: >
  FileMaker DDR Viewer 固有の設計パターンと注意点を提供する。
  DDR・Tauri・FTS5・IPC・SQLite に関する作業時に自動発動してよい。
---

# DDR Viewer Context Skill

## よくある落とし穴（過去のバグ修正より）

### DB ID vs FM 内部 ID

```
❌ 間違い: search_index.element_id に FM 内部 ID を保存
✅ 正解:   search_index.element_id には DB auto-increment ID を保存

理由: list_scripts が返す ScriptRow.id は DB ID。
     FM ID で index を作ると frontend の
     scripts.find(s => s.id === element_id) が失敗する。
```

### FTS5 検索

```
❌ 間違い: name:"word"*  （カラム指定あり）
✅ 正解:   "word"*       （全カラム検索）

理由: スクリプトステップ内容・計算式・TO名は content カラムに格納されており、
     name のみの検索ではヒットしない。
```

### IPC 呼び出し

```
❌ 間違い: コンポーネントから直接 invoke("list_scripts", ...) を呼ぶ
✅ 正解:   src/hooks/ 配下の適切なドメインファイルのフックを使う

理由: モック・エラーハンドリング・型安全性を一元管理するため。
```

### 表示優先順位

```
selectedElement が設定 → 詳細画面
searchQuery が非空    → SearchResults
それ以外              → ダッシュボード

selectedElement は searchQuery より優先する。
（検索後に詳細へ遷移しても検索状態を保持するため）
```

## Tauri IPC コマンド追加時のチェックリスト

1. `src-tauri/src/commands/` に関数追加
2. `src-tauri/src/lib.rs` の `.invoke_handler()` に登録
3. `src/hooks/` 配下の適切なドメインファイルにフック追加
4. フロントエンド型定義を `src/types/ddr.ts` に追加

## DDR XML 基本情報

### ルート要素とパースの起点
- ルート要素は `<FMPReport>`
- パース処理では、`<FMPReport>` の `version` 属性を判定し、`parser/version.rs` の `VersionAdapter` で FileMaker 17〜最新バージョンの差異を正規化する。

### 重要な参照タイプ（パース・解析対象）
パーサーや解析エンジンで特に注意すべき XML 要素：
- `ScriptCall` — Perform Script ステップでの他スクリプト呼び出し
- `FieldReference` — Set Field や If 条件などでのフィールド参照
- `LayoutField` — レイアウト上へのフィールド配置
- `ScriptTrigger` — レイアウトやオブジェクトに設定されたトリガー
- `CalculationField` — 計算式（フィールドやカスタム関数など）内での参照
- `RelationshipField` — リレーションのキーとして使用されるフィールド
- `CustomFunctionCall` — カスタム関数の呼び出し
