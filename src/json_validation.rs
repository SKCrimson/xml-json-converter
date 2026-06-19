use crate::utils;

// State: what are we expecting?
#[derive(PartialEq)]
enum Expect {
    Any,
    Key,           // Just after '{' — empty object is OK
    Value,         // Just after '[' or ':' — empty array is OK
    KeyAfterComma, // After ',' in object — must have a key (trailing comma not OK)
    ValueAfterComma, // After ',' in array — must have a value (trailing comma not OK)
    Colon,
    CommaOrClose,
}

pub fn get_content(file_path: &str) -> Result<String, String> {
    let content = utils::read_file(file_path)?;
    validate(&content)?;
    Ok(content)
}

pub fn validate(json: &str) -> Result<(), String> {
    is_well_formed(json)
}

struct PosTracker<I: Iterator<Item = char>> {
    inner: I,
    line: usize,
    col: usize,
}
impl<I: Iterator<Item = char>> Iterator for PosTracker<I> {
    type Item = (char, usize, usize); // Return the character along with its position

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.inner.next()?;
        if c == '\n' {
            self.line += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }
        Some((c, self.line, self.col))
    }
}

fn is_well_formed(json: &str) -> Result<(), String> {
    let tracker = PosTracker {
        inner: json.chars(),
        line: 1,
        col: 0,
    };
    let mut chars = tracker.peekable();
    let mut stack: Vec<char> = Vec::new();
    let mut expect = Expect::Any;

    // Now c is (char, line, col)
    while let Some((c, l, c_pos)) = chars.next() {
        if c.is_whitespace() {
            continue;
        }

        match c {
            '{' => {
                stack.push('{');
                // Key = fresh open, empty object is allowed
                expect = Expect::Key;
            }
            '[' => {
                stack.push('[');
                // Value = fresh open, empty array is allowed
                expect = Expect::Value;
            }
            '}' | ']' => {
                // Allowed after fresh open (Key/Value) or after a value (CommaOrClose).
                // NOT allowed after comma (KeyAfterComma/ValueAfterComma = trailing comma).
                if expect != Expect::CommaOrClose
                    && expect != Expect::Key
                    && expect != Expect::Value
                {
                    return Err(format!(
                        "Unexpected closing '{}' at line {}, col {}",
                        c, l, c_pos
                    ));
                }

                let open = if c == '}' { '{' } else { '[' };
                if stack.pop() != Some(open) {
                    return Err(format!(
                        "Mismatched closing '{}' at line {}, col {}",
                        c, l, c_pos
                    ));
                }
                expect = Expect::CommaOrClose;
            }
            ':' => {
                if expect != Expect::Colon {
                    return Err(format!("Unexpected ':' at line {}, col {}", l, c_pos));
                }
                expect = Expect::Value;
            }
            ',' => {
                if expect != Expect::CommaOrClose {
                    return Err(format!("Unexpected ',' at line {}, col {}", l, c_pos));
                }
                // After comma: require a real key or value (closing bracket not allowed)
                expect = if stack.last() == Some(&'{') {
                    Expect::KeyAfterComma
                } else {
                    Expect::ValueAfterComma
                };
            }
            '"' => {
                let is_key = expect == Expect::Key || expect == Expect::KeyAfterComma;
                let is_value = expect == Expect::Value
                    || expect == Expect::ValueAfterComma
                    || expect == Expect::Any;
                if !is_key && !is_value {
                    return Err(format!(
                        "Unexpected string at line {}, col {}",
                        l, c_pos
                    ));
                }
                // Pass the iterator to consume_string
                consume_string(&mut chars)
                    .map_err(|e| format!("{} near line {}, col {}", e, l, c_pos))?;

                if is_key {
                    expect = Expect::Colon;
                } else {
                    expect = Expect::CommaOrClose;
                }
            }
            _ => {
                if expect == Expect::Value
                    || expect == Expect::ValueAfterComma
                    || expect == Expect::Any
                {
                    consume_literal(c, &mut chars)
                        .map_err(|e| format!("{} near line {}, col {}", e, l, c_pos))?;
                    expect = Expect::CommaOrClose;
                } else {
                    return Err(format!(
                        "Unexpected character '{}' at line {}, col {}",
                        c, l, c_pos
                    ));
                }
            }
        }
    }

    if !stack.is_empty() {
        return Err("Unexpected EOF: unclosed structures".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_object() {
        assert!(validate("{\"key\": \"value\"}").is_ok());
    }

    #[test]
    fn valid_array() {
        assert!(validate("[1, 2, 3]").is_ok());
    }

    #[test]
    fn valid_empty_object() {
        assert!(validate("{}").is_ok());
    }

    #[test]
    fn valid_empty_array() {
        assert!(validate("[]").is_ok());
    }

    #[test]
    fn valid_nested_structures() {
        assert!(validate("{\"a\": {\"b\": [1, true, null]}}").is_ok());
    }

    #[test]
    fn valid_all_scalar_types() {
        assert!(validate("{\"s\": \"str\", \"n\": 42, \"b\": true, \"f\": false, \"z\": null}").is_ok());
    }

    #[test]
    fn unclosed_brace_fails() {
        assert!(validate("{\"key\": \"value\"").is_err());
    }

    #[test]
    fn unclosed_bracket_fails() {
        assert!(validate("[1, 2").is_err());
    }

    #[test]
    fn missing_colon_fails() {
        assert!(validate("{\"key\" \"value\"}").is_err());
    }

    #[test]
    fn double_comma_fails() {
        assert!(validate("[1,, 2]").is_err());
    }

    #[test]
    fn mismatched_brackets_fail() {
        assert!(validate("[1, 2}").is_err());
    }

    #[test]
    fn trailing_comma_in_object_fails() {
        assert!(validate("{\"a\": 1,}").is_err());
    }

    #[test]
    fn invalid_keyword_literal_fails() {
        assert!(validate("{\"x\": treu}").is_err());
    }

    #[test]
    fn invalid_number_literal_fails() {
        assert!(validate("{\"x\": 1.2.3}").is_err());
    }
}

fn consume_string<I>(chars: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = (char, usize, usize)>,
{
    let mut escaped = false;

    while let Some((c, _, _)) = chars.next() {
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            return Ok(());
        }
    }
    Err("Unclosed string".into())
}

fn consume_literal<I>(first_c: char, chars: &mut std::iter::Peekable<I>) -> Result<String, String>
where
    I: Iterator<Item = (char, usize, usize)>,
{
    let mut lit = String::from(first_c);
    while let Some(&(c, _, _)) = chars.peek() {
        if c.is_whitespace() || c == ',' || c == '}' || c == ']' || c == ':' {
            break;
        }
        if let Some((c, _, _)) = chars.next() {
            lit.push(c);
        }
    }
    match lit.as_str() {
        "true" | "false" | "null" => Ok(lit),
        _ if lit.parse::<f64>().is_ok() => Ok(lit),
        _ => Err(format!("Invalid literal '{}'", lit)),
    }
}
