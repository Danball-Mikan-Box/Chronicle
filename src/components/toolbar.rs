use dioxus::prelude::*;

use crate::model::project::WritingMode;

#[component]
pub fn Toolbar(
    writing_mode: Signal<WritingMode>,
    content: Signal<String>,
) -> Element {
    let word_count = content.read().chars().count();

    rsx! {
        header {
            class: "toolbar",
            h1 { "Chronicle" }
            div { class: "toolbar-actions",
                button {
                    onclick: move |_| {
                        writing_mode.set(WritingMode::Vertical);
                    },
                    "縦書き"
                }
                button {
                    onclick: move |_| {
                        writing_mode.set(WritingMode::Horizontal);
                    },
                    "横書き"
                }
                span { class: "word-count",
                    "文字数: {word_count}"
                }
            }
        }
    }
}
