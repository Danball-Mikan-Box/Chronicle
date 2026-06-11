# Chronicle — 小説執筆補助ソフト 開発計画

## 概要
Dioxus (Rust) + Markdown を用いた小説執筆に特化したデスクトップアプリケーション。
Obsidian のようなライブプレビュー + 小説向け機能（縦書き/横書き切替、プロジェクト管理、章構成）を提供する。

## 技術スタック
- **フレームワーク**: Dioxus 0.6 (desktop)
- **言語**: Rust
- **Markdown パース**: pulldown-cmark / comrak
- **Markdown レンダリング**: 自前 or Dioxus コンポーネント
- **状態管理**: Dioxus Signals
- **ファイル管理**: プロジェクト単位でディレクトリ構造
- **スタイル**: CSS (vertical-rl / horizontal-tb 切替)

## ディレクトリ構成計画
```
chronicle/
├── src/
│   ├── main.rs          # エントリポイント
│   ├── app.rs           # メインアプリコンポーネント
│   ├── components/      # UI コンポーネント
│   │   ├── editor.rs    # Markdown エディタ
│   │   ├── preview.rs   # プレビュー表示
│   │   ├── sidebar.rs   # プロジェクト/章ツリー
│   │   └── toolbar.rs   # ツールバー
│   ├── model/           # データモデル
│   │   ├── project.rs   # プロジェクト構造
│   │   └── chapter.rs   # 章/話構造
│   ├── markdown/        # Markdown 処理
│   │   ├── parser.rs    # パース
│   │   └── renderer.rs  # レンダリング
│   └── styles/          # CSS
│       └── main.css
├── Cargo.toml
└── README.md
```

## フェーズ分け

### Phase 1: プロジェクト基盤 (今回)
- Dioxus プロジェクトのセットアップ
- ウィンドウ表示
- 最小限の UI レイアウト

### Phase 2: エディタ + プレビュー
- Markdown エディタ (CodeMirror or textarea)
- リアルタイムプレビュー
- ファイル保存/読込

### Phase 3: プロジェクト管理
- プロジェクト作成/開く
- 章・話のツリー表示
- ファイルシステムとの同期

### Phase 4: 縦書き/横書き対応
- CSS writing-mode 切替
- ルビ対応
- 原稿用紙スタイル

### Phase 5: 小説特化機能
- 文字数カウント
- 目標文字数設定
- 執筆統計
- エクスポート (PDF/EPUB)
- テンプレート

## Phase 1 タスク
- [x] 開発計画作成
- [ ] Git リポジトリ初期化
- [ ] Cargo プロジェクト作成
- [ ] Dioxus 依存追加
- [ ] 最小限のアプリ表示
