# FileMaker DDR Viewer

FileMaker Database Design Report (DDR) XML を解析・可視化するデスクトップツール。

---

## 機能一覧

| カテゴリ | 機能 |
|---|---|
| **インポート** | DDR XML（概要.xml）をソリューション単位でインポート |
| **ブラウズ** | テーブル・フィールド・スクリプト・レイアウト・バリューリスト・カスタム関数・アカウント |
| **検索** | 全文検索（前方一致）＋ 部分一致、スコープ選択（全体 / ソリューション / プロジェクト） |
| **参照解析** | フィールド使用箇所（スクリプト・レイアウト・リレーション）、壊れた参照検出 |
| **グラフ** | リレーショングラフ（TO 間の関係をビジュアル表示） |
| **コールチェーン** | スクリプトの呼び出し関係ツリー、孤立スクリプト検出 |
| **差分比較** | 2 プロジェクト間の差分（追加・削除・変更） |
| **アップグレードチェック** | 非推奨機能・外部スクリプト・カスタム関数呼び出しを一覧化、CSV エクスポート対応 |
| **ナビゲーション** | 戻る / 進む 履歴、マウスサイドボタン対応 |

---

## 動作環境

- **OS**: Windows 10 / 11（64bit）※ Windows 11 または最新の Windows 10 を推奨
- **対応 FM バージョン**: FileMaker 14 〜 最新（Claris FileMaker）
- **DDR 形式**: `概要.xml`（FileMaker Pro の「ファイル → データベースの設計レポート」で出力）

---

## 使い始め方（利用者向け）

1. [リリースページ](https://github.com/august8/filemaker-ddr-viewer/releases)から `filemaker-ddr-viewer.exe` をダウンロード
2. 任意のフォルダに置いてダブルクリックで起動（インストール不要）

> **注意**: 初回起動時に Windows Defender のスマートスクリーンが表示される場合があります。
> 「詳細情報」→「実行」をクリックしてください。

### データの保存場所

インポートしたデータは以下に保存されます：

- **Windows**: `%APPDATA%\filemaker-ddr-viewer\`

データをリセットしたい場合はアプリを終了してからこのフォルダを削除し、再起動してください。

---

## 使い方

### 1. DDR をインポートする

1. FileMaker Pro でデータベースを開き、「ファイル → データベースの設計レポート」を実行
2. 「概要」を選択して XML 形式で出力（複数ファイルをまとめて出力推奨）
3. アプリ上部の「DDR をインポート」ボタンから `概要.xml` を選択
4. インポート完了後、左サイドバーにソリューションとプロジェクトが表示される

### 2. 内容を確認する

- 左サイドバーでソリューション → プロジェクトを選択
- ツリーからテーブル・スクリプト・レイアウト等を選ぶと詳細が表示される
- 上部の検索バーでキーワード検索（全プロジェクト横断）

### 3. アップグレードチェック

- 左サイドバーのソリューション名の下にある「🔍 アップグレードチェック」をクリック
- 検出する項目は右上の設定から変更可能
- 「CSV エクスポート」で結果をファイルに保存できる

---

## 開発環境のセットアップ

### 前提条件

| ツール | バージョン | インストール方法 |
|---|---|---|
| Node.js | 18 以上 | https://nodejs.org/ |
| Rust | 1.77 以上 | https://rustup.rs/ |
| Windows Build Tools | — | `rustup target add x86_64-pc-windows-msvc` |

### セットアップ手順

```bash
# リポジトリをクローン
git clone https://github.com/august8/filemaker-ddr-viewer
cd filemaker-ddr-viewer

# フロントエンド依存関係をインストール
npm install

# 開発サーバー起動（ホットリロード付き）
npm run tauri dev
```

---

## 開発コマンド

```bash
# 開発
npm run tauri dev          # アプリ起動（ホットリロード）

# テスト
npm run test               # React テスト（Vitest）
npm run test:watch         # ウォッチモード
cd src-tauri
cargo test --lib           # Rust 単体テスト

# ビルド
npm run build              # フロントエンドのみビルド（型チェック含む）
npm run tauri build        # プロダクションビルド

# コード品質
cd src-tauri
cargo clippy -- -D warnings   # Rust lint
cargo fmt --check             # Rust フォーマットチェック
```

### プロダクションビルドの成果物

```
src-tauri/target/release/filemaker-ddr-viewer.exe   ← 配布用ポータブル版
```

---

## プロジェクト構成

```
filemaker-ddr-viewer/
├── src/                        # React フロントエンド
│   ├── components/             # UI コンポーネント
│   │   └── detail/             # 各詳細パネル
│   ├── hooks/useTauriCommand.ts # Tauri IPC フック（全 invoke はここ）
│   ├── stores/appStore.ts      # グローバル状態（Zustand）
│   └── types/ddr.ts            # 型定義
├── src-tauri/                  # Rust バックエンド
│   └── src/
│       ├── commands/           # Tauri IPC コマンド
│       ├── parser/             # DDR XML パーサー
│       ├── db/                 # SQLite データ層
│       └── analyzer/           # 解析エンジン
├── ARCHITECTURE.md             # モジュール構成・技術スタック
└── CONTRIBUTING.md             # コントリビューションガイド
```

---

## ドキュメント

| ファイル | 内容 |
|---|---|
| `ARCHITECTURE.md` | モジュール構成・DB スキーマ・IPC コマンド一覧 |
| `CONTRIBUTING.md` | コーディング規約・テスト・PR の出し方 |

---

## エラーが発生した場合

アプリがクラッシュした場合、画面に「エラーが発生しました」と表示されます。
「エラー情報をクリップボードにコピー」ボタンを押して [Issue](https://github.com/august8/filemaker-ddr-viewer/issues) に報告してください。
あわせて、エラーが発生した際に使用していた DDR ファイル（`概要.xml`）を添付いただけると調査が早まります。

## ライセンス

[MIT](LICENSE)
