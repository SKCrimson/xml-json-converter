use std::fs;

// Состояние: чего мы ждем?
// Value, Key, Colon, CommaOrEnd
#[derive(PartialEq)]
enum Expect {
    Any,
    Key,
    Value,
    Colon,
    CommaOrClose,
}

pub fn get_content(file_path: &str) -> Result<String, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|_| "Failed to read the file. Please provide a valid JSON file.".to_string())?;

    if content.len() == 0 {
        return Err("Content is empty".to_string());
    }

    match is_well_formed(&content) {
        Ok(_) => (),
        Err(err) => return Err(err),
    };

    Ok(content)
}

struct PosTracker<I: Iterator<Item = char>> {
    inner: I,
    line: usize,
    col: usize,
}
impl<I: Iterator<Item = char>> Iterator for PosTracker<I> {
    type Item = (char, usize, usize); // Возвращаем символ вместе с его позицией

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

    // Теперь c — это (char, line, col)
    while let Some((c, l, c_pos)) = chars.next() {
        if c.is_whitespace() {
            continue;
        }

        match c {
            '{' => {
                stack.push('{');
                expect = Expect::Key;
                if let Some(&(nc, _, _)) = chars.peek() {
                    if nc == '}' {
                        expect = Expect::CommaOrClose;
                    }
                }
            }
            '[' => {
                stack.push('[');
                expect = Expect::Value;
                if let Some(&(nc, _, _)) = chars.peek() {
                    if nc == ']' {
                        expect = Expect::CommaOrClose;
                    }
                }
            }
            '}' | ']' => {
                // Проверка: можем ли мы закрыться сейчас?
                // Мы можем закрыться, если ждали CommaOrClose ИЛИ если это пустой контейнер (Value/Key)
                if expect != Expect::CommaOrClose
                    && expect != Expect::Value
                    && expect != Expect::Key
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
                expect = if stack.last() == Some(&'{') {
                    Expect::Key
                } else {
                    Expect::Value
                };
            }
            '"' => {
                // Передаем итератор в consume_string
                consume_string(&mut chars)
                    .map_err(|e| format!("{} near line {}, col {}", e, l, c_pos))?;

                if expect == Expect::Key {
                    expect = Expect::Colon;
                } else {
                    expect = Expect::CommaOrClose;
                }
            }
            _ => {
                if expect == Expect::Value {
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

fn consume_string<I>(chars: &mut std::iter::Peekable<I>) -> Result<String, String>
where
    I: Iterator<Item = (char, usize, usize)>,
{
    let mut s = String::new();
    let mut escaped = false;

    while let Some((c, _, _)) = chars.next() {
        if escaped {
            s.push(c);
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            return Ok(s);
        } else {
            s.push(c);
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
        lit.push(chars.next().unwrap().0);
    }
    Ok(lit)
}
