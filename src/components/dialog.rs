use dioxus::prelude::*;

use crate::model::project::{PanelPosition, ProjectSettings, SidebarPosition, WritingMode};
use crate::model::version::VERSION;
use crate::export::ExportFormat;

#[derive(Debug, Clone)]
pub enum PendingDelete {
    Chapter(String),
    Tale(String, String),
    Material(String),
}

impl PendingDelete {
    pub fn message(&self) -> String {
        match self {
            PendingDelete::Chapter(name) => format!("章「{}」を削除しますか？\nこの操作は元に戻せません。", name),
            PendingDelete::Tale(ch, name) => format!("話「{}」({})を削除しますか？\nこの操作は元に戻せません。", name, ch),
            PendingDelete::Material(name) => format!("資料「{}」を削除しますか？\nこの操作は元に戻せません。", name),
        }
    }
}

#[component]
pub fn ConfirmDialog(
    pending: Signal<Option<PendingDelete>>,
    on_confirm: EventHandler<()>,
) -> Element {
    if pending.read().is_none() {
        return Ok(VNode::placeholder());
    }

    let msg = pending.read().as_ref().map(|p| p.message()).unwrap_or_default();

    let close = move |_| pending.set(None);
    let on_keydown = move |e: Event<KeyboardData>| {
        if e.key() == Key::Escape { pending.set(None); }
    };

    rsx! {
        div { class: "dialog-overlay", onclick: close, onkeydown: on_keydown,
            div { class: "dialog", onclick: |e| e.stop_propagation(),
                h2 { "削除の確認" }
                div { class: "dialog-body",
                    p { "{msg}" }
                }
                div { class: "dialog-actions",
                    button {
                        class: "dialog-btn",
                        onclick: close,
                        "キャンセル"
                    }
                    button {
                        class: "dialog-btn danger",
                        onclick: move |_| {
                            on_confirm.call(());
                        },
                        "削除する"
                    }
                }
            }
        }
    }
}

#[component]
pub fn ExportDialog(
    visible: Signal<bool>,
    on_export: EventHandler<ExportFormat>,
) -> Element {
    if !*visible.read() {
        return Ok(VNode::placeholder());
    }

    let mut show_split_choice = use_signal(|| false);
    let mut show_platform_choice = use_signal(|| false);

    let close = move |_| visible.set(false);
    let on_keydown = move |e: Event<KeyboardData>| {
        if e.key() == Key::Escape { visible.set(false); }
    };

    let formats = [
        (ExportFormat::ProjectZip, "プロジェクト丸ごと (ZIP)", "プロジェクトの全ファイルをZIPに圧縮して出力します。バックアップに最適です。ファイル名は -backup.zip になります。"),
        (ExportFormat::SiteZip, "サイト出力（ZIP）", "HTML サイト形式で出力し、index.html が含まれます。"),
    ];

    rsx! {
        div { class: "dialog-overlay", onclick: close, onkeydown: on_keydown,
            div { class: "dialog dialog-wide", onclick: |e| e.stop_propagation(),
                h2 { "エクスポート" }
                div { class: "dialog-body",
                    p { class: "dialog-description", "出力形式を選択してください。" }
                    div { class: "export-format-list",
                        for (fmt, title, desc) in formats {
                            div {
                                class: "export-format-item",
                                onclick: move |_| {
                                    on_export.call(fmt.clone());
                                    visible.set(false);
                                },
                                h3 { "{title}" }
                                p { "{desc}" }
                            }
                        }
                        div {
                            class: "export-format-item",
                            onclick: move |_| show_split_choice.set(true),
                            h3 { "章・話ごとに分ける (ZIP)" }
                            p { "章ごとにフォルダを分け、各話をファイルとしてZIPにまとめます。テキスト/HTMLを選択。" }
                        }
                        div {
                            class: "export-format-item",
                            onclick: move |_| show_platform_choice.set(true),
                            h3 { "投稿サイト用に出力 (ZIP)" }
                            p { "各投稿サイトの書式に変換してテキストファイルを出力します。" }
                        }
                    }
                }
                div { class: "dialog-actions",
                    button {
                        class: "dialog-btn",
                        onclick: close,
                        "キャンセル"
                    }
                }
            }
        }

        if *show_split_choice.read() {
            div { class: "dialog-overlay", onclick: move |_| show_split_choice.set(false),
                div { class: "dialog", onclick: |e| e.stop_propagation(),
                    h2 { "出力形式を選択" }
                    div { class: "dialog-body", style: "display: flex; justify-content: center; gap: 1rem; padding: 1rem;",
                        button {
                            class: "dialog-btn primary",
                            onclick: move |_| {
                                on_export.call(ExportFormat::ManuscriptZipTxt);
                                visible.set(false);
                                show_split_choice.set(false);
                            },
                            "テキスト (.txt)"
                        }
                        button {
                            class: "dialog-btn primary",
                            onclick: move |_| {
                                on_export.call(ExportFormat::ManuscriptZipHtml);
                                visible.set(false);
                                show_split_choice.set(false);
                            },
                            "HTML (.html)"
                        }
                    }
                    div { class: "dialog-actions",
                        button { class: "dialog-btn", onclick: move |_| show_split_choice.set(false), "キャンセル" }
                    }
                }
            }
        }

        if *show_platform_choice.read() {
            div { class: "dialog-overlay", onclick: move |_| show_platform_choice.set(false),
                div { class: "dialog", onclick: |e| e.stop_propagation(),
                    h2 { "投稿サイトを選択" }
                    div { class: "dialog-body", style: "display: flex; justify-content: center; gap: 1rem; padding: 1rem; flex-wrap: wrap;",
                        button {
                            class: "dialog-btn primary",
                            onclick: move |_| {
                                on_export.call(ExportFormat::NarouZip);
                                visible.set(false);
                                show_platform_choice.set(false);
                            },
                            "小説家になろう"
                        }
                        button {
                            class: "dialog-btn primary",
                            onclick: move |_| {
                                on_export.call(ExportFormat::KakuyomuZip);
                                visible.set(false);
                                show_platform_choice.set(false);
                            },
                            "カクヨム"
                        }
                        button {
                            class: "dialog-btn primary",
                            onclick: move |_| {
                                on_export.call(ExportFormat::HamelnZip);
                                visible.set(false);
                                show_platform_choice.set(false);
                            },
                            "ハーメルン"
                        }
                    }
                    div { class: "dialog-actions",
                        button { class: "dialog-btn", onclick: move |_| show_platform_choice.set(false), "キャンセル" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ProjectDialog(
    visible: Signal<bool>,
    title: String,
    on_confirm: EventHandler<(String, String)>,
) -> Element {
    let mut name = use_signal(|| String::new());
    let mut author = use_signal(|| String::new());

    if !*visible.read() {
        return Ok(VNode::placeholder());
    }

    let close = move |_| visible.set(false);
    let on_keydown = move |e: Event<KeyboardData>| {
        if e.key() == Key::Escape { visible.set(false); }
    };

    rsx! {
        div { class: "dialog-overlay", onclick: close, onkeydown: on_keydown,
            div { class: "dialog", onclick: |e| e.stop_propagation(),
                h2 { "{title}" }
                div { class: "dialog-body",
                    label { "作品名" }
                    input {
                        class: "dialog-input",
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                        placeholder: "作品名を入力",
                    }
                    label { "作者名" }
                    input {
                        class: "dialog-input",
                        value: "{author}",
                        oninput: move |e| author.set(e.value()),
                        placeholder: "作者名（任意）",
                    }
                }
                div { class: "dialog-actions",
                    button {
                        class: "dialog-btn",
                        onclick: close,
                        "キャンセル"
                    }
                    button {
                        class: "dialog-btn primary",
                        disabled: name.read().is_empty(),
                        onclick: move |_| {
                            let n = name.read().clone();
                            let a = author.read().clone();
                            if !n.is_empty() {
                                on_confirm.call((n, a));
                                name.set(String::new());
                                author.set(String::new());
                                visible.set(false);
                            }
                        },
                        "作成"
                    }
                }
            }
        }
    }
}

#[component]
pub fn RenameDialog(
    visible: Signal<bool>,
    file_name: Signal<(String, String)>,
    on_confirm: EventHandler<(String, String)>,
) -> Element {
    if !*visible.read() {
        return Ok(VNode::placeholder());
    }

    let target = file_name.read().clone();
    let old_name = target.0.clone();
    let current_title = target.1.clone();
    let mut new_title = use_signal(|| current_title.clone());

    let dialog_title = if old_name.contains('|') {
        "話名の変更"
    } else if old_name.contains(".md") {
        "資料名の変更"
    } else {
        "章名の変更"
    };

    let close = move |_| visible.set(false);
    let on_keydown = move |e: Event<KeyboardData>| {
        if e.key() == Key::Escape { visible.set(false); }
    };

    let label_text = match dialog_title {
        "話名の変更" => "新しい話名",
        "資料名の変更" => "新しい資料名",
        _ => "新しい章名",
    };
    let placeholder = format!("{}を入力", label_text);

    rsx! {
        div { class: "dialog-overlay", onclick: close, onkeydown: on_keydown,
            div { class: "dialog", onclick: |e| e.stop_propagation(),
                h2 { "{dialog_title}" }
                div { class: "dialog-body",
                    label { "{label_text}" }
                    input {
                        class: "dialog-input",
                        value: "{new_title}",
                        oninput: move |e| new_title.set(e.value()),
                        placeholder: "{placeholder}",
                    }
                }
                div { class: "dialog-actions",
                    button {
                        class: "dialog-btn",
                        onclick: close,
                        "キャンセル"
                    }
                    button {
                        class: "dialog-btn primary",
                        disabled: new_title.read().is_empty(),
                        onclick: move |_| {
                            let t = new_title.read().clone();
                            if !t.is_empty() {
                                on_confirm.call((old_name.clone(), t));
                                visible.set(false);
                            }
                        },
                        "変更"
                    }
                }
            }
        }
    }
}

#[component]
pub fn SettingsDialog(
    visible: Signal<bool>,
    project_name: Signal<String>,
    project_settings: Signal<ProjectSettings>,
    global_settings: Signal<crate::model::project::GlobalSettings>,
    project_is_open: bool,
    on_save: EventHandler<(String, ProjectSettings, crate::model::project::GlobalSettings)>,
) -> Element {
    if !*visible.read() {
        return Ok(VNode::placeholder());
    }

    let settings = project_settings.read().clone();
    let g_settings = global_settings.read().clone();
    let mut name = use_signal(|| project_name.read().clone());
    let mut author = use_signal(|| settings.author.clone());
    let mut description = use_signal(|| settings.description.clone());
    let mut daily_goal = use_signal(|| settings.daily_goal.to_string());
    let mut writing_mode = use_signal(|| settings.writing_mode);
    let mut preview_position = use_signal(|| settings.preview_position);
    let mut sidebar_position = use_signal(|| settings.sidebar_position);
    let mut editor_indent = use_signal(|| settings.indent_paragraphs);

    let mut dark = use_signal(|| g_settings.theme_dark);
    let mut auto_save = use_signal(|| g_settings.auto_save);
    let mut fs = use_signal(|| g_settings.font_size);
    let mut editor_font = use_signal(|| g_settings.font_family.clone());
    let mut editor_lh = use_signal(|| g_settings.line_height.to_string());
    let mut editor_mw = use_signal(|| g_settings.max_width.to_string());

    let close = move |_| visible.set(false);
    let on_keydown = move |e: Event<KeyboardData>| {
        if e.key() == Key::Escape { visible.set(false); }
    };

    rsx! {
        div { class: "dialog-overlay", onclick: close, onkeydown: on_keydown,
            div { class: "dialog dialog-wide", onclick: |e| e.stop_propagation(),
                h2 { "設定" }
                div { class: "dialog-body",
                    if project_is_open {
                        h3 { class: "dialog-section-title", "作品情報" }
                        label { "作品名" }
                        input {
                            class: "dialog-input",
                            value: "{name}",
                            oninput: move |e| name.set(e.value()),
                        }
                        label { "作者名" }
                        input {
                            class: "dialog-input",
                            value: "{author}",
                            oninput: move |e| author.set(e.value()),
                        }
                        label { "説明" }
                        textarea {
                            class: "dialog-input dialog-textarea",
                            value: "{description}",
                            oninput: move |e| description.set(e.value()),
                        }

                        h3 { class: "dialog-section-title", "執筆目標" }
                        label { "1日の目標文字数" }
                        input {
                            class: "dialog-input",
                            value: "{daily_goal}",
                            oninput: move |e| daily_goal.set(e.value()),
                        }
                    }

                    h3 { class: "dialog-section-title", "エディタ・プレビュー設定 (アプリ共通)" }
                    label { "フォント" }
                    select {
                        class: "dialog-input",
                        value: "{editor_font}",
                        onchange: move |e| editor_font.set(e.value()),
                        option { value: "Noto Serif JP", "Noto Serif JP (明朝)" }
                        option { value: "Noto Sans JP", "Noto Sans JP (ゴシック)" }
                        option { value: "Yu Mincho", "游明朝" }
                        option { value: "Yu Gothic", "游ゴシック" }
                        option { value: "monospace", "等幅フォント" }
                    }
                    
                    div { class: "dialog-row",
                        div { class: "dialog-col",
                            label { "行の高さ (倍率)" }
                            input {
                                class: "dialog-input",
                                r#type: "number",
                                step: "0.1",
                                min: "1.0",
                                max: "4.0",
                                value: "{editor_lh}",
                                oninput: move |e| editor_lh.set(e.value()),
                            }
                        }
                        div { class: "dialog-col",
                            label { "最大幅 (px)" }
                            input {
                                class: "dialog-input",
                                r#type: "number",
                                step: "50",
                                min: "400",
                                max: "2000",
                                value: "{editor_mw}",
                                oninput: move |e| editor_mw.set(e.value()),
                            }
                        }
                    }

                    label { class: "dialog-checkbox",
                        input {
                            r#type: "checkbox",
                            checked: *editor_indent.read(),
                            onchange: move |_| { let v = !*editor_indent.read(); editor_indent.set(v); },
                        }
                        "段落の先頭を字下げする"
                    }

                    label { "フォントサイズ: {fs}px" }
                    input {
                        r#type: "range",
                        min: "8",
                        max: "32",
                        value: "{fs}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<u32>() {
                                fs.set(v);
                            }
                        },
                    }

                    h3 { class: "dialog-section-title", "レイアウト・表示" }
                    div { class: "dialog-row",
                        if project_is_open {
                            div { class: "dialog-col",
                                label { "書字方向 (作品固有)" }
                                div { class: "dialog-radio-group",
                                    button {
                                        class: if *writing_mode.read() == WritingMode::Vertical { "dialog-btn primary" } else { "dialog-btn" },
                                        onclick: move |_| writing_mode.set(WritingMode::Vertical),
                                        "縦書き"
                                    }
                                    button {
                                        class: if *writing_mode.read() == WritingMode::Horizontal { "dialog-btn primary" } else { "dialog-btn" },
                                        onclick: move |_| writing_mode.set(WritingMode::Horizontal),
                                        "横書き"
                                    }
                                }
                            }
                        }
                        div { class: "dialog-col",
                            label { "プレビュー位置" }
                            div { class: "dialog-radio-group",
                                button {
                                    class: if *preview_position.read() == PanelPosition::Right { "dialog-btn primary" } else { "dialog-btn" },
                                    onclick: move |_| preview_position.set(PanelPosition::Right),
                                    "右"
                                }
                                button {
                                    class: if *preview_position.read() == PanelPosition::Bottom { "dialog-btn primary" } else { "dialog-btn" },
                                    onclick: move |_| preview_position.set(PanelPosition::Bottom),
                                    "下"
                                }
                            }
                        }
                    }

                    div { class: "dialog-row",
                        div { class: "dialog-col",
                            label { "サイドバー位置" }
                            div { class: "dialog-radio-group",
                                button {
                                    class: if *sidebar_position.read() == SidebarPosition::Left { "dialog-btn primary" } else { "dialog-btn" },
                                    onclick: move |_| sidebar_position.set(SidebarPosition::Left),
                                    "左"
                                }
                                button {
                                    class: if *sidebar_position.read() == SidebarPosition::Right { "dialog-btn primary" } else { "dialog-btn" },
                                    onclick: move |_| sidebar_position.set(SidebarPosition::Right),
                                    "右"
                                }
                            }
                        }
                        div { class: "dialog-col",
                            label { "共通設定" }
                            div { class: "dialog-checkbox-group-stack",
                                label { class: "dialog-checkbox",
                                    input {
                                        r#type: "checkbox",
                                        checked: *dark.read(),
                                        onchange: move |_| { let v = !*dark.read(); dark.set(v); },
                                    }
                                    "ダークモード"
                                }
                                label { class: "dialog-checkbox",
                                    input {
                                        r#type: "checkbox",
                                        checked: *auto_save.read(),
                                        onchange: move |_| { let v = !*auto_save.read(); auto_save.set(v); },
                                    }
                                    "自動保存"
                                }
                            }
                        }
                    }
                }
                div { class: "dialog-version", "Chronicle v{VERSION}" }
                div { class: "dialog-actions",
                    button {
                        class: "dialog-btn",
                        onclick: close,
                        "キャンセル"
                    }
                    button {
                        class: "dialog-btn primary",
                        disabled: project_is_open && name.read().is_empty(),
                        onclick: move |_| {
                            let n = name.read().clone();
                            let goal = daily_goal.read().parse::<usize>().unwrap_or(1000);
                            let lh = editor_lh.read().parse::<f32>().unwrap_or(2.0);
                            let mw = editor_mw.read().parse::<u32>().unwrap_or(800);
                            
                            let settings = ProjectSettings {
                                author: author.read().clone(),
                                description: description.read().clone(),
                                daily_goal: goal,
                                writing_mode: *writing_mode.read(),
                                preview_position: *preview_position.read(),
                                sidebar_position: *sidebar_position.read(),
                                indent_paragraphs: *editor_indent.read(),
                            };

                            let g_settings = crate::model::project::GlobalSettings {
                                theme_dark: *dark.read(),
                                font_size: *fs.read(),
                                auto_save: *auto_save.read(),
                                font_family: editor_font.read().clone(),
                                line_height: lh,
                                max_width: mw,
                            };

                            on_save.call((n, settings, g_settings));
                            visible.set(false);
                        },
                        "保存"
                    }
                }
            }
        }
    }
}
