use super::parser;

pub fn render_to_html(markdown: &str) -> String {
    let preprocessed = preprocess(markdown);
    let html = parser::parse_markdown(&preprocessed);
    wrap_alphanumeric(&html)
}

fn wrap_alphanumeric(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut text_buf = String::new();
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => {
                push_processed_text(&mut result, &text_buf);
                text_buf.clear();
                in_tag = true;
                result.push(c);
            }
            '>' if in_tag => {
                in_tag = false;
                result.push(c);
            }
            _ if !in_tag => {
                text_buf.push(c);
            }
            _ => {
                result.push(c);
            }
        }
    }
    push_processed_text(&mut result, &text_buf);
    result
}

fn push_processed_text(result: &mut String, text: &str) {
    let mut alnum = String::new();
    let mut chars = text.char_indices().peekable();
    while let Some((_, c)) = chars.next() {
        if c == '&' {
            flush_alnum(result, &mut alnum);
            let mut entity = String::from('&');
            while let Some(&(_, next)) = chars.peek() {
                entity.push(next);
                chars.next();
                if next == ';' {
                    break;
                }
            }
            result.push_str(&entity);
        } else if c.is_ascii_alphanumeric() {
            alnum.push(c);
        } else {
            flush_alnum(result, &mut alnum);
            result.push(c);
        }
    }
    flush_alnum(result, &mut alnum);
}

fn flush_alnum(result: &mut String, alnum: &mut String) {
    if !alnum.is_empty() {
        result.push_str("<span class=\"upright\">");
        result.push_str(alnum);
        result.push_str("</span>");
        alnum.clear();
    }
}

fn preprocess(input: &str) -> String {
    let mut result = String::with_capacity(input.len() + input.len() / 2);
    let mut chars = input.char_indices().peekable();
    let mut in_code_block = false;

    while let Some((_, c)) = chars.next() {
        if c == '`' {
            let mut count = 1;
            while let Some(&(_, '`')) = chars.peek() {
                count += 1;
                chars.next();
            }
            if count >= 3 {
                in_code_block = !in_code_block;
            }
            result.push_str(&"`".repeat(count));
            continue;
        }

        if in_code_block {
            result.push(c);
            continue;
        }

        if c == '\n' {
            let mut nl_count = 1;
            while let Some(&(_, '\n')) = chars.peek() {
                nl_count += 1;
                chars.next();
            }
            if nl_count == 1 {
                result.push_str("\n\n");
            } else if nl_count >= 3 {
                result.push_str("\n\n---\n\n");
            } else {
                result.push_str("\n\n");
            }
            continue;
        }

        if c == '{' {
            let mut ruby_text = String::new();
            let mut has_pipe = false;
            let mut brace_depth = 1;
            let mut start = String::new();

            while let Some((_, next_c)) = chars.next() {
                match next_c {
                    '{' => brace_depth += 1,
                    '}' => {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            break;
                        }
                    }
                    '|' if brace_depth == 1 => {
                        has_pipe = true;
                        start.clone_from(&ruby_text);
                        ruby_text.clear();
                        continue;
                    }
                    _ => {}
                }
                ruby_text.push(next_c);
            }

            if has_pipe && brace_depth == 0 {
                result.push_str(&format!("<ruby>{0}<rt>{1}</rt></ruby>", start, ruby_text));
            } else {
                result.push('{');
                result.push_str(&start);
                if has_pipe {
                    result.push('|');
                }
                result.push_str(&ruby_text);
                if brace_depth == 0 {
                    result.push('}');
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ruby_simple() {
        let out = preprocess("{漢字|かんじ}");
        assert_eq!(out, "<ruby>漢字<rt>かんじ</rt></ruby>");
    }

    #[test]
    fn test_ruby_in_markdown() {
        let md = "これは{日本語|にほんご}の文章です。";
        let expected = "これは<ruby>日本語<rt>にほんご</rt></ruby>の文章です。";
        assert_eq!(preprocess(md), expected);
    }

    #[test]
    fn test_ruby_no_pipe() {
        assert_eq!(preprocess("{ただの波カッコ}"), "{ただの波カッコ}");
    }

    #[test]
    fn test_code_block_skip() {
        let md = "```\n{漢字|かんじ}\n```";
        assert_eq!(preprocess(md), md);
    }

    #[test]
    fn test_single_newline_to_paragraph() {
        let md = "あいうえお\nかきくけこ";
        let out = preprocess(md);
        assert_eq!(out, "あいうえお\n\nかきくけこ");
    }

    #[test]
    fn test_double_newline_stays() {
        let md = "あいうえお\n\nかきくけこ";
        assert_eq!(preprocess(md), md);
    }

    #[test]
    fn test_triple_newline_to_scene_break() {
        let md = "あいうえお\n\n\nかきくけこ";
        let out = preprocess(md);
        assert_eq!(out, "あいうえお\n\n---\n\nかきくけこ");
    }

    #[test]
    fn test_quadruple_newline_to_scene_break() {
        let md = "あいうえお\n\n\n\nかきくけこ";
        let out = preprocess(md);
        assert_eq!(out, "あいうえお\n\n---\n\nかきくけこ");
    }

    #[test]
    fn test_newline_in_code_block_preserved() {
        let md = "```\nline1\nline2\n\nline3\n```";
        assert_eq!(preprocess(md), md);
    }

    #[test]
    fn test_wrap_alphanumeric_simple() {
        assert_eq!(wrap_alphanumeric("<p>ABC</p>"), "<p><span class=\"upright\">ABC</span></p>");
    }

    #[test]
    fn test_wrap_alphanumeric_mixed() {
        assert_eq!(
            wrap_alphanumeric("<p>これはABCです</p>"),
            "<p>これは<span class=\"upright\">ABC</span>です</p>"
        );
    }

    #[test]
    fn test_wrap_alphanumeric_in_tag() {
        assert_eq!(
            wrap_alphanumeric("<p>これは<div class=\"test\">ABC</div>です</p>"),
            "<p>これは<div class=\"test\"><span class=\"upright\">ABC</span></div>です</p>"
        );
    }

    #[test]
    fn test_wrap_alphanumeric_numbers() {
        assert_eq!(
            wrap_alphanumeric("<p>42です</p>"),
            "<p><span class=\"upright\">42</span>です</p>"
        );
    }

    #[test]
    fn test_wrap_alphanumeric_single() {
        assert_eq!(
            wrap_alphanumeric("<p>Aです</p>"),
            "<p><span class=\"upright\">A</span>です</p>"
        );
    }

    #[test]
    fn test_full_pipeline() {
        let md = "これはABCの42章です。";
        let out = render_to_html(md);
        assert!(out.contains("<span class=\"upright\">ABC</span>"));
        assert!(out.contains("<span class=\"upright\">42</span>"));
    }

    #[test]
    fn test_wrap_entity_skipped() {
        assert_eq!(
            wrap_alphanumeric("<p>A&amp;B</p>"),
            "<p><span class=\"upright\">A</span>&amp;<span class=\"upright\">B</span></p>"
        );
    }
}
