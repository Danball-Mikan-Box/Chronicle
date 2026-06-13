use dioxus::prelude::*;

use crate::markdown::renderer;

#[component]
pub fn Preview(content: Signal<String>, writing_mode: Memo<String>) -> Element {
    let html = renderer::render_to_html(&content.read());
    let wm = writing_mode.read().clone();

    let vert_class = if wm == "vertical" { " vertical" } else { "" };

    rsx! {
        div {
            class: "preview{vert_class}",
            dangerous_inner_html: "{html}",
            "data-wm": "{wm}",
        }
    }
}
