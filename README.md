# FileMaker DDR Viewer

[![CI](https://github.com/august8/filemaker-ddr-viewer/actions/workflows/ci.yml/badge.svg)](https://github.com/august8/filemaker-ddr-viewer/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/august8/filemaker-ddr-viewer/branch/main/graph/badge.svg)](https://codecov.io/gh/august8/filemaker-ddr-viewer)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

FileMaker Database Design Report (DDR) XML を解析・可視化するデスクトップツール。

---

## 動作環境

- **OS**: Windows 10 / 11（64bit）
- **対応 FM バージョン**: FileMaker 17 〜 最新（Claris FileMaker）
- **DDR 形式**: `概要.xml`（FileMaker Pro のメニュー「ツール → データベースデザインレポート」で出力）

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
│   ├── hooks/                   # Tauri IPC フック（ドメイン別に分割）
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
| `docs/decisions/` | 設計判断記録（ADR） |
| `CHANGELOG.md` | バージョン履歴 |

---

## Acknowledgements
AI エージェントの設定は以下のリポジトリをベースにしています。
- [imkohenauser/antigravity-starter-ja](https://github.com/imkohenauser/antigravity-starter-ja)

---

## ライセンス

[MIT](LICENSE)
