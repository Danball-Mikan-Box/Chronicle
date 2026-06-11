use dioxus::prelude::*;

#[component]
pub fn Editor(content: Signal<String>) -> Element {
    rsx! {
        textarea {
            class: "editor",
            value: "{content}",
            oninput: move |evt| content.set(evt.value()),
            placeholder: "Markdownで本文を書いてください...",
        }
    }
}
