use std::fs;

pub fn get_content(file_path: &str) -> Result<String, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|_| "Failed to read the file. Please provide a valid XML file.".to_string())?;

    if content.len() == 0 {
        return Err("content is empty".to_string());
    }

    match is_well_formed(&content) {
        Ok(_) => (),
        Err(err) => return Err(err),
    };

    Ok(content)
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
                    // Skip declaration <?...?>
                    while let Some(inner) = chars.next() {
                        if inner == '>' {
                            break;
                        }
                    }
                }
                Some('!') => {
                    // Check that this is really the beginning of a comment
                    while let Some(c) = chars.next() {
                        if c == '-' {
                            // Look ahead to check if there's another '-' and '>'
                            if let Some(&'-') = chars.peek() {
                                // If we found the second '-', temporarily extract it
                                chars.next();
                                if let Some(&'>') = chars.peek() {
                                    // Found '>', comment is finished
                                    chars.next();
                                    break;
                                }
                                // If there's no '>' after '--', continue the loop
                            }
                        }
                    }
                    continue; // Return to main parser loop
                }
                Some('/') => {
                    // Process closing tag </tag>
                    chars.next(); // Skip '/'
                    let mut tag_name = String::new();
                    while let Some(&inner) = chars.peek() {
                        if inner == '>' {
                            break;
                        }
                        tag_name.push(chars.next().unwrap());
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

                    while let Some(&inner) = chars.peek() {
                        if inner == '>' {
                            break;
                        }
                        let current = chars.next().unwrap();
                        tag_content.push(current);
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
                                "Error at {}:{}: Second root element </{}>",
                                start_line, start_col, tag_name
                            ));
                        }
                        if stack.is_empty() {
                            root_count += 1;
                        }
                        stack.push(tag_name.to_string());
                        // println!(
                        //     "DEBUG: Found tag '{}', self-closing: {}, stack: {:?}",
                        //     tag_name, is_self_closing, stack
                        // );
                    } else {
                        // For self-closing tags, just check root structure
                        if stack.is_empty() && root_count > 0 {
                            return Err(format!(
                                "Error at {}:{}: Second root element </{}>",
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
