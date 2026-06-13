use dioxus::prelude::*;
use dioxus_desktop::use_window;

use crate::model::project::{Project, WritingMode};

#[component]
pub fn Toolbar(
    writing_mode: Signal<WritingMode>,
    content: Signal<String>,
    project: Signal<Option<Project>>,
    on_new_project: EventHandler<()>,
    on_open_project: EventHandler<()>,
    on_save: EventHandler<()>,
    on_export: EventHandler<()>,
    on_toggle_dark: EventHandler<()>,
    is_dark: bool,
    show_sidebar: bool,
    show_editor: bool,
    show_preview: bool,
    focus_mode: bool,
    font_size: u32,
    on_toggle_sidebar: EventHandler<()>,
    on_toggle_editor: EventHandler<()>,
    on_toggle_preview: EventHandler<()>,
    on_toggle_focus_mode: EventHandler<()>,
    on_increase_font: EventHandler<()>,
    on_decrease_font: EventHandler<()>,
    on_settings: EventHandler<()>,
) -> Element {
    let text = content.read().clone();
    let char_count = text.chars().filter(|c| !c.is_whitespace()).count();

    let reading_time = if char_count > 0 {
        let mins = (char_count as f64 / 400.0).ceil() as usize;
        if mins < 1 { 1 } else { mins }
    } else {
        0
    };

    let proj_name = project.read().as_ref().map(|p| p.name.clone()).unwrap_or_default();
    let daily_goal = project.read().as_ref().map(|p| p.settings.daily_goal).unwrap_or(1000);
    let progress = if daily_goal > 0 {
        ((char_count as f64 / daily_goal as f64) * 100.0).min(100.0) as usize
    } else {
        0
    };

    let desktop = use_window();
    let min_btn = desktop.clone();
    let max_btn = desktop.clone();
    let close_btn = desktop.clone();

    rsx! {
        div {
            class: "titlebar",
            onmousedown: move |_| { desktop.drag(); },
            span { class: "titlebar-text", "Chronicle" }
            div { class: "titlebar-controls",
                button {
                    class: "titlebar-btn",
                    onmousedown: |e| e.stop_propagation(),
                    onclick: move |_| { min_btn.set_minimized(true); },
                    "\u{2014}"
                }
                button {
                    class: "titlebar-btn",
                    onmousedown: |e| e.stop_propagation(),
                    onclick: move |_| { max_btn.toggle_maximized(); },
                    "\u{25A1}"
                }
                button {
                    class: "titlebar-btn titlebar-close",
                    onmousedown: |e| e.stop_propagation(),
                    onclick: move |_| { close_btn.close(); },
                    "\u{2715}"
                }
            }
        }
        header {
            class: "toolbar",
            div { class: "toolbar-left",
                h1 { "Chronicle" }
                if !proj_name.is_empty() {
                    span { class: "project-name", "{proj_name}" }
                }
            }
            div { class: "toolbar-center",
                button {
                    class: "toolbar-btn",
                    onclick: move |_| on_new_project.call(()),
                    "新規"
                }
                button {
                    class: "toolbar-btn",
                    onclick: move |_| on_open_project.call(()),
                    "開く"
                }
                button {
                    class: "toolbar-btn",
                    disabled: proj_name.is_empty(),
                    onclick: move |_| on_save.call(()),
                    "保存"
                }
                button {
                    class: "toolbar-btn",
                    disabled: proj_name.is_empty(),
                    onclick: move |_| on_settings.call(()),
                    "設定"
                }
                span { class: "separator" }
                button {
                    class: if *writing_mode.read() == WritingMode::Vertical { "toolbar-btn active" } else { "toolbar-btn" },
                    disabled: proj_name.is_empty(),
                    onclick: move |_| writing_mode.set(WritingMode::Vertical),
                    "縦書"
                }
                button {
                    class: if *writing_mode.read() == WritingMode::Horizontal { "toolbar-btn active" } else { "toolbar-btn" },
                    disabled: proj_name.is_empty(),
                    onclick: move |_| writing_mode.set(WritingMode::Horizontal),
                    "横書"
                }
                span { class: "separator" }
                button {
                    class: "toolbar-btn",
                    disabled: proj_name.is_empty(),
                    onclick: move |_| on_export.call(()),
                    "出力"
                }
                button {
                    class: if is_dark { "toolbar-btn active" } else { "toolbar-btn" },
                    onclick: move |_| on_toggle_dark.call(()),
                    if is_dark { "\u{2601}" } else { "\u{2600}" }
                }
                span { class: "separator" }
                button {
                    class: if show_sidebar { "toolbar-btn active" } else { "toolbar-btn" },
                    onclick: move |_| on_toggle_sidebar.call(()),
                    title: "サイドバー",
                    "\u{2630}"
                }
                button {
                    class: if show_editor { "toolbar-btn active" } else { "toolbar-btn" },
                    onclick: move |_| on_toggle_editor.call(()),
                    title: "エディタ",
                    "E"
                }
                button {
                    class: if show_preview { "toolbar-btn active" } else { "toolbar-btn" },
                    onclick: move |_| on_toggle_preview.call(()),
                    title: "プレビュー",
                    "\u{1F441}"
                }
                button {
                    class: if focus_mode { "toolbar-btn active" } else { "toolbar-btn" },
                    onclick: move |_| on_toggle_focus_mode.call(()),
                    title: "集中モード",
                    "\u{26F6}"
                }
                span { class: "separator" }
                div { class: "font-size-controls",
                    button {
                        class: "toolbar-btn",
                        onclick: move |_| on_decrease_font.call(()),
                        title: "文字を小さく",
                        "A-"
                    }
                    span { class: "font-size-value", "{font_size}" }
                    button {
                        class: "toolbar-btn",
                        onclick: move |_| on_increase_font.call(()),
                        title: "文字を大きく",
                        "A+"
                    }
                }
            }
            div { class: "toolbar-right",
                div { class: "stats",
                    div { class: "stat-item", title: "文字数（空白除く）",
                        span { class: "stat-value", "{char_count}" }
                        span { class: "stat-label", "字" }
                    }
                    div { class: "stat-item", title: "目標進捗",
                        span { class: "stat-value", "{progress}" }
                        span { class: "stat-label", "%" }
                    }
                    div { class: "stat-item", title: "読了時間（目安）",
                        span { class: "stat-value", "{reading_time}" }
                        span { class: "stat-label", "分" }
                    }
                }
                div { class: "progress-bar-container",
                    div { class: "progress-bar", style: "width: {progress}%;" }
                }
            }
        }
    }
}
