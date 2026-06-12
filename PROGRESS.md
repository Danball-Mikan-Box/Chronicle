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

## 残っている小さな問題
- 40 個の警告 (主に未使用変数・未使用メソッド) — 動作に影響なし。`cargo fix` で自動修正可能
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
