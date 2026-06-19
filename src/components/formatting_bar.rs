use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FormatKind {
    Bold,
    Italic,
    Heading,
    SubHeading,
    SubSubHeading,
    Quote,
    BulletList,
    NumberedList,
    Link,
    Ruby,
    Separator,
}

#[component]
pub fn FormattingBar(on_format: EventHandler<FormatKind>) -> Element {
    let mut show_help = use_signal(|| false);

    rsx! {
        div { class: "format-bar",
            div { class: "fmt-group-label", "文字" }
            button { class: "fmt-btn", title: "太字 (Ctrl+B)", onclick: move |_| on_format.call(FormatKind::Bold), "太字" }
            button { class: "fmt-btn", title: "斜体 (Ctrl+I)", onclick: move |_| on_format.call(FormatKind::Italic), "斜体" }
            button { class: "fmt-btn", title: "ルビ", onclick: move |_| on_format.call(FormatKind::Ruby), "ルビ" }

            span { class: "fmt-sep" }

            div { class: "fmt-group-label", "見出" }
            button { class: "fmt-btn", title: "大見出し", onclick: move |_| on_format.call(FormatKind::Heading), "H1" }
            button { class: "fmt-btn", title: "中見出し", onclick: move |_| on_format.call(FormatKind::SubHeading), "H2" }
            button { class: "fmt-btn", title: "小見出し", onclick: move |_| on_format.call(FormatKind::SubSubHeading), "H3" }

            span { class: "fmt-sep" }

            div { class: "fmt-group-label", "挿入" }
            button { class: "fmt-btn", title: "引用文", onclick: move |_| on_format.call(FormatKind::Quote), "\u{300C}\u{500D}" }
            button { class: "fmt-btn", title: "箇条書き", onclick: move |_| on_format.call(FormatKind::BulletList), "\u{2022} list" }
            button { class: "fmt-btn", title: "番号付きリスト", onclick: move |_| on_format.call(FormatKind::NumberedList), "1. list" }
            button { class: "fmt-btn", title: "リンク", onclick: move |_| on_format.call(FormatKind::Link), "Link" }
            button { class: "fmt-btn", title: "区切り線", onclick: move |_| on_format.call(FormatKind::Separator), "\u{2501}\u{2501}" }

            span { class: "fmt-sep" }

            button {
                class: if *show_help.read() { "fmt-btn active" } else { "fmt-btn" },
                title: "使い方",
                onclick: move |_| { let v = *show_help.read(); show_help.set(!v); },
                "?"
            }
        }
        if *show_help.read() {
            div { class: "fmt-help",
                p { "エディタで文字を入力し、範囲を選択（ドラッグ）してからボタンを押すとMarkdown書式が挿入されます" }
                p { "何も選択せずにボタンを押すと、その位置に見本テキストが挿入されます" }
                p { "ショートカット: Ctrl+B = 太字, Ctrl+I = 斜体, Ctrl+S = 保存, Ctrl+Z = 戻す, Ctrl+Y = やり直し" }
                p { "右側のプレビューでMarkdownの結果を確認できます" }
            }
        }
    }
}
