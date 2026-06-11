use super::parser;

pub fn render_to_html(markdown: &str) -> String {
    parser::parse_markdown(markdown)
}
