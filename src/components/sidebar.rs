use dioxus::prelude::*;

use crate::model::project::Project;

#[component]
pub fn Sidebar(project: Signal<Option<Project>>, selected_chapter: Signal<String>) -> Element {
    let project_read = project.read();
    let project_clone = project_read.clone();

    rsx! {
        aside {
            class: "sidebar",
            h2 { "プロジェクト" }
            match project_clone {
                Some(proj) => rsx! {
                    h3 { "{proj.name}" }
                    ul {
                        {proj.chapters.into_iter().map(|chapter| {
                            let fname = chapter.file_name.clone();
                            let is_active = *selected_chapter.read() == fname;
                            rsx! {
                                li {
                                    class: if is_active { "active" } else { "" },
                                    onclick: move |_| selected_chapter.set(fname.clone()),
                                    "{chapter.title}"
                                }
                            }
                        })}
                    }
                },
                None => rsx! {
                    p { "プロジェクトがありません" }
                    p { "「ファイル > 新規プロジェクト」で作成してください" }
                }
            }
        }
    }
}
