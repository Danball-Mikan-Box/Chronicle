use dioxus::prelude::*;
use std::collections::HashMap;

use crate::model::{DocRef, Project};

#[component]
pub fn StatusBar(
    project: Signal<Option<Project>>,
    active_tab: Signal<Option<DocRef>>,
    tab_content: Signal<HashMap<DocRef, String>>,
    is_saved: Signal<bool>,
    auto_save_enabled: Memo<bool>,
    writing_mode: Memo<String>,
    font_size: Memo<u32>,
    on_increase_font: EventHandler<()>,
    on_decrease_font: EventHandler<()>,
) -> Element {
    let filename = match active_tab.read().as_ref() {
        Some(doc) => doc.short_label(),
        None => "ファイル未選択".to_string(),
    };

    let text = active_tab.read().as_ref()
        .and_then(|doc| tab_content.read().get(doc).cloned())
        .unwrap_or_default();
    let char_count = text.chars().filter(|c| !c.is_whitespace()).count();
    let saved = *is_saved.read();
    let auto = *auto_save_enabled.read();
    let fs = *font_size.read();

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
                span { class: "status-item", "{fs}px" }
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
