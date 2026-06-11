use dioxus::prelude::*;

use crate::markdown::renderer;

#[component]
pub fn Preview(content: Signal<String>, writing_mode: Memo<String>) -> Element {
    let html = renderer::render_to_html(&content.read());
    let wm = writing_mode.read().clone();

    rsx! {
        div {
            class: "preview",
            dangerous_inner_html: "{html}",
            style: if wm == "vertical" { "writing-mode: vertical-rl; height: 100%; padding: 40px 20px;" } else { "writing-mode: horizontal-tb;" },
        }
    }
}
