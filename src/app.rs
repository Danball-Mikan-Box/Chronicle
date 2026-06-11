use dioxus::prelude::*;

use crate::components::editor::Editor;
use crate::components::preview::Preview;
use crate::components::sidebar::Sidebar;
use crate::components::toolbar::Toolbar;
use crate::model::project::{Project, WritingMode};
use crate::styles;

#[component]
pub fn App() -> Element {
    let content = use_signal(|| String::new());
    let selected_chapter = use_signal(|| String::new());
    let project = use_signal(|| Option::<Project>::None);
    let writing_mode = use_signal(|| WritingMode::Horizontal);

    let writing_mode_str = use_memo(move || match *writing_mode.read() {
        WritingMode::Vertical => "vertical",
        WritingMode::Horizontal => "horizontal",
    });

    rsx! {
        style { "{styles::MAIN_CSS}" }
        div { class: "app",
            Toolbar {
                writing_mode: writing_mode,
                content: content,
            }
            div { class: "main-content",
                Sidebar {
                    project: project,
                    selected_chapter: selected_chapter,
                }
                div { class: "editor-pane",
                    Editor { content: content }
                }
                div { class: "preview-pane",
                    Preview {
                        content: content,
                        writing_mode: writing_mode_str,
                    }
                }
            }
        }
    }
}
