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

    let close = move |_| visible.set(false);
    let on_keydown = move |e: Event<KeyboardData>| {
        if e.key() == Key::Escape { visible.set(false); }
    };

    rsx! {
        div { class: "dialog-overlay", onclick: close, onkeydown: on_keydown,
            div { class: "dialog", onclick: |e| e.stop_propagation(),
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
                        onclick: close,
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

    let dialog_title = if old_name.contains('|') {
        "話名の変更"
    } else if old_name.contains(".md") {
        "資料名の変更"
    } else {
        "章名の変更"
    };

    let close = move |_| visible.set(false);
    let on_keydown = move |e: Event<KeyboardData>| {
        if e.key() == Key::Escape { visible.set(false); }
    };

    let label_text = match dialog_title {
        "話名の変更" => "新しい話名",
        "資料名の変更" => "新しい資料名",
        _ => "新しい章名",
    };
    let placeholder = format!("{}を入力", label_text);

    rsx! {
        div { class: "dialog-overlay", onclick: close, onkeydown: on_keydown,
            div { class: "dialog", onclick: |e| e.stop_propagation(),
                h2 { "{dialog_title}" }
                div { class: "dialog-body",
                    label { "{label_text}" }
                    input {
                        class: "dialog-input",
                        value: "{new_title}",
                        oninput: move |e| new_title.set(e.value()),
                        placeholder: "{placeholder}",
                    }
                }
                div { class: "dialog-actions",
                    button {
                        class: "dialog-btn",
                        onclick: close,
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
