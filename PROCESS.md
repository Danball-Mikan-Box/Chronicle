# Chronicle 開発過程メモ

## プロジェクト概要

小説執筆支援アプリ。Dioxus 0.6 + Rust 製デスクトップアプリ。

## 既知の問題・修正履歴

### 2026-06-13: タブ・エディタ同期問題の修正

#### 問題1: タブ切替でエディタの内容が更新されない

**症状**: タブを切り替えても同じ文章が表示され続ける。

**原因**:
- `use_effect(use_reactive(&chapter_version, ...))` が Dioxus 0.6 で正しく動作していない可能性
- `chapter_version` が `u32` の値として渡され、参照の変更が検知されない

**修正方法**:
- Editor コンポーネント内で `chapter_version` を `prev_version` と明示的に比較
- 値が変わった場合のみ `spawn` で JS を実行し textarea の内容を更新
- `use_effect(use_reactive(...))` パターンを避け、コンポーネント本体の値比較 + spawn 遅延実行に変更

**ステータス**: 修正済み

#### 問題2: is_saved（保存状態）が正しく表示されない

**症状**: タブ切替直後や保存直後に「未保存」と表示される。

**原因**: `content` の use_effect が全ての content 変更（ユーザ入力＋タブ切替＋保存）で `is_saved = false` にしていた。

**修正**: `is_saved` Signal を Editor に渡し、ユーザ入力時（oninput, 書式ツールバー, 」+Enter）のみ `is_saved = false` に設定。content use_effect から `is_saved.set(false)` を削除。

**ステータス**: 修正済み

#### 問題3: オートセーブが別タブの内容を保存する競合

**原因**: オートセーブの非同期タスクが 3 秒後に `active_tab.read()` を読んでいたため、その間にタブが切り替わると別のファイルに書き込む危険があった。

**修正**: タスク作成時に `doc` をキャプチャ。3 秒後も同じドキュメントに保存する。

**ステータス**: 修正済み

#### 問題4: 章名変更でタブのキーが更新されない

**原因**: `rename_chapter` が旧ディレクトリ名を返していた。さらに `*chapter_dir = old_dir.clone()` と自分自身に代入していた。

**修正**: `rename_chapter` が新ディレクトリ名を返すように変更。tab_content, open_tabs, active_tab のキーを全て新しいディレクトリ名で更新。

**ステータス**: 修正済み

#### 問題5: 削除した話/資料がファイル上に残る

**症状**: 話や資料を削除しても .md ファイルがディスクに残り続ける。

**原因**: 削除ハンドラの `DocRef` 一致条件が誤っていた。
- 削除ハンドラは `(chapter_dir, tale_file)` を受け取るが、`open_tabs.remove` / `tab_content.remove` は `DocRef` 全体（タイトル含む）で一致判定していた
- タイトル変更後に削除すると `DocRef` が一致せず、`tab_content` / `open_tabs` のエントリが残り続ける

**修正**: 全削除ハンドラ（章/話/資料）の一致条件をフィールドベースに変更:
- 話: `chapter_dir + tale_file` のみで一致
- 資料: `file_name` のみで一致
- 章: `chapter_dir` のみで一致
- `tab_content` はキーを直接フィルタリングし削除

**ステータス**: 修正済み

#### 問題6: 削除時に確認ダイアログがない

**症状**: ×ボタン一つで即座に削除される。

**修正**: `ConfirmDialog` コンポーネントを作成（`components/dialog.rs`）。
- `PendingDelete` enum（Chapter / Tale / Material）で削除対象を管理
- 削除ボタンクリック → `pending_delete` Signal を設定 → ConfirmDialog 表示
- 確認後、`on_confirm_delete` が実際の削除を実行
- `danger` CSS クラスで削除ボタンを赤く表示

**ステータス**: 修正済み

#### 問題7: プロジェクト設定の編集UIがない

**症状**: プロジェクト名、作者名、説明、目標文字数などの設定を変更するUIがない。

**修正**: `SettingsDialog` コンポーネントを作成（`components/dialog.rs`）。
- ツールバーに「設定」ボタンを追加
- 編集可能項目: 作品名、作者名、説明、目標文字数、書字方向（縦/横）、プレビュー位置（右/下）、サイドバー位置（左/右）、ダークモード、自動保存、フォントサイズ
- 設定保存時に `fs::project::save_project` で `chronicle.json` に永続化

**ステータス**: 修正済み

#### 問題8: Dioxus のウィンドウ上部バーを消す

**症状**: ネイティブのウィンドウ装飾（タイトルバー）が表示されている。

**修正**:
- `main.rs`: `WindowBuilder::new().with_decorations(false)` を追加
- `components/toolbar.rs`: カスタムタイトルバーを追加
  - `data-drag-region` 属性でドラッグ可能領域を設定
  - 最小化/最大化/閉じるボタンを配置（`use_window()` の API を使用）
  - CSS でタイトルバーのスタイルを定義

**ステータス**: 修正済み

### 2026-06-13: `」` + Enter の自動改行バグ修正

#### 問題
- `」` で改行し自動的に空行が挿入される機能が、2回目以降の Enter で動作しなかった。

#### 原因
- `evaluate_script`（一方通行）+ 合成 `Event('input')` → Dioxus が `Event`（非 `InputEvent`）を拾えず `content` シグナルが未更新に
- 結果、2回目の `」` + Enter で条件 `content.endsWith('」')` が false に

#### 修正
- `evaluate_script` + 合成イベント → `eval()`（双方向）+ Rust で直接 `content.set()` に変更
- JS から `dioxus.send(ta.value)` で値を返し、Rust の `spawn_kd.set(new_val)` で反映

#### ステータス: 修正済み

### 2026-06-13: プレビューの水平線ルール修正

#### 問題
- 3行以上の空行が自動的に `<hr>`（水平線）に変換されていた
- `---` が正しく `<hr>` としてレンダリングされていなかった

#### 修正
- `renderer.rs` の `preprocess()` で `nl_count >= 3` → `\n\n---\n\n` の分岐を削除
- 連続する改行は単に `\n\n` にまとめる（段落区切りのみ）
- 手動で入力した `---` は pulldown-cmark が正しく `<hr>` としてレンダリング
- テスト追加: `test_hrule_from_dashes`、`test_no_hrule_from_blank_lines`

#### ステータス: 修正済み

## アーキテクチャメモ

### タブ・エディタ間のデータフロー

```
App (app.rs)
├── content: Signal<String>           // 編集中のテキスト（アクティブタブの内容）
├── tab_content: Signal<HashMap<DocRef, String>>  // 全タブのキャッシュ
├── active_tab: Signal<Option<DocRef>>            // アクティブなタブ
├── open_tabs: Signal<Vec<DocRef>>                // 開いているタブ一覧
├── is_saved: Signal<bool>                        // 保存状態
├── chapter_version: Signal<u32>                  // タブ切替カウンター
├── pending_delete: Signal<Option<PendingDelete>> // 削除確認保留中
│
├── Editor (content, is_saved, on_save, chapter_version, font_size, placeholder)
│   ├── oninput → content.set() + is_saved.set(false)
│   ├── 書式ツールバー → content.set() + is_saved.set(false)
│   ├── 」+Enter → content.set() + is_saved.set(false)
│   └── chapter_version 変更検知 → JS: textarea.value = content
│
├── Preview (content, writing_mode)
├── Sidebar (project, active_tab, ...)
├── TabBar (open_tabs, active_tab, on_close_tab, on_open_doc)
├── Toolbar (content, project, ...)
│   └── カスタムタイトルバー（最小化/最大化/閉じる + drag-region）
├── StatusBar (project, active_tab, tab_content, is_saved, ...)
├── SettingsDialog → プロジェクト設定の編集（名称、作者、目標字数、書字方向、表示設定）
└── ConfirmDialog → 削除確認
```

### 主要な Signal と役割

| Signal | 型 | 役割 |
|--------|------|---------|
| `content` | `Signal<String>` | エディタの編集中テキスト |
| `tab_content` | `Signal<HashMap<DocRef, String>>` | 全開封タブのテキストキャッシュ |
| `active_tab` | `Signal<Option<DocRef>>` | 現在アクティブなタブ |
| `open_tabs` | `Signal<Vec<DocRef>>` | 開いているタブのリスト |
| `is_saved` | `Signal<bool>` | 保存済みかどうか |
| `chapter_version` | `Signal<u32>` | タブ切替時にインクリメント → Editor の textarea を更新 |
| `pending_delete` | `Signal<Option<PendingDelete>>` | 削除確認ダイアログの状態 |

### タブ切替の流れ

1. ユーザがサイドバー/タブバーでドキュメントをクリック
2. `on_open_doc.call(doc)` → `switch_to_doc(doc)`
3. `switch_to_doc`:
   a. `tab_content` から内容を取得（なければファイルから読み込み）
   b. `content.set(text)` でエディタの内容を更新
   c. `chapter_version += 1` で Editor の textarea 更新をトリガー

### DocRef の一致条件（重要）

`DocRef` は `#[derive(PartialEq, Eq, Hash)]` で全フィールドを比較。
- `DocRef::Tale { chapter_dir, tale_file, chapter_title, tale_title }` - 全て一致が必要
- `DocRef::Material { file_name, title }` - 全て一致が必要

**注意**: 章名/話名を変更すると `chapter_title`/`tale_title` が変わるため、
古い DocRef で `tab_content` を lookup すると見つからない。
rename 時に `tab_content` のキーも更新する必要がある。

**削除時の一致条件**:
- 話: `chapter_dir + tale_file` のみ（タイトル不一致を許容）
- 資料: `file_name` のみ（タイトル不一致を許容）
- 章: `chapter_dir` のみで全話を対象

### 削除確認フロー

1. 削除ボタンクリック → `on_delete_*` が `pending_delete` をセット
2. `ConfirmDialog` が表示される（Escape / キャンセル / オーバーレイクリックで閉じる）
3. 「削除する」クリック → `on_confirm_delete` が実際の削除処理を実行
4. ファイル削除 → `tab_content` / `open_tabs` から除去 → アクティブタブ切替 → `save_project`

### 設定画面フロー

1. ツールバー「設定」ボタンクリック → `on_settings` が現在の設定を読み込み `settings_visible` をセット
2. `SettingsDialog` が表示される
3. 各フィールド編集 → 「保存」クリック → `on_confirm_settings` が設定を永続化
4. ダークモード/自動保存/フォントサイズはアプリ全体に即時反映

## ビルド方法

```bash
cargo run --release
```

## テスト

```bash
cargo test
```
