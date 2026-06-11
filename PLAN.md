# Chronicle — 小説執筆補助ソフト 開発計画

## 概要
Dioxus (Rust) + Markdown を用いた小説執筆に特化したデスクトップアプリケーション。
Obsidian のようなライブプレビュー + 小説向け機能（縦書き/横書き切替、プロジェクト管理、章構成、書式ツールバー）を提供する。

## 技術スタック
- **フレームワーク**: Dioxus 0.6 (desktop, wry WebView)
- **言語**: Rust
- **Markdown パース**: pulldown-cmark
- **状態管理**: Dioxus Signals
- **ファイル管理**: プロジェクト単位でディレクトリ構造 (JSON + Markdown)
- **ファイルダイアログ**: rfd (Rust File Dialog)
- **スタイル**: CSS (vertical-rl / horizontal-tb 切替)

## ディレクトリ構成
```
chronicle/
├── src/
│   ├── main.rs              # エントリポイント
│   ├── app.rs               # メインアプリ (全状態管理)
│   ├── components/
│   │   ├── editor.rs        # Markdown エディタ + 書式ツールバー統合
│   │   ├── preview.rs       # リアルタイムプレビュー
│   │   ├── sidebar.rs       # プロジェクト/章ツリー
│   │   ├── toolbar.rs       # ツールバー (縦横切替、文字数、統計)
│   │   ├── formatting_bar.rs# 書式ツールバー (太字、斜体、見出し等)
│   │   └── dialog.rs        # プロジェクト作成ダイアログ
│   ├── fs/
│   │   ├── project.rs       # プロジェクトファイル操作
│   │   └── chapter.rs       # チャプターファイル操作
│   ├── model/
│   │   └── project.rs       # プロジェクト/章データモデル
│   ├── markdown/
│   │   ├── parser.rs        # pulldown-cmark パース
│   │   └── renderer.rs      # HTML レンダリング + ルビ前処理
│   └── styles/
│       └── main.css         # 全スタイル
├── Cargo.toml
├── PLAN.md
└── README.md
```

## 実装済み機能

### Phase 1: プロジェクト基盤
- [x] Dioxus プロジェクトセットアップ
- [x] ウィンドウ表示 (Desktop)
- [x] 3ペインレイアウト (サイドバー / エディタ / プレビュー)

### Phase 2: エディタ + プレビュー
- [x] Markdown エディタ (textarea)
- [x] 書式ツールバー (太字、斜体、見出し、引用、箇条書き、リンク、ルビ、区切り線)
- [x] リアルタイムHTMLプレビュー
- [x] ルビ対応 ({漢字|かんじ} → `<ruby>`)

### Phase 3: プロジェクト管理
- [x] プロジェクト作成 (rfdファイルダイアログ)
- [x] プロジェクトを開く
- [x] 章の追加/削除
- [x] ファイル保存/読込 (chronicle.json + chapters/*.md)
- [x] 自動セーブ（章切り替え時に保存）
- [x] 保存通知

### Phase 4: 縦書き/横書き対応
- [x] CSS writing-mode 切替 (縦書/横書ボタン)
- [x] 原稿用紙風スタイル
- [x] ルビ対応 (furigana)

### Phase 5: 執筆補助機能
- [x] 文字数カウント（空白除く）
- [x] 目標文字数進捗バー
- [x] 読了時間表示
- [x] 統計表示 (ツールバー右側)

### 未実装（将来）
- [ ] エクスポート (PDF/EPUB/TXT)
- [ ] テンプレート
- [ ] 執筆統計 (日別/週別グラフ)
- [ ] ダークモード
- [ ] スペルチェック
- [ ] キーボードショートカット
- [ ] 全文検索
