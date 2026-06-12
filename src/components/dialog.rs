use dioxus::prelude::*;

#[component]
pub fn ProjectDialog(
    visible: Signal<bool>,
    title: String,
    on_confirm: EventHandler<(String, String)>,
) -> Element {
    let mut name = use_signal(|| String::new());
    let mut author = use_signal(|| String::new());

    if !*visible.read() {
        return Ok(VNode::placeholder());
    }

    rsx! {
        div { class: "dialog-overlay",
            div { class: "dialog",
                h2 { "{title}" }
                div { class: "dialog-body",
                    label { "作品名" }
                    input {
                        class: "dialog-input",
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                        placeholder: "作品名を入力",
                    }
                    label { "作者名" }
                    input {
                        class: "dialog-input",
                        value: "{author}",
                        oninput: move |e| author.set(e.value()),
                        placeholder: "作者名（任意）",
                    }
                }
                div { class: "dialog-actions",
                    button {
                        class: "dialog-btn",
                        onclick: move |_| visible.set(false),
                        "キャンセル"
                    }
                    button {
                        class: "dialog-btn primary",
                        disabled: name.read().is_empty(),
                        onclick: move |_| {
                            let n = name.read().clone();
                            let a = author.read().clone();
                            if !n.is_empty() {
                                on_confirm.call((n, a));
                                name.set(String::new());
                                author.set(String::new());
                                visible.set(false);
                            }
                        },
                        "作成"
                    }
                }
            }
        }
    }
}

#[component]
pub fn RenameDialog(
    visible: Signal<bool>,
    file_name: Signal<(String, String)>,
    on_confirm: EventHandler<(String, String)>,
) -> Element {
    if !*visible.read() {
        return Ok(VNode::placeholder());
    }

    let target = file_name.read().clone();
    let old_name = target.0.clone();
    let current_title = target.1.clone();
    let mut new_title = use_signal(|| current_title.clone());

    rsx! {
        div { class: "dialog-overlay",
            div { class: "dialog",
                h2 { "章名の変更" }
                div { class: "dialog-body",
                    label { "新しい章名" }
                    input {
                        class: "dialog-input",
                        value: "{new_title}",
                        oninput: move |e| new_title.set(e.value()),
                        placeholder: "章名を入力",
                    }
                }
                div { class: "dialog-actions",
                    button {
                        class: "dialog-btn",
                        onclick: move |_| visible.set(false),
                        "キャンセル"
                    }
                    button {
                        class: "dialog-btn primary",
                        disabled: new_title.read().is_empty(),
                        onclick: move |_| {
                            let t = new_title.read().clone();
                            if !t.is_empty() {
                                on_confirm.call((old_name.clone(), t));
                                visible.set(false);
                            }
                        },
                        "変更"
                    }
                }
            }
        }
    }
}
