use std::fs;

pub fn get_content(file_path: &str) -> Result<String, &'static str> {
    let content = fs::read_to_string(file_path)
        .map_err(|_| "Failed to read the file. Please provide a valid XML file.")?;

    if content.len() == 0 {
        return Err("content is empty");
    } else if !is_well_formed(&content) {
        return Err("XML validation failed. Please provide a well-formed XML file.");
    }

    Ok(content)
}

fn is_well_formed(xml: &str) -> bool {
    let mut stack = Vec::new();
    let mut root_count = 0;
    let mut chars = xml.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '<' {
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
                    // 1. Check that this is really the beginning of a comment
                    while let Some(c) = chars.next() {
                        if c == '-' {
                            // Look ahead: check if there's another '-' and '>'
                            if let Some(&'-') = chars.peek() {
                                // If we found the second '-', temporarily extract it
                                chars.next();
                                if let Some(&'>') = chars.peek() {
                                    // Found '>', comment is finished
                                    chars.next();
                                    break;
                                }
                                // If there's no '>' after "--", continue the loop
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

                    if stack.pop() != Some(tag_name.trim().to_string()) {
                        return false;
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
                        tag_content.pop(); // Remove '/' from name
                    }

                    chars.next(); // Skip '>'

                    let tag_name = tag_content
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string();

                    if !is_self_closing {
                        if stack.is_empty() && root_count > 0 {
                            return false;
                        }
                        if stack.is_empty() {
                            root_count += 1;
                        }
                        stack.push(tag_name.to_string());
                        // println!(
                        //     "DEBUG: Found tag '{}', self-closing: {}, stack before: {:?}",
                        //     tag_name, is_self_closing, stack
                        // );
                    } else {
                        // For self-closing just check root structure
                        if stack.is_empty() && root_count > 0 {
                            return false;
                        }
                        if stack.is_empty() {
                            root_count += 1;
                        }
                    }
                }
            }
        }
    }
    stack.is_empty() && root_count == 1
}
