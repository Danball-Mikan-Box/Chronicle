use dioxus::prelude::*;

use crate::markdown::renderer;
use crate::model::project::Project;

#[component]
pub fn Preview(
    content: Signal<String>,
    writing_mode: Memo<String>,
    project: Signal<Option<Project>>,
    global_settings: Signal<crate::model::project::GlobalSettings>,
) -> Element {
    let settings = project.read().as_ref().map(|p| p.settings.clone()).unwrap_or_default();
    let gs = global_settings.read();
    let html = renderer::render_to_html(&content.read());
    let wm = writing_mode.read().clone();

    let vert_class = if wm == "vertical" { " vertical" } else { "" };
    let indent_class = if settings.indent_paragraphs { " indent-paragraphs" } else { "" };

    let margin = if wm == "vertical" { "margin-left: auto; margin-right: 0;" } else { "margin-left: 0; margin-right: auto;" };
    let transform = if wm == "vertical" { "text-transform: full-width;" } else { "text-transform: full-width;" };
    let style = format!(
        "font-family: '{}'; font-size: {}px; line-height: {}; max-width: {}px; {} {}",
        gs.font_family,
        gs.font_size,
        gs.line_height,
        if wm == "vertical" { "none".to_string() } else { gs.max_width.to_string() },
        margin,
        transform
    );

    rsx! {
        div {
            class: "preview{vert_class}{indent_class}",
            style: "{style}",
            dangerous_inner_html: "{html}",
            "data-wm": "{wm}",
        }
    }
}
