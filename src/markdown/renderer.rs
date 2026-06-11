use super::parser;

pub fn render_to_html(markdown: &str) -> String {
    let preprocessed = preprocess_ruby(markdown);
    parser::parse_markdown(&preprocessed)
}

fn preprocess_ruby(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
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
                        start = ruby_text.clone();
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
        assert_eq!(
            preprocess_ruby("{漢字|かんじ}"),
            "<ruby>漢字<rt>かんじ</rt></ruby>"
        );
    }

    #[test]
    fn test_ruby_in_markdown() {
        let md = "これは{日本語|にほんご}の文章です。";
        let expected = "これは<ruby>日本語<rt>にほんご</rt></ruby>の文章です。";
        assert_eq!(preprocess_ruby(md), expected);
    }

    #[test]
    fn test_ruby_no_pipe() {
        assert_eq!(preprocess_ruby("{ただの波カッコ}"), "{ただの波カッコ}");
    }

    #[test]
    fn test_code_block_skip() {
        let md = "```\n{漢字|かんじ}\n```";
        assert_eq!(preprocess_ruby(md), md);
    }
}
