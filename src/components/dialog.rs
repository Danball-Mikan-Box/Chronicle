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
