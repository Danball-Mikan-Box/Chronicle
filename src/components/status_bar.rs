use dioxus::prelude::*;

use crate::model::project::Project;

#[component]
pub fn StatusBar(
    project: Signal<Option<Project>>,
    selected_chapter: Signal<String>,
    content: Signal<String>,
    is_saved: Signal<bool>,
    auto_save_enabled: Signal<bool>,
    writing_mode: Memo<String>,
    font_size: u32,
    on_increase_font: EventHandler<()>,
    on_decrease_font: EventHandler<()>,
) -> Element {
    let chap = selected_chapter.read().clone();
    let filename = if chap.is_empty() {
        "ファイル未選択".to_string()
    } else {
        chap.clone()
    };

    let char_count = content.read().chars().filter(|c| !c.is_whitespace()).count();
    let saved = *is_saved.read();
    let auto = *auto_save_enabled.read();

    rsx! {
        footer { class: "status-bar",
            div { class: "status-left",
                span { class: "status-item", "{filename}" }
                span { class: "status-sep", "|" }
                span {
                    class: if saved { "status-item saved" } else { "status-item unsaved" },
                    if saved { "保存済" } else { "未保存" }
                }
                span { class: "status-sep", "|" }
                span { class: "status-item", "{font_size}px" }
            }
            div { class: "status-right",
                span { class: "status-item", "{char_count} 文字" }
                span { class: "status-sep", "|" }
                span { class: "status-item",
                    if auto { "自動保存 ON" } else { "自動保存 OFF" }
                }
            }
        }
    }
}
