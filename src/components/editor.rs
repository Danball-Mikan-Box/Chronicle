use dioxus::prelude::*;
use dioxus_desktop::DesktopContext;

use crate::components::formatting_bar::{FormatKind, FormattingBar};

fn build_format_js(kind: FormatKind) -> &'static str {
    match kind {
        FormatKind::Bold => r#"
            const ta = document.querySelector('.editor');
            if (!ta) return;
            const s = ta.selectionStart, e = ta.selectionEnd;
            const sel = ta.value.substring(s, e);
            const before = ta.value.substring(0, s);
            const after = ta.value.substring(e);
            if (sel) {
                ta.value = before + '**' + sel + '**' + after;
                ta.selectionStart = ta.selectionEnd = s + 2 + sel.length + 2;
            } else {
                ta.value = before + '**太文字**' + after;
                ta.selectionStart = ta.selectionEnd = s + 2;
            }
            ta.dispatchEvent(new Event('input', { bubbles: true }));
        "#,
        FormatKind::Italic => r#"
            const ta = document.querySelector('.editor');
            if (!ta) return;
            const s = ta.selectionStart, e = ta.selectionEnd;
            const sel = ta.value.substring(s, e);
            const before = ta.value.substring(0, s);
            const after = ta.value.substring(e);
            if (sel) {
                ta.value = before + '*' + sel + '*' + after;
                ta.selectionStart = ta.selectionEnd = s + 1 + sel.length + 1;
            } else {
                ta.value = before + '*斜体*' + after;
                ta.selectionStart = ta.selectionEnd = s + 1;
            }
            ta.dispatchEvent(new Event('input', { bubbles: true }));
        "#,
        FormatKind::Heading => r#"
            const ta = document.querySelector('.editor');
            if (!ta) return;
            const s = ta.selectionStart;
            const before = ta.value.substring(0, s);
            const after = ta.value.substring(s);
            ta.value = before + '\n# 見出し1\n' + after;
            ta.selectionStart = ta.selectionEnd = s + 3;
            ta.dispatchEvent(new Event('input', { bubbles: true }));
        "#,
        FormatKind::SubHeading => r#"
            const ta = document.querySelector('.editor');
            if (!ta) return;
            const s = ta.selectionStart;
            const before = ta.value.substring(0, s);
            const after = ta.value.substring(s);
            ta.value = before + '\n## 見出し2\n' + after;
            ta.selectionStart = ta.selectionEnd = s + 4;
            ta.dispatchEvent(new Event('input', { bubbles: true }));
        "#,
        FormatKind::SubSubHeading => r#"
            const ta = document.querySelector('.editor');
            if (!ta) return;
            const s = ta.selectionStart;
            const before = ta.value.substring(0, s);
            const after = ta.value.substring(s);
            ta.value = before + '\n### 見出し3\n' + after;
            ta.selectionStart = ta.selectionEnd = s + 5;
            ta.dispatchEvent(new Event('input', { bubbles: true }));
        "#,
        FormatKind::Quote => r#"
            const ta = document.querySelector('.editor');
            if (!ta) return;
            const s = ta.selectionStart;
            const before = ta.value.substring(0, s);
            const after = ta.value.substring(s);
            ta.value = before + '\n> 引用文\n' + after;
            ta.selectionStart = ta.selectionEnd = s + 3;
            ta.dispatchEvent(new Event('input', { bubbles: true }));
        "#,
        FormatKind::BulletList => r#"
            const ta = document.querySelector('.editor');
            if (!ta) return;
            const s = ta.selectionStart;
            const before = ta.value.substring(0, s);
            const after = ta.value.substring(s);
            ta.value = before + '\n- 箇条書き\n' + after;
            ta.selectionStart = ta.selectionEnd = s + 3;
            ta.dispatchEvent(new Event('input', { bubbles: true }));
        "#,
        FormatKind::NumberedList => r#"
            const ta = document.querySelector('.editor');
            if (!ta) return;
            const s = ta.selectionStart;
            const before = ta.value.substring(0, s);
            const after = ta.value.substring(s);
            ta.value = before + '\n1. 番号付き\n' + after;
            ta.selectionStart = ta.selectionEnd = s + 4;
            ta.dispatchEvent(new Event('input', { bubbles: true }));
        "#,
        FormatKind::Link => r#"
            const ta = document.querySelector('.editor');
            if (!ta) return;
            const s = ta.selectionStart, e = ta.selectionEnd;
            const sel = ta.value.substring(s, e);
            const before = ta.value.substring(0, s);
            const after = ta.value.substring(e);
            if (sel) {
                ta.value = before + '[' + sel + '](url)' + after;
                ta.selectionStart = ta.selectionEnd = s + 1;
            } else {
                ta.value = before + '[リンクテキスト](url)' + after;
                ta.selectionStart = ta.selectionEnd = s + 1;
            }
            ta.dispatchEvent(new Event('input', { bubbles: true }));
        "#,
        FormatKind::Ruby => r#"
            const ta = document.querySelector('.editor');
            if (!ta) return;
            const s = ta.selectionStart, e = ta.selectionEnd;
            const sel = ta.value.substring(s, e);
            const before = ta.value.substring(0, s);
            const after = ta.value.substring(e);
            if (sel) {
                ta.value = before + '{' + sel + '|ルビ}' + after;
                ta.selectionStart = ta.selectionEnd = s + 1 + sel.length + 2;
            } else {
                ta.value = before + '{漢字|かんじ}' + after;
                ta.selectionStart = ta.selectionEnd = s + 1;
            }
            ta.dispatchEvent(new Event('input', { bubbles: true }));
        "#,
        FormatKind::Separator => r#"
            const ta = document.querySelector('.editor');
            if (!ta) return;
            const s = ta.selectionStart;
            const before = ta.value.substring(0, s);
            const after = ta.value.substring(s);
            ta.value = before + '\n---\n' + after;
            ta.selectionStart = ta.selectionEnd = s + 5;
            ta.dispatchEvent(new Event('input', { bubbles: true }));
        "#,
    }
}

fn keyboard_shortcut_js(key: &str) -> &'static str {
    match key {
        "b" => build_format_js(FormatKind::Bold),
        "i" => build_format_js(FormatKind::Italic),
        _ => "",
    }
}

#[component]
pub fn Editor(content: Signal<String>, on_save: EventHandler<()>) -> Element {
    let desktop = dioxus_core::prelude::consume_context::<DesktopContext>();

    let desktop2 = desktop.clone();

    let on_format = move |kind: FormatKind| {
        let js = build_format_js(kind);
        let _ = desktop.webview.evaluate_script(js);
    };

    let on_keydown = move |evt: Event<KeyboardData>| {
        if evt.modifiers().contains(Modifiers::CONTROL) {
            match evt.key() {
                Key::Character(c) if c.as_str() == "s" => {
                    on_save.call(());
                }
                Key::Character(c) if c.as_str() == "b" => {
                    let js = keyboard_shortcut_js("b");
                    let _ = desktop2.webview.evaluate_script(js);
                }
                Key::Character(c) if c.as_str() == "i" => {
                    let js = keyboard_shortcut_js("i");
                    let _ = desktop2.webview.evaluate_script(js);
                }
                _ => {}
            }
        }
    };

    rsx! {
        div { class: "editor-wrapper",
            FormattingBar { on_format: on_format }
            textarea {
                class: "editor",
                value: "{content}",
                oninput: move |evt| content.set(evt.value()),
                onkeydown: on_keydown,
                placeholder: "ここに物語を書いてください...\nCtrl+S: 保存  Ctrl+B: 太字  Ctrl+I: 斜体",
            }
        }
    }
}
