use std::fs;

pub fn get_content(file_path: &str) -> Result<String, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read the file: {}", e))?;

    if content.is_empty() {
        return Err("Content is empty".to_string());
    }

    validate(&content)?;

    Ok(content)
}

pub fn validate(xml: &str) -> Result<(), String> {
    is_well_formed(xml)
}

fn is_well_formed(xml: &str) -> Result<(), String> {
    let mut stack = Vec::new();
    let mut root_count = 0;
    let mut chars = xml.chars().peekable();

    let mut line = 1;
    let mut col = 0;

    while let Some(c) = chars.next() {
        if c == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }

        if c == '<' {
            let start_line = line;
            let start_col = col;

            // Look ahead
            match chars.peek() {
                Some('?') => {
                    // Skip processing instruction <?...?>
                    let mut prev_question = false;
                    let mut closed = false;
                    while let Some(inner) = chars.next() {
                        if prev_question && inner == '>' {
                            closed = true;
                            break;
                        }
                        prev_question = inner == '?';
                    }
                    if !closed {
                        return Err(format!(
                            "Error at {}:{}: Unclosed processing instruction",
                            start_line, start_col
                        ));
                    }
                }
                Some('!') => {
                    chars.next(); // Skip '!'
                    let first = chars.next();
                    if first == Some('-') && chars.peek() == Some(&'-') {
                        chars.next(); // consume second '-', now inside <!-- -->
                        let mut closed = false;
                        while let Some(c) = chars.next() {
                            if c == '-' {
                                if let Some(&'-') = chars.peek() {
                                    chars.next();
                                    if let Some(&'>') = chars.peek() {
                                        chars.next();
                                        closed = true;
                                        break;
                                    }
                                }
                            }
                        }
                        if !closed {
                            return Err(format!(
                                "Error at {}:{}: Unclosed comment",
                                start_line, start_col
                            ));
                        }
                    } else {
                        // <!DOCTYPE> or other declaration — skip to closing >
                        while let Some(c) = chars.next() {
                            if c == '>' { break; }
                        }
                    }
                    continue;
                }
                Some('/') => {
                    // Process closing tag </tag>
                    chars.next(); // Skip '/'
                    let mut tag_name = String::new();
                    while let Some(&inner) = chars.peek() {
                        if inner == '>' {
                            break;
                        }
                        if let Some(c) = chars.next() {
                            tag_name.push(c);
                        }
                    }
                    chars.next(); // Skip '>'

                    let expected = tag_name.trim();
                    match stack.pop() {
                        Some(last) if last == expected => {}
                        Some(last) => {
                            return Err(format!(
                                "Error at {}:{}: Expected </{}>, but found </{}>",
                                start_line, start_col, last, expected
                            ));
                        }
                        None => {
                            return Err(format!(
                                "Error at {}:{}: Unexpected closing tag </{}>",
                                start_line, start_col, expected
                            ));
                        }
                    }
                }
                _ => {
                    // Opening or self-closing tag
                    let mut tag_content = String::new();
                    let mut is_self_closing = false;
                    let mut inside_quote: Option<char> = None;

                    while let Some(&inner) = chars.peek() {
                        if let Some(q) = inside_quote {
                            if inner == q {
                                inside_quote = None;
                            }
                        } else if inner == '"' || inner == '\'' {
                            inside_quote = Some(inner);
                        } else if inner == '>' {
                            break;
                        }
                        if let Some(current) = chars.next() {
                            tag_content.push(current);
                        }
                    }

                    if tag_content.ends_with('/') {
                        is_self_closing = true;
                        tag_content.pop(); // Remove '/' from tag name
                    }

                    chars.next(); // Skip '>'

                    let tag_name = tag_content
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string();

                    if !is_self_closing {
                        if stack.is_empty() && root_count > 0 {
                            return Err(format!(
                                "Error at {}:{}: Second root element <{}>",
                                start_line, start_col, tag_name
                            ));
                        }
                        if stack.is_empty() {
                            root_count += 1;
                        }
                        stack.push(tag_name.to_string());
                    } else {
                        // For self-closing tags, just check root structure
                        if stack.is_empty() && root_count > 0 {
                            return Err(format!(
                                "Error at {}:{}: Second root element <{}>",
                                start_line, start_col, tag_name
                            ));
                        }
                        if stack.is_empty() {
                            root_count += 1;
                        }
                    }
                }
            }
        }
    }

    if let Some(last) = stack.pop() {
        return Err(format!(
            "Error: Tag <{}> was not closed before end of file",
            last
        ));
    }

    if root_count == 0 {
        return Err("Error: Empty document".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_simple_xml() {
        assert!(validate("<root><child>text</child></root>").is_ok());
    }

    #[test]
    fn valid_with_declaration() {
        assert!(validate("<?xml version=\"1.0\"?><root/>").is_ok());
    }

    #[test]
    fn valid_self_closing_child() {
        assert!(validate("<root><br/></root>").is_ok());
    }

    #[test]
    fn valid_comment_inside() {
        assert!(validate("<root><!-- comment --><child/></root>").is_ok());
    }

    #[test]
    fn unclosed_comment_inside_root_fails() {
        assert!(validate("<root><!-- unclosed</root>").is_err());
    }

    #[test]
    fn unclosed_comment_after_root_fails() {
        assert!(validate("<root><child/></root><!-- unclosed").is_err());
    }

    #[test]
    fn valid_with_attributes() {
        assert!(validate("<root id=\"1\"><child name=\"x\"/></root>").is_ok());
    }

    #[test]
    fn empty_document_fails() {
        assert!(validate("").is_err());
    }

    #[test]
    fn unclosed_tag_fails() {
        assert!(validate("<root><child></root>").is_err());
    }

    #[test]
    fn mismatched_closing_tag_fails() {
        assert!(validate("<root><a></b></root>").is_err());
    }

    #[test]
    fn unexpected_closing_tag_fails() {
        assert!(validate("</root>").is_err());
    }

    #[test]
    fn multiple_root_elements_fail() {
        assert!(validate("<a/><b/>").is_err());
    }

    #[test]
    fn tag_not_closed_at_eof_fails() {
        assert!(validate("<root><child>text</child>").is_err());
    }

    #[test]
    fn valid_with_doctype() {
        assert!(validate("<!DOCTYPE root><root/>").is_ok());
    }

    #[test]
    fn valid_doctype_before_content() {
        assert!(validate("<!DOCTYPE root><root><child>text</child></root>").is_ok());
    }

    #[test]
    fn greater_than_in_double_quoted_attr_is_valid() {
        assert!(validate("<root attr=\"1>0\"/>").is_ok());
    }

    #[test]
    fn greater_than_in_single_quoted_attr_is_valid() {
        assert!(validate("<root attr='1>0'/>").is_ok());
    }

    #[test]
    fn greater_than_in_attr_with_children_is_valid() {
        assert!(validate("<root><item href=\"a>b\"/></root>").is_ok());
    }

    #[test]
    fn multiple_attrs_with_greater_than_is_valid() {
        assert!(validate("<root x=\"1>2\" y=\"3>4\"/>").is_ok());
    }

    #[test]
    fn processing_instruction_with_gt_inside_is_valid() {
        // '>' inside a PI must not terminate it; only '?>' does
        assert!(validate("<?pi foo=\"1>2\"?><root/>").is_ok());
    }

    #[test]
    fn unclosed_processing_instruction_fails() {
        assert!(validate("<?unclosed <root/>").is_err());
    }

    #[test]
    fn processing_instruction_at_eof_fails() {
        assert!(validate("<?xml version=\"1.0\"").is_err());
    }
}
