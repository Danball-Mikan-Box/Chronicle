use dioxus::prelude::*;

use crate::model::project::{Project, WritingMode};
#[cfg(not(target_os = "android"))]
use crate::model::version::VERSION;

#[component]
pub fn Toolbar(
    writing_mode: Signal<WritingMode>,
    content: Signal<String>,
    daily_progress: Signal<usize>,
    project: Signal<Option<Project>>,
    on_new_project: EventHandler<()>,
    on_open_project: EventHandler<()>,
    on_import_project: EventHandler<()>,
    on_close_project: EventHandler<()>,
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
    let file_char_count = text.chars().filter(|c| !c.is_whitespace()).count();
    let today_char_count = *daily_progress.read();
    let mut show_overflow = use_signal(|| false);

    let reading_time = if file_char_count > 0 {
        let mins = (file_char_count as f64 / 400.0).ceil() as usize;
        if mins < 1 { 1 } else { mins }
    } else {
        0
    };

    let proj_name = project.read().as_ref().map(|p| p.name.clone()).unwrap_or_default();
    let daily_goal = project.read().as_ref().map(|p| p.settings.daily_goal).unwrap_or(1000);
    let progress = if daily_goal > 0 {
        ((today_char_count as f64 / daily_goal as f64) * 100.0).min(100.0) as usize
    } else {
        0
    };

    // Desktop title bar – shows version info; window controls are in the native titlebar
    #[cfg(not(target_os = "android"))]
    let titlebar = {
        rsx! {
            div {
                class: "titlebar",
                span { class: "titlebar-text", "Chronicle v{VERSION}" }
            }
        }
    };
    #[cfg(target_os = "android")]
    let titlebar = rsx! {};

    rsx! {
        {titlebar}
        header {
            class: "toolbar",
            div { class: "toolbar-left",
                h1 { "Chronicle" }
                if !proj_name.is_empty() {
                    span { class: "project-name", "{proj_name}" }
                }
            }
            div { class: "toolbar-center",
                button { class: "toolbar-btn keep-visible", disabled: proj_name.is_empty(), onclick: move |_| on_save.call(()), "保存" }
                button { class: "toolbar-btn keep-visible", onclick: move |_| on_settings.call(()), "設定" }
                button { class: "toolbar-btn toolbar-overflow-btn", onclick: move |_| { let v = !*show_overflow.read(); show_overflow.set(v); }, "\u{22EE}" }
                button { class: "toolbar-btn overflow-hidden", onclick: move |_| on_new_project.call(()), "新規" }
                button { class: "toolbar-btn overflow-hidden", onclick: move |_| on_open_project.call(()), "開く" }
                button { class: "toolbar-btn overflow-hidden", onclick: move |_| on_import_project.call(()), "取込" }
                button { class: "toolbar-btn overflow-hidden", disabled: proj_name.is_empty(), onclick: move |_| on_close_project.call(()), "閉じる" }
                span { class: "separator" }
                button { class: if *writing_mode.read() == WritingMode::Vertical { "toolbar-btn overflow-hidden active" } else { "toolbar-btn overflow-hidden" }, disabled: proj_name.is_empty(), onclick: move |_| writing_mode.set(WritingMode::Vertical), "縦書" }
                button { class: if *writing_mode.read() == WritingMode::Horizontal { "toolbar-btn overflow-hidden active" } else { "toolbar-btn overflow-hidden" }, disabled: proj_name.is_empty(), onclick: move |_| writing_mode.set(WritingMode::Horizontal), "横書" }
                span { class: "separator" }
                button { class: "toolbar-btn overflow-hidden", disabled: proj_name.is_empty(), onclick: move |_| on_export.call(()), "出力" }
                button { class: if is_dark { "toolbar-btn overflow-hidden active" } else { "toolbar-btn overflow-hidden" }, onclick: move |_| on_toggle_dark.call(()), if is_dark { "\u{2601}" } else { "\u{2600}" } }
                span { class: "separator" }
                button { class: if show_sidebar { "toolbar-btn overflow-hidden active" } else { "toolbar-btn overflow-hidden" }, onclick: move |_| on_toggle_sidebar.call(()), "\u{2630}" }
                button { class: if show_editor { "toolbar-btn overflow-hidden active" } else { "toolbar-btn overflow-hidden" }, onclick: move |_| on_toggle_editor.call(()), "E" }
                button { class: if show_preview { "toolbar-btn overflow-hidden active" } else { "toolbar-btn overflow-hidden" }, onclick: move |_| on_toggle_preview.call(()), "\u{1F441}" }
                button { class: if focus_mode { "toolbar-btn overflow-hidden active" } else { "toolbar-btn overflow-hidden" }, onclick: move |_| on_toggle_focus_mode.call(()), "\u{26F6}" }
                span { class: "separator" }
                div { class: "font-size-controls",
                    button { class: "toolbar-btn", onclick: move |_| on_decrease_font.call(()), title: "文字を小さく", "A-" }
                    span { class: "font-size-value", "{font_size}" }
                    button { class: "toolbar-btn", onclick: move |_| on_increase_font.call(()), title: "文字を大きく", "A+" }
                }
            }
            if *show_overflow.read() {
                div {
                    class: "menu-dropdown open",
                    onclick: move |_| show_overflow.set(false),
                    div { class: "menu-dropdown-panel", onclick: |e| e.stop_propagation(),
                        button { class: "menu-dropdown-item", onclick: move |_| { on_new_project.call(()); show_overflow.set(false); }, "新規" }
                        button { class: "menu-dropdown-item", onclick: move |_| { on_open_project.call(()); show_overflow.set(false); }, "開く" }
                        button { class: "menu-dropdown-item", onclick: move |_| { on_import_project.call(()); show_overflow.set(false); }, "取込" }
                        button { class: "menu-dropdown-item", disabled: proj_name.is_empty(), onclick: move |_| { on_close_project.call(()); show_overflow.set(false); }, "閉じる" }
                        button { class: "menu-dropdown-item", disabled: proj_name.is_empty(), onclick: move |_| { on_export.call(()); show_overflow.set(false); }, "出力" }
                        button { class: "menu-dropdown-item", onclick: move |_| { on_toggle_dark.call(()); show_overflow.set(false); }, if is_dark { "\u{2601} ライト" } else { "\u{2600} ダーク" } }
                        button { class: "menu-dropdown-item", onclick: move |_| { on_toggle_sidebar.call(()); show_overflow.set(false); }, "\u{2630} サイドバー" }
                        button { class: "menu-dropdown-item", onclick: move |_| { on_toggle_editor.call(()); show_overflow.set(false); }, "E エディタ" }
                        button { class: "menu-dropdown-item", onclick: move |_| { on_toggle_preview.call(()); show_overflow.set(false); }, "\u{1F441} プレビュー" }
                        button { class: "menu-dropdown-item", onclick: move |_| { on_toggle_focus_mode.call(()); show_overflow.set(false); }, "\u{26F6} 集中" }
                        button { class: "menu-dropdown-item", onclick: move |_| { on_decrease_font.call(()); show_overflow.set(false); }, "A- 縮小" }
                        button { class: "menu-dropdown-item", onclick: move |_| { on_increase_font.call(()); show_overflow.set(false); }, "A+ 拡大" }
                    }
                }
            }
            div { class: "toolbar-right",
                div { class: "stats",
                    div { class: "stat-item", title: "今日の執筆文字数",
                        span { class: "stat-value", "{today_char_count}" }
                        span { class: "stat-label", "字" }
                    }
                    div { class: "stat-item", title: "今日の目標進捗",
                        span { class: "stat-value", "{progress}" }
                        span { class: "stat-label", "%" }
                    }
                    div { class: "stat-item", title: "現在の話の読了時間（目安）",
                        span { class: "stat-value", "{reading_time}" }
                        span { class: "stat-label", "分" }
                    }
                }
                div { class: "progress-bar-container",
                    div { class: "progress-bar", style: "width: {progress}%" }
                }
            }
        }
    }
}


