use dioxus::prelude::*;

use crate::model::{
    ActivityTab, DocRef, MaterialCategory, Project,
};

#[component]
pub fn Sidebar(
    project: Signal<Option<Project>>,
    active_tab: Signal<Option<DocRef>>,
    activity_tab: Signal<ActivityTab>,
    on_add_chapter: EventHandler<String>,
    on_delete_chapter: EventHandler<String>,
    on_rename_chapter: EventHandler<(String, String)>,
    on_add_tale: EventHandler<(String, String)>,
    on_delete_tale: EventHandler<(String, String)>,
    on_rename_tale: EventHandler<(String, String, String)>,
    on_add_material: EventHandler<(String, MaterialCategory)>,
    on_delete_material: EventHandler<String>,
    on_rename_material: EventHandler<(String, String)>,
    on_open_doc: EventHandler<DocRef>,
) -> Element {
    let proj = project.read().clone();
    let activity = *activity_tab.read();

    rsx! {
        div { class: "sidebar",
            // ── Title bar ──
            div { class: "sidebar-titlebar",
                div { class: "sidebar-titlebar-tabs",
                    button {
                        class: if activity == ActivityTab::Explorer { "titlebar-tab active" } else { "titlebar-tab" },
                        onclick: move |_| activity_tab.set(ActivityTab::Explorer),
                        "\u{1F4C4} 章・話"
                    }
                    button {
                        class: if activity == ActivityTab::Materials { "titlebar-tab active" } else { "titlebar-tab" },
                        onclick: move |_| activity_tab.set(ActivityTab::Materials),
                        "\u{1F4CB} 設定資料"
                    }
                }
            }
            // ── Body ──
            div { class: "sidebar-body",
                match (proj, activity) {
                    (Some(p), ActivityTab::Explorer) => {
                        let on_add_chapter = on_add_chapter.clone();
                        let on_add_tale = on_add_tale.clone();
                        rsx! {
                        div { class: "project-info",
                            h3 { "{p.name}" }
                        }
                        ul { class: "chapter-list",
                            {p.chapters.iter().map(|ch| {
                                let ch_dir = ch.dir_name.clone();
                                let ch_title = ch.title.clone();
                                let ch_dir_del = ch_dir.clone();
                                let ch_dir_ren = ch_dir.clone();
                                let ch_dir_tale = ch_dir.clone();
                                let is_ch_active = matches!(&*active_tab.read(), Some(DocRef::Tale { chapter_dir, .. }) if *chapter_dir == ch_dir);
                                let expanded = use_signal(|| true);

                                let on_click_ch = {
                                    let ch_dir = ch_dir.clone();
                                    let ch_title = ch_title.clone();
                                    let tales = p.chapters.iter().find(|c| c.dir_name == ch_dir.clone()).map(|c| &c.tales).cloned();
                                    let on_open_doc = on_open_doc.clone();
                                    let t_ch_dir = ch_dir.clone();
                                    let t_ch_title = ch_title.clone();
                                    move |_| {
                                        let ch_dir = t_ch_dir.clone();
                                        let ch_title = t_ch_title.clone();
                                        if let Some(ref tales) = tales {
                                            if let Some(first) = tales.first() {
                                                on_open_doc.call(DocRef::Tale {
                                                    chapter_dir: ch_dir.clone(),
                                                    tale_file: first.file_name.clone(),
                                                    chapter_title: ch_title.clone(),
                                                    tale_title: first.title.clone(),
                                                });
                                            }
                                        }
                                    }
                                };

                                rsx! {
                                    li { class: "chapter-group",
                                        div {
                                            class: if is_ch_active { "chapter-header active" } else { "chapter-header" },
                                            span {
                                                class: "chapter-toggle",
                                                onclick: move |_| { let mut e = expanded; let v = *e.read(); e.set(!v); },
                                                if expanded() { "\u{25BE}" } else { "\u{25B8}" }
                                            }
                                            span {
                                                class: "chapter-title",
                                                onclick: on_click_ch,
                                                ondoubleclick: move |_| on_rename_chapter.call((ch_dir_ren.clone(), ch_title.clone())),
                                                "{ch.title}"
                                            }
                                            button {
                                                class: "sidebar-del-btn",
                                                title: "章を削除",
                                                onclick: move |_| on_delete_chapter.call(ch_dir_del.clone()),
                                                "\u{00D7}"
                                            }
                                        }
                                        if *expanded.read() {
                                            ul { class: "tale-list",
                                                {ch.tales.iter().map(|tale| {
                                                    let t_ch_dir = ch_dir.clone();
                                                    let t_file = tale.file_name.clone();
                                                    let t_title = tale.title.clone();
                                                    let t_ch_ren = ch_dir.clone();
                                                    let t_old_file = tale.file_name.clone();
                                                    let t_ch_del = ch_dir.clone();
                                                    let t_file_del = tale.file_name.clone();
                                                    let t_active_ch_dir = ch_dir.clone();
                                                    let t_active_file = tale.file_name.clone();
                                                    let t_active_ch_title = ch_title.clone();
                                                    let t_active_t_title = t_title.clone();
                                                    let is_tale_active = *active_tab.read() == Some(DocRef::Tale {
                                                        chapter_dir: t_active_ch_dir,
                                                        tale_file: t_active_file,
                                                        chapter_title: t_active_ch_title,
                                                        tale_title: t_active_t_title,
                                                    });

                                                    let on_open = {
                                                        let on_open_doc = on_open_doc.clone();
                                                        let ch_dir = ch_dir.clone();
                                                        let ch_title = ch_title.clone();
                                                        let t_file = t_file.clone();
                                                        let t_title = t_title.clone();
                                                        move |_| {
                                                            on_open_doc.call(DocRef::Tale {
                                                                chapter_dir: ch_dir.clone(),
                                                                tale_file: t_file.clone(),
                                                                chapter_title: ch_title.clone(),
                                                                tale_title: t_title.clone(),
                                                            })
                                                        }
                                                    };

                                                    let on_ren = {
                                                        let on_rename_tale = on_rename_tale.clone();
                                                        let ch_dir = ch_dir.clone();
                                                        let old_file = tale.file_name.clone();
                                                        let title = tale.title.clone();
                                                        move |_| on_rename_tale.call((ch_dir.clone(), old_file.clone(), title.clone()))
                                                    };

                                                    rsx! {
                                                        li {
                                                            class: if is_tale_active { "tale-item active" } else { "tale-item" },
                                                            div { class: "tale-content",
                                                                span {
                                                                    class: "tale-title",
                                                                    onclick: on_open,
                                                                    ondoubleclick: on_ren,
                                                                    "{tale.title}"
                                                                }
                                                                button {
                                                                    class: "sidebar-del-btn",
                                                                    title: "削除",
                                                                    onclick: move |_| on_delete_tale.call((t_ch_del.clone(), t_file_del.clone())),
                                                                    "\u{00D7}"
                                                                }
                                                            }
                                                        }
                                                    }
                                                })}
                                            }
                                            div { class: "sidebar-sub-action",
                                                button {
                                                    onclick: move |_| on_add_tale.call((ch_dir_tale.clone(), "新規話".to_string())),
                                                    "+ 話を追加"
                                                }
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
                            button {
                                class: "sidebar-action-btn",
                                onclick: move |_| {
                                    let has_chapters = project.read().as_ref()
                                        .map(|p| !p.chapters.is_empty())
                                        .unwrap_or(false);
                                    if has_chapters {
                                        let ch_dir = project.read().as_ref()
                                            .and_then(|p| p.chapters.first().map(|c| c.dir_name.clone()))
                                            .unwrap_or_default();
                                        if !ch_dir.is_empty() {
                                            on_add_tale.call((ch_dir, "新規話".to_string()));
                                        }
                                    } else {
                                        on_add_chapter.call("本編".to_string());
                                        let ch_dir = project.read().as_ref()
                                            .and_then(|p| p.chapters.last().map(|c| c.dir_name.clone()))
                                            .unwrap_or_default();
                                        if !ch_dir.is_empty() {
                                            on_add_tale.call((ch_dir, "新規話".to_string()));
                                        }
                                    }
                                },
                                "+ 話を追加"
                            }
                        }
                    }
                    },
                    (Some(p), ActivityTab::Materials) => rsx! {
                        ul { class: "material-list",
                            {p.materials.iter().map(|mat| {
                                let m_file = mat.file_name.clone();
                                let m_title = mat.title.clone();
                                let m_file_del = m_file.clone();
                                let m_file_active = m_file.clone();
                                let m_title_active = m_title.clone();
                                let is_active = *active_tab.read() == Some(DocRef::Material {
                                    file_name: m_file_active,
                                    title: m_title_active,
                                });

                                let on_open = {
                                    let on_open_doc = on_open_doc.clone();
                                    let m_file = m_file.clone();
                                    let m_title = m_title.clone();
                                    move |_| {
                                        on_open_doc.call(DocRef::Material {
                                            file_name: m_file.clone(),
                                            title: m_title.clone(),
                                        })
                                    }
                                };

                                let on_ren = {
                                    let on_rename_material = on_rename_material.clone();
                                    let m_file = m_file.clone();
                                    let m_title = m_title.clone();
                                    move |_| on_rename_material.call((m_file.clone(), m_title.clone()))
                                };

                                rsx! {
                                    li {
                                        class: if is_active { "material-item active" } else { "material-item" },
                                        div { class: "material-content",
                                            span { class: "material-cat-badge", "{mat.category.label()}" }
                                            span {
                                                class: "material-title",
                                                onclick: on_open,
                                                ondoubleclick: on_ren,
                                                "{mat.title}"
                                            }
                                            button {
                                                class: "sidebar-del-btn",
                                                title: "削除",
                                                onclick: move |_| on_delete_material.call(m_file_del.clone()),
                                                "\u{00D7}"
                                            }
                                        }
                                    }
                                }
                            })}
                        }
                        div { class: "sidebar-actions",
                            button {
                                class: "sidebar-action-btn",
                                onclick: move |_| on_add_material.call(("新規資料".to_string(), MaterialCategory::Other("資料".to_string()))),
                                "+ 設定資料を追加"
                            }
                        }
                    },
                    (None, _) => rsx! {
                        div { class: "sidebar-empty",
                            p { "プロジェクトがありません" }
                            p { small {"「ファイル > 新規プロジェクト」または「ファイル > 開く」から始めてください"} }
                        }
                    },
                }
            }
        }
    }
}
