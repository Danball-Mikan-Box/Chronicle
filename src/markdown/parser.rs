pub fn parse_markdown(input: &str) -> String {
    let parser = pulldown_cmark::Parser::new_ext(
        input,
        pulldown_cmark::Options::ENABLE_TABLES
            | pulldown_cmark::Options::ENABLE_STRIKETHROUGH
            | pulldown_cmark::Options::ENABLE_TASKLISTS
            | pulldown_cmark::Options::ENABLE_FOOTNOTES,
    );
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    html
}
