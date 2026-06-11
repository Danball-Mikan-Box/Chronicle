use dioxus::prelude::*;

use crate::model::project::{Project, WritingMode};

#[component]
pub fn Toolbar(
    writing_mode: Signal<WritingMode>,
    content: Signal<String>,
    project: Signal<Option<Project>>,
    on_new_project: EventHandler<()>,
    on_open_project: EventHandler<()>,
    on_save: EventHandler<()>,
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

    rsx! {
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
                    onclick: move |_| on_save.call(()),
                    "保存"
                }
                span { class: "separator" }
                button {
                    class: if *writing_mode.read() == WritingMode::Vertical { "toolbar-btn active" } else { "toolbar-btn" },
                    onclick: move |_| writing_mode.set(WritingMode::Vertical),
                    "縦書"
                }
                button {
                    class: if *writing_mode.read() == WritingMode::Horizontal { "toolbar-btn active" } else { "toolbar-btn" },
                    onclick: move |_| writing_mode.set(WritingMode::Horizontal),
                    "横書"
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
