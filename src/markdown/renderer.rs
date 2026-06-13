use super::parser;

pub fn render_to_html(markdown: &str) -> String {
    let preprocessed = preprocess(markdown);
    let html = parser::parse_markdown(&preprocessed);
    wrap_punctuation(&html)
}

fn wrap_punctuation(html: &str) -> String {
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
    let mut buf = String::new();
    for c in text.chars() {
        if c == '、' || c == '。' {
            if !buf.is_empty() {
                result.push_str(&buf);
                buf.clear();
            }
            result.push_str("<span class=\"punct\">");
            result.push(c);
            result.push_str("</span>");
        } else {
            buf.push(c);
        }
    }
    if !buf.is_empty() {
        result.push_str(&buf);
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
            while let Some(&(_, '\n')) = chars.peek() {
                chars.next();
            }
            result.push_str("\n\n");
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
    fn test_triple_newline_to_paragraph() {
        let md = "あいうえお\n\n\nかきくけこ";
        let out = preprocess(md);
        assert_eq!(out, "あいうえお\n\nかきくけこ");
    }

    #[test]
    fn test_quadruple_newline_to_paragraph() {
        let md = "あいうえお\n\n\n\nかきくけこ";
        let out = preprocess(md);
        assert_eq!(out, "あいうえお\n\nかきくけこ");
    }

    #[test]
    fn test_newline_in_code_block_preserved() {
        let md = "```\nline1\nline2\n\nline3\n```";
        assert_eq!(preprocess(md), md);
    }

    #[test]
    fn test_wrap_punct_simple() {
        assert_eq!(
            wrap_punctuation("<p>あいうえお。</p>"),
            "<p>あいうえお<span class=\"punct\">。</span></p>"
        );
    }

    #[test]
    fn test_wrap_punct_commma() {
        assert_eq!(
            wrap_punctuation("<p>あ、いうえお。</p>"),
            "<p>あ<span class=\"punct\">、</span>いうえお<span class=\"punct\">。</span></p>"
        );
    }

    #[test]
    fn test_wrap_punct_inside_tag() {
        assert_eq!(
            wrap_punctuation("<p>これは<div>テスト。</div>です。</p>"),
            "<p>これは<div>テスト<span class=\"punct\">。</span></div>です<span class=\"punct\">。</span></p>"
        );
    }

    #[test]
    fn test_wrap_punct_entity_skipped() {
        assert_eq!(
            wrap_punctuation("<p>あ&amp;いう。</p>"),
            "<p>あ&amp;いう<span class=\"punct\">。</span></p>"
        );
    }

    #[test]
    fn test_full_pipeline() {
        let md = "これはテストです。";
        let out = render_to_html(md);
        assert!(out.contains("<span class=\"punct\">。</span>"));
    }

    #[test]
    fn test_hrule_from_dashes() {
        let md = "text\n---\ntext";
        let out = render_to_html(md);
        assert!(out.contains("<hr"));
        assert!(!out.contains("---"));
    }

    #[test]
    fn test_no_hrule_from_blank_lines() {
        let md = "text\n\n\ntext";
        let out = render_to_html(md);
        assert!(!out.contains("<hr"));
    }

    #[test]
    fn test_alphanumeric_not_wrapped() {
        let md = "ABC123";
        let out = render_to_html(md);
        assert!(!out.contains("upright"));
        assert!(!out.contains("punct"));
    }
}
