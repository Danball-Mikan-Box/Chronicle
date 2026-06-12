use dioxus::document::eval;
use dioxus::prelude::*;
use dioxus_desktop::use_window;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

use crate::components::formatting_bar::{FormatKind, FormattingBar};

#[derive(Deserialize, Default)]
struct TextParts {
    before: String,
    sel: String,
    after: String,
}

#[derive(Serialize)]
struct FormatResponse {
    text: String,
    sel_start: usize,
    sel_end: usize,
}

fn utf16_len(s: &str) -> usize {
    s.chars().map(|c| c.len_utf16()).sum()
}

fn apply_format(sel: &str, kind: FormatKind) -> (String, usize, usize) {
    let empty = sel.is_empty();
    match kind {
        FormatKind::Bold => {
            if empty {
                ("**太文字**".to_string(), 2, 5)
            } else {
                let t = format!("**{}**", sel);
                let pos = 2 + utf16_len(sel) + 2;
                (t, pos, pos)
            }
        }
        FormatKind::Italic => {
            if empty {
                ("*斜体*".to_string(), 1, 3)
            } else {
                let t = format!("*{}*", sel);
                let pos = 1 + utf16_len(sel) + 1;
                (t, pos, pos)
            }
        }
        FormatKind::Heading => ("# 見出し".to_string(), 2, 2),
        FormatKind::SubHeading => ("## 見出し".to_string(), 3, 3),
        FormatKind::SubSubHeading => ("### 見出し".to_string(), 4, 4),
        FormatKind::Quote => ("> 引用文".to_string(), 2, 2),
        FormatKind::BulletList => ("- 箇条書き".to_string(), 2, 2),
        FormatKind::NumberedList => ("1. 番号付き".to_string(), 3, 3),
        FormatKind::Link => {
            if empty {
                ("[リンクテキスト](url)".to_string(), 1, 7)
            } else {
                (format!("[{}](url)", sel), 1 + utf16_len(sel) + 3, 1 + utf16_len(sel) + 6)
            }
        }
        FormatKind::Ruby => {
            if empty {
                ("{漢字|かんじ}".to_string(), 1, 3)
            } else {
                let t = format!("{{{}}}|ルビ", sel);
                let start = 1 + utf16_len(sel) + 1;
                (t, start, start + 2)
            }
        }
        FormatKind::Separator => ("---".to_string(), 3, 3),
    }
}

#[component]
pub fn Editor(
    content: Signal<String>,
    on_save: EventHandler<()>,
    chapter_version: u32,
    font_size: u32,
    placeholder: String,
) -> Element {
    let desktop = use_window();

    let desktop_sync = desktop.clone();
    use_effect(use_reactive(&chapter_version, move |_| {
        let val = content.read().clone();
        let js = format!(
            r#"const e=document.querySelector('.editor');if(e){{e.value={};e.selectionStart=e.selectionEnd=e.value.length;e.focus();}}"#,
            serde_json::to_string(&val).unwrap()
        );
        let _ = desktop_sync.webview.evaluate_script(&js);
    }));

    let do_format: Rc<dyn Fn(FormatKind)> = {
        let content = content.clone();
        Rc::new(move |kind: FormatKind| {
            let mut content = content;
            spawn(async move {
                let js = r#"
                    const ta = document.querySelector('.editor');
                    if (!ta) { dioxus.send({before:'',sel:'',after:''}); return; }
                    dioxus.send({
                        before: ta.value.substring(0, ta.selectionStart),
                        sel: ta.value.substring(ta.selectionStart, ta.selectionEnd),
                        after: ta.value.substring(ta.selectionEnd)
                    });
                    const r = await dioxus.recv();
                    ta.value = r.text;
                    ta.selectionStart = r.sel_start;
                    ta.selectionEnd = r.sel_end;
                    ta.focus();
                "#;

                let mut e = eval(js);
                let parts: TextParts = match e.recv().await {
                    Ok(p) => p,
                    Err(_) => TextParts::default(),
                };

                let (formatted, sel_start_utf16, sel_end_utf16) = apply_format(&parts.sel, kind);
                let is_inline = matches!(
                    kind,
                    FormatKind::Bold
                        | FormatKind::Italic
                        | FormatKind::Link
                        | FormatKind::Ruby
                );

                let (new_text, cursor_start, cursor_end) = if is_inline {
                    let new_text =
                        format!("{}{}{}", parts.before, formatted, parts.after);
                    let cursor_start = utf16_len(&parts.before) + sel_start_utf16;
                    let cursor_end = utf16_len(&parts.before) + sel_end_utf16;
                    (new_text, cursor_start, cursor_end)
                } else if matches!(kind, FormatKind::Separator) {
                    let combined_after = format!("{}{}", parts.sel, parts.after);
                    let new_text =
                        format!("{}\n{}\n{}", parts.before, formatted, combined_after);
                    let cursor =
                        utf16_len(&parts.before) + 1 + utf16_len(&formatted) + 1;
                    (new_text, cursor, cursor)
                } else {
                    let combined_after = format!("{}{}", parts.sel, parts.after);
                    let new_text =
                        format!("{}\n{}\n{}", parts.before, formatted, combined_after);
                    let cursor_start =
                        utf16_len(&parts.before) + 1 + sel_start_utf16;
                    let cursor_end =
                        utf16_len(&parts.before) + 1 + sel_end_utf16;
                    (new_text, cursor_start, cursor_end)
                };

                content.set(new_text.clone());
                let _ = e.send(FormatResponse {
                    text: new_text,
                    sel_start: cursor_start,
                    sel_end: cursor_end,
                });
            });
        })
    };

    let on_format = {
        let do_format = do_format.clone();
        move |kind: FormatKind| do_format(kind)
    };

    let on_keydown = {
        let do_format = do_format.clone();
        move |evt: Event<KeyboardData>| {
            if evt.modifiers().contains(Modifiers::CONTROL) {
                match evt.key() {
                    Key::Character(c) if c.as_str() == "s" => {
                        evt.prevent_default();
                        on_save.call(());
                    }
                    Key::Character(c) if c.as_str() == "b" => {
                        evt.prevent_default();
                        do_format(FormatKind::Bold);
                    }
                    Key::Character(c) if c.as_str() == "i" => {
                        evt.prevent_default();
                        do_format(FormatKind::Italic);
                    }
                    _ => {}
                }
            }
        }
    };

    rsx! {
        div { class: "editor-wrapper",
            FormattingBar { on_format: on_format }
            textarea {
                class: "editor",
                oninput: move |evt| content.set(evt.value()),
                onkeydown: on_keydown,
                placeholder: "{placeholder}",
                style: "font-size: {font_size}px;",
            }
        }
    }
}
