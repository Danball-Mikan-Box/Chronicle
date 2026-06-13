# Chronicle 再構築 進捗メモ

## 完了したタスク

### ✅ データモデル (model/project.rs)
- `TaleEntry { title, file_name, order }`
- `MaterialEntry { title, file_name, category, order }`
- `MaterialCategory` enum: Character, World, Glossary, Timeline, Other(String)
- `ChapterEntry`: `children` 削除, `tales: Vec<TaleEntry>` 追加, `file_name` → `dir_name`
- `Project` に `materials: Vec<MaterialEntry>` 追加
- `DocRef` enum (後で model に追加予定)
  - `Tale { chapter_dir, tale_file, chapter_title, tale_title }`
  - `Material { file_name, title }`
- `ProjectSettings` に追加:
  - `preview_position: PanelPosition { Right, Bottom }`
  - `sidebar_position: SidebarPosition { Left, Right }`
- `ActivityTab` enum: Explorer, Materials

### ✅ ファイルシステム (fs/)
- `fs/project.rs`: materials ディレクトリ作成対応
- `fs/chapter.rs`: 章ディレクトリ＋話ファイル構造に書き換え
  - `save_tale`, `load_tale`
  - `rename_chapter_dir`, `rename_tale_file`
  - `delete_chapter_dir`, `delete_tale_file`
- `fs/material.rs`: 新規ファイル
  - `save_material`, `load_material`
  - `rename_material_file`, `delete_material_file`

### ✅ サイドバー (components/sidebar.rs)
- Activity Bar: Explorer (📄) / Materials (📋) 切り替え
- Explorer モード:
  - 章ごとに折りたたみ可能なグループ (▾/▸)
  - 各章の下に話のリスト
  - 章/話の追加・削除・リネーム
  - クリックでドキュメントオープン
- Materials モード:
  - 設定資料のリスト (カテゴリバッジ付き)
  - 資料の追加・削除・リネーム

### ✅ タブバー (components/tab_bar.rs) — 新規
- 開いているドキュメントをタブ表示
- アクティブタブの切り替え
- 閉じるボタン (×)
- タブラベル: "章タイトル / 話タイトル" 形式

## 未完了のタスク

### 🔄 Editor 更新 (components/editor.rs)
- DocRef を受け取り、適切な content を表示
- 内容が変更されたら on_content_change を発火
- FormattingBar はそのまま

### 🔄 App 状態管理 (app.rs) — 大規模書き換え
新しいシグナル:
- `open_tabs: Signal<Vec<DocRef>>`
- `active_tab: Signal<DocRef>`
- `tab_content: Signal<HashMap<DocRef, String>>`
- `activity_tab: Signal<ActivityTab>`

コールバック:
- on_open_doc: タブを開く/切り替え
- on_close_tab: タブを閉じる
- content を tab_content と同期 (use_effect で監視)

データ読み込み:
- タブ切り替え時に fs から読み込み
- 自動保存は継続 (3 秒 debounce)

### ✅ パネル配置設定
- Preview 位置: 右 / 下 → CSS クラス `.preview-bottom`
- Sidebar 位置: 左 / 右 → CSS クラス `.sidebar-right`
- 設定は ProjectSettings に保存

### ✅ CSS 追加
- `.activity-bar`, `.activity-btn`
- `.sidebar-content` レイアウト
- `.tab-bar`, `.tab`, `.tab-close`
- `.chapter-group`, `.chapter-toggle`, `.tale-list`
- `.material-list`, `.material-cat-badge`
- `.sidebar-sub-action`, `.sidebar-del-btn`
- Preview position classes

### ✅ DocRef を model に移動
- model/project.rs に `DocRef` enum と `TabState` struct を定義
- sidebar.rs, tab_bar.rs, app.rs から利用

## サイドバーのウィンドウ化 (June 12)
**問題:** 左のプロジェクト構成パネルにタイトルバーがなく、ウィンドウとしての体裁が不十分だった。

**修正:**
- サイドバーに `titlebar` を追加: タブ式のビュー切り替え (📄 章・話 / 📋 設定資料) + 閉じるボタン (×)
- アクティビティバーを削除し、タイトルバーに統合
- `sidebar-content` を `sidebar-body` に改名、タイトルバーと明確に分離
- `on_close` コールバック経由で閉じるボタンがサイドバー非表示をトグル

## パネルリサイズのバグ修正 (June 12)
**問題:** 一方のパネルを小さくしても他方が延長されず、隙間が空く。

**原因:** リサイズ JS が sidebar と preview の `width` を % 計算で設定していたが、editor は `flex: 1` に依存。border/padding の box-model と `Math.round` の累積誤差で合計幅がコンテナと一致しなかった。

**修正:**
- 全 3 パネルを `flex: none` に変更 (CSS 側)
- JS は sidebar と preview の幅を % 計算し、**editor の幅は残りすべて** (`aw - sw - pw`) として設定 → 常に合計 = `aw` を保証
- パネルの表示/非表示トグル時に `apply()` を呼び出して再計算
- preview-bottom モードの垂直リサイズ対応 (clientY)
- `resize` イベント時も 30ms デバウンスで再計算

## ビルド失敗の修正 (June 13)

### 問題1: `ashpd` crate の async runtime 競合
**原因:** `rfd` の default features が `ashpd/async-std` を有効化する一方、`dioxus-desktop` が `ashpd/tokio` を有効化しており、`ashpd` が両方の feature を同時に許可しないためコンパイルエラー。

**修正:** `Cargo.toml` で `rfd` を `default-features = false, features = ["xdg-portal", "tokio"]` に変更。

### 問題2: `libxdo-dev` 未インストール
**原因:** Linux でリンカが `-lxdo` を見つけられずエラー。

**修正:** `sudo apt install libxdo-dev` を実行。

### 問題3: Windows で `xdg-portal` feature が使えない
**原因:** `xdg-portal` feature が `ashpd` → `zbus` → `nix` を引き込み、`nix` は Linux 専用クレートのため Windows でコンパイルエラーになる。

**修正:** `Cargo.toml` で `rfd` を target-specific に分割:
- Linux: `features = ["xdg-portal", "tokio"]`
- その他 (Windows, macOS): `default-features = false`

## IME 対応の改善 (June 13)

**問題:** textarea での日本語入力中、`oninput` が IME 変換中の未確定文字列を content に書き込んでいた。また、`use_effect` の JS `e.value = ...` が編集中の IME を強制キャンセルしていた。

**修正:** `editor.rs`
- `is_composing` signal を追加し、`compositionstart` / `compositionend` で状態を追跡
- `oninput` では `is_composing` が `true` の間は `content.set()` をスキップ → 変換確定後の `input` イベントでのみ更新
- `chapter_version` 変更時の JS による value 設定も `is_composing` 中はスキップ
- フォーマットバー (`do_format`) も変換中は実行しないようガード
- `onkeydown` のショートカット（Ctrl+S/B/I）も `evt.is_composing()` をチェックしてスキップ

## 英数字の縦書き対応 (June 13)

**問題:** 縦書きプレビューで英数字（アルファベット・数字）が適切に表示されていなかった。

**修正:** `preview.rs` + `main.css`
- preview の vertical style を `text-orientation: upright`（全文字正立） + `text-combine-upright: digits 2`（数字2桁まで1文字幅に圧縮）に変更
- `.preview.vertical` CSS class を追加し、縦書き時に適切なスタイルを適用:
  - `text-orientation: upright` で英数字も回転させず正立表示
  - `text-combine-upright: digits 2` で数字を2桁まで1文字幅に圧縮（縦中横）
  - `line-height: 2.4`（段間拡大）、`letter-spacing: 0.25em`（文字間拡大）で詰まりすぎを防止
  - 段落の `text-indent` を除去（横書き用）
  - 見出しの border-bottom → border-right（縦書き用）
  - blockquote の border-left → border-top（縦書き用）
  - table / code / pre は `text-orientation: mixed` で横書き維持

## "Couldn't get key from code: Backquote" の修正 (June 13)

**問題:** `tao` の `raw_key_to_key()` に `grave`/`dead_grave` (バッククォートキー) のマッピングがなく、debug build で `eprintln!` が出力される。`glib::Propagation::Proceed` により WebView への文字入力自体は妨げられないが、不安材料になるため修正。

**修正:** `patches/tao-0.30.8/` に tao のローカルコピーを作成し、`keyboard.rs` の `raw_key_to_key()` に `grave | dead_grave => Some(Key::Character("`"))` を追加。`Cargo.toml` で `[patch.crates-io]` を使用しパッチ版を適用。

## パネルリサイズの改善 (June 13)

**問題:** ウィンドウリサイズ時にパネル（sidebar, editor, preview）の幅が絶対px指定のままで崩れる。preview-bottom モードで `!important` が JS の設定を上書きしていた。

**修正:**
- resize JS を全面書き換え:
  - `ResizeObserver` でウィンドウリサイズを確実に検出（debounce付き `resize` イベントから変更）
  - パーセント計算 + 絶対px から、現在の幅を読み取って min/max でクランプする方式に変更
  - ドラッグリサイズもpx直接操作に（従来のパーセント計算方式から）
  - editor の幅は JS で設定せず CSS flex 任せに
- CSS 改善:
  - `flex: none` → `flex: 1 1 0px` に変更し、エディタが残り幅を自動で埋める
  - preview-bottom の `!important` を削除
  - sidebar / preview-pane に min/max-width を CSS で設定

## ダイアログ改善 (June 13)
- **Escape キー** でダイアログを閉じられるように
- **オーバーレイクリック** でダイアログを閉じられるように
- リネームダイアログのタイトルを動的に変更（章名/話名/資料名）

## エディタ改善 (June 13)
- **Escape キー** でテキストエリアのフォーカスを解除（`.blur()`）

## 改行の反映改善 (June 13)

**問題:** Markdown 標準のパースでは単一の `\n` はスペースとして扱われ、改行が段落区切りとして反映されない。小説執筆では各行が独立した段落になるのが自然。

**修正:** `renderer.rs` の `preprocess` 関数に改行処理を統合:
- 単一の `\n` → 段落区切り（`\n\n`）に変換。1行ごとに `<p>` タグに。
- `\n\n`（空行1つ） → そのまま段落区切り（従来通り）
- `\n\n\n` 以上（空行2つ以上） → 場面転換（`---` / `<hr>`）としてレンダリング
- コードブロック内の改行は変換せずそのまま保持

## 英数字プレビューの折り返しと列レイアウト修正 (June 13)

**問題:** 
- 縦書き: 英数字を含む長い文章が折り返されず右に無限表示される。CSS Multi-column 導入後は `column-width` が原因で約8字ごとに改行される「意味のわからない改行」になる。
- 横書き: 改行なしの長い英数字列が折り返されない。

**原因:**
- 縦書き: `column-width` は `writing-mode: vertical-rl` においてカラムの**高さ**（インライン方向サイズ）を制御する。`20em` の高さ制限により約8字で次のカラムに送られていた。CSS Multi-column 自体が縦書きで期待通り動作しない。
- 横書き: `overflow-wrap` / `word-break` の未設定により、長い英数字列（URLなど）がブロック幅を超えてオーバーフロー。

**修正:** `main.css`
- 縦書きから CSS Multi-column を完全に除去（`column-width`, `column-fill`, `column-gap` を削除）
- `.preview.vertical`: `overflow: hidden` で溢れを隠し、`writing-mode: vertical-rl` の自然なカラム生成（右端スタート、左へカラム送り）を利用。
- `direction: ltr` を明示指定し、インライン方向（上→下）を確定。
- ベース `.preview` クラスに `overflow-wrap: break-word` を追加 — 横書き時の長い英数字列の折り返しを保証

## プレビュースクロール対応 + 折り返し調整 (June 13)

**問題:**
- 縦書きも横書きも、内容が一定以上長くなるとプレビューでスクロールできない（`overflow: hidden` でクリップされる）
- `overflow-wrap: break-word` / `word-break: break-all` により改行指定のない位置で強制改行が入る（「英数字でプレビューが改行されない問題」の修正の副作用）

**修正:** `main.css`
- 縦書き (`.preview[data-wm="vertical"]`): `overflow-x: auto; overflow-y: hidden` — カラムの横スクロールを許可。縦方向は隠してカラム送りに強制。
- 横書き (`.preview:not([data-wm="vertical"])`): `overflow: auto` — 縦横両方向のスクロールを許可。`height: 100%; box-sizing: border-box` を追加。
- `overflow-wrap: break-word` を横書きから削除 — 単語の途中での強制改行を防止。
- `word-break: break-all` は縦書きに維持 — Latin文字の文字単位折り返しに必要。

## 残っている小さな問題
- 35 個の警告 (主に未使用変数・未使用メソッド) — 動作に影響なし
- sidebar.rs: `t_ch_dir`, `t_ch_ren`, `t_old_file`, `m_file_ren` など — クロージャ内 clone の残骸
- app.rs: `save_notification` が多数のクロージャで clone されているが使われていない
- model/project.rs: `chapter_dir()`, `tab_label()`, `TabState` が未使用

## 動作確認方法
```
cargo run  # 起動後、新規プロジェクト作成 → 章追加 → 話追加 → 編集 → 保存 → プレビュー
```

## ディレクトリ構造 (新)

```
project/
  chronicle.json
  chapters/
    01-序幕/          ← ChapterEntry.dir_name
      01-出会い.md    ← TaleEntry.file_name
      02-旅立ち.md
    02-第一章/
      01-事件.md
  materials/
    キャラクター.md    ← MaterialEntry.file_name
    世界観.md
```
