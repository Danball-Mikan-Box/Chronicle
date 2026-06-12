use dioxus::prelude::*;

use crate::model::DocRef;

#[component]
pub fn TabBar(
    open_tabs: Signal<Vec<DocRef>>,
    active_tab: Signal<Option<DocRef>>,
    on_close_tab: EventHandler<DocRef>,
    on_open_doc: EventHandler<DocRef>,
) -> Element {
    let tabs = open_tabs.read().clone();
    let active = active_tab.read().clone();

    rsx! {
        div { class: "tab-bar",
            {tabs.iter().map(|doc| {
                let is_active = Some(doc.clone()) == active;
                let label = match doc {
                    DocRef::Tale { chapter_title, tale_title, .. } => {
                        format!("{} / {}", chapter_title, tale_title)
                    }
                    DocRef::Material { title, .. } => title.clone(),
                };

                let doc_for_open = doc.clone();
                let doc_for_close = doc.clone();

                rsx! {
                    div {
                        class: if is_active { "tab active" } else { "tab" },
                        onclick: move |_| on_open_doc.call(doc_for_open.clone()),
                        span { class: "tab-label", "{label}" }
                        button {
                            class: "tab-close",
                            onclick: move |evt| {
                                evt.stop_propagation();
                                on_close_tab.call(doc_for_close.clone());
                            },
                            "\u{00D7}"
                        }
                    }
                }
            })}
        }
    }
}
