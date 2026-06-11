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

const FORMAT_BUTTONS: &[(FormatKind, &str, &str)] = &[
    (FormatKind::Bold, "B", "太字"),
    (FormatKind::Italic, "I", "斜体"),
    (FormatKind::Heading, "H1", "見出し1"),
    (FormatKind::SubHeading, "H2", "見出し2"),
    (FormatKind::SubSubHeading, "H3", "見出し3"),
    (FormatKind::Quote, "\u{201C}", "引用"),
    (FormatKind::BulletList, "\u{2022}", "箇条書き"),
    (FormatKind::NumberedList, "1.", "番号付きリスト"),
    (FormatKind::Link, "\u{1F517}", "リンク"),
    (FormatKind::Ruby, "あ", "ルビ"),
    (FormatKind::Separator, "\u{2501}", "区切り線"),
];

#[component]
pub fn FormattingBar(on_format: EventHandler<FormatKind>) -> Element {
    rsx! {
        div { class: "format-bar",
            {FORMAT_BUTTONS.iter().map(|&(kind, label, title)| {
                rsx! {
                    button {
                        class: "fmt-btn",
                        title: "{title}",
                        onclick: move |_| on_format.call(kind),
                        "{label}"
                    }
                }
            })}
        }
    }
}
