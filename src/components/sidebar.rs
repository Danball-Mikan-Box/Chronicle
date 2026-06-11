use dioxus::prelude::*;

use crate::model::project::Project;

#[component]
pub fn Sidebar(
    project: Signal<Option<Project>>,
    selected_chapter: Signal<String>,
    on_add_chapter: EventHandler<String>,
    on_delete_chapter: EventHandler<String>,
) -> Element {
    let proj = project.read().clone();

    rsx! {
        aside {
            class: "sidebar",
            div { class: "sidebar-header",
                h2 { "作品構成" }
            }
            match proj {
                Some(p) => rsx! {
                    div { class: "project-info",
                        h3 { "{p.name}" }
                        if !p.settings.author.is_empty() {
                            small { "{p.settings.author}" }
                        }
                    }
                    ul { class: "chapter-list",
                        {p.chapters.into_iter().map(|ch| {
                            let fname = ch.file_name.clone();
                            let fname2 = fname.clone();
                            let title = ch.title.clone();
                            let is_active = *selected_chapter.read() == fname;
                            rsx! {
                                li {
                                    class: if is_active { "chapter-item active" } else { "chapter-item" },
                                    div { class: "chapter-item-content",
                                        span {
                                            class: "chapter-title",
                                            onclick: move |_| selected_chapter.set(fname.clone()),
                                            "{title}"
                                        }
                                        button {
                                            class: "chapter-del-btn",
                                            title: "削除",
                                            onclick: move |_| on_delete_chapter.call(fname2.clone()),
                                            "×"
                                        }
                                    }
                                }
                            }
                        })}
                    }
                    div { class: "sidebar-actions",
                        button {
                            class: "sidebar-action-btn",
                            onclick: move |_| on_add_chapter.call("新規章".to_string()),
                            "+ 章を追加"
                        }
                    }
                },
                None => rsx! {
                    div { class: "sidebar-empty",
                        p { "プロジェクトがありません" }
                        p { small {"「ファイル > 新規プロジェクト」または「ファイル > 開く」から始めてください"} }
                    }
                }
            }
        }
    }
}
