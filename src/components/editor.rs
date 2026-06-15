use dioxus::document::eval;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

use crate::components::formatting_bar::{FormatKind, FormattingBar};
use crate::model::project::Project;

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
                (format!("[{}](url)", sel), 1, 1 + utf16_len(sel))
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
    project: Signal<Option<Project>>,
    global_settings: Signal<crate::model::project::GlobalSettings>,
    is_saved: Signal<bool>,
    on_save: EventHandler<()>,
    focus_mode: Signal<bool>,
    placeholder: String,
) -> Element {
    let is_composing = use_signal(|| false);

    let settings = project.read().as_ref().map(|p| p.settings.clone()).unwrap_or_default();
    let gs = global_settings.read();

    let wrapper_style = format!(
        "--editor-font-family: '{}'; --editor-font-size: {}px; --editor-line-height: {}; --editor-max-width: {}px;",
        gs.font_family, gs.font_size, gs.line_height, gs.max_width,
    );

    let editor_class = if settings.indent_paragraphs {
        "editor indent-paragraphs"
    } else {
        "editor"
    };

    let do_format: Rc<dyn Fn(FormatKind)> = {
        let content = content.clone();
        let is_saved_fmt = is_saved.clone();
        let composing_fmt = is_composing.clone();
        Rc::new(move |kind: FormatKind| {
            if *composing_fmt.read() { return; }
            let mut content = content;
            let mut is_saved_fmt = is_saved_fmt.clone();
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
                is_saved_fmt.set(false);
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
        let content_kd = content.clone();
        move |evt: Event<KeyboardData>| {
            if evt.is_composing() { return; }
            match evt.key() {
                Key::Escape => {
                    let _ = eval("document.querySelector('.editor').blur();");
                    return;
                }
                Key::Character(c) if evt.modifiers().contains(Modifiers::CONTROL) => {
                    match c.as_str() {
                        "s" => { evt.prevent_default(); on_save.call(()); }
                        "b" => { evt.prevent_default(); do_format(FormatKind::Bold); }
                        "i" => { evt.prevent_default(); do_format(FormatKind::Italic); }
                        _ => {}
                    }
                }
                Key::Enter => {
                    let mut spawn_kd = content_kd.clone();
                    let mut is_saved_enter = is_saved.clone();
                    if spawn_kd.read().ends_with('\u{300d}') {
                        evt.prevent_default();
                        spawn(async move {
                            let js = r#"
                                const ta = document.querySelector('.editor');
                                const pos = ta.selectionStart;
                                ta.value = ta.value.substring(0, pos) + '\n\n' + ta.value.substring(pos);
                                ta.selectionStart = ta.selectionEnd = pos + 2;
                                dioxus.send(ta.value);
                            "#;
                            let mut e = eval(js);
                            if let Ok(new_val) = e.recv().await {
                                spawn_kd.set(new_val);
                                is_saved_enter.set(false);
                            }
                        });
                    }
                }
                _ => {}
            }
        }
    };

    let mut composing = is_composing.clone();
    let on_compositionstart = move |_: CompositionEvent| {
        composing.set(true);
    };

    let on_compositionend = move |_: CompositionEvent| {
        composing.set(false);
    };

    let on_input = {
        let mut is_saved_inp = is_saved.clone();
        move |evt: Event<FormData>| {
            content.set(evt.value());
            is_saved_inp.set(false);
        }
    };

    rsx! {
        div { class: "editor-wrapper", style: "{wrapper_style}",
            if !*focus_mode.read() {
                FormattingBar { on_format: on_format }
            }
            textarea {
                class: "{editor_class}",
                value: "{content}",
                oninput: on_input,
                onkeydown: on_keydown,
                oncompositionstart: on_compositionstart,
                oncompositionend: on_compositionend,
                placeholder: "{placeholder}",
            }
        }
    }
}
