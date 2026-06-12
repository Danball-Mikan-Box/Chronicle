use dioxus::prelude::*;

use crate::model::project::Project;

#[component]
pub fn Sidebar(
    project: Signal<Option<Project>>,
    selected_chapter: Signal<String>,
    on_add_chapter: EventHandler<String>,
    on_delete_chapter: EventHandler<String>,
    on_rename_chapter: EventHandler<(String, String)>,
    recent_projects: Signal<Vec<String>>,
    on_open_recent: EventHandler<String>,
) -> Element {
    let proj = project.read().clone();
    let recent = recent_projects.read().clone();

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
                            let fname_del = fname.clone();
                            let fname_ren = fname.clone();
                            let title = ch.title.clone();
                            let title_ren = title.clone();
                            let is_active = *selected_chapter.read() == fname;
                            rsx! {
                                li {
                                    class: if is_active { "chapter-item active" } else { "chapter-item" },
                                    div { class: "chapter-item-content",
                                        span {
                                            class: "chapter-title",
                                            onclick: move |_| selected_chapter.set(fname.clone()),
                                            ondoubleclick: move |_| on_rename_chapter.call((fname_ren.clone(), title_ren.clone())),
                                            "{title}"
                                        }
                                        button {
                                            class: "chapter-del-btn",
                                            title: "削除",
                                            onclick: move |_| on_delete_chapter.call(fname_del.clone()),
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
            if !recent.is_empty() {
                div { class: "sidebar-recent",
                    h2 { "最近のプロジェクト" }
                    ul {
                        {recent.iter().rev().take(5).map(|path| {
                            let p = path.clone();
                            let name = std::path::Path::new(&p)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(&p)
                                .to_string();
                            rsx! {
                                li {
                                    class: "recent-item",
                                    onclick: move |_| on_open_recent.call(p.clone()),
                                    "{name}"
                                }
                            }
                        })}
                    }
                }
            }
        }
    }
}
