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

fn is_well_formed(json: &str) -> Result<(), String> {
    let mut stack: Vec<char> = Vec::new();
    let mut chars = json.chars().enumerate().peekable();

    let mut expect = Expect::Any;

    while let Some((_idx, c)) = chars.next() {
        if c.is_whitespace() {
            continue;
        }

        match c {
            '{' => {
                stack.push('{');
                expect = Expect::Key; // В пустом объекте {} это изменится ниже
            }
            '[' => {
                stack.push('[');
                expect = Expect::Value;
            }
            '}' => {
                if stack.pop() != Some('{') {
                    return Err("Unexpected '}'".into());
                }
                expect = Expect::CommaOrClose;
            }
            ']' => {
                if stack.pop() != Some('[') {
                    return Err("Unexpected ']'".into());
                }
                expect = Expect::CommaOrClose;
            }
            ':' => {
                if expect != Expect::Colon {
                    return Err("Unexpected ':'".into());
                }
                expect = Expect::Value;
            }
            ',' => {
                if expect != Expect::CommaOrClose {
                    return Err("Unexpected ','".into());
                }
                // После запятой в объекте ждем ключ, в массиве - значение
                expect = if stack.last() == Some(&'{') {
                    Expect::Key
                } else {
                    Expect::Value
                };
            }
            '"' => {
                // Логика пропуска строки...
                consume_string(&mut chars)?;

                if expect == Expect::Key {
                    expect = Expect::Colon;
                } else if expect == Expect::Value {
                    expect = Expect::CommaOrClose;
                } else {
                    return Err("Unexpected string".into());
                }
            }
            _ => {
                // Обработка чисел, true, false, null
                if expect == Expect::Value {
                    consume_literal(c, &mut chars)?;
                    expect = Expect::CommaOrClose;
                }
            }
        }
    }

    if !stack.is_empty() {
        return Err("Unclosed braces".into());
    }
    Ok(())
}

fn consume_string<I>(chars: &mut std::iter::Peekable<I>) -> Result<String, String>
where
    I: Iterator<Item = (usize, char)>,
{
    let mut s = String::new();
    let mut escaped = false;

    while let Some((_, c)) = chars.next() {
        if escaped {
            s.push(c);
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            return Ok(s); // Строка успешно завершена
        } else {
            s.push(c);
        }
    }
    Err("Unexpected end of input in string".to_string())
}

fn consume_literal<I>(
    first_char: char,
    chars: &mut std::iter::Peekable<I>,
) -> Result<String, String>
where
    I: Iterator<Item = (usize, char)>,
{
    let mut literal = String::from(first_char);

    // Читаем, пока не встретим разделитель JSON
    while let Some(&(_, c)) = chars.peek() {
        if c.is_whitespace() || c == ',' || c == '}' || c == ']' || c == ':' {
            break;
        }
        literal.push(chars.next().unwrap().1);
    }

    // Простая валидация типов (опционально)
    match literal.as_str() {
        "true" | "false" | "null" => Ok(literal),
        _ if literal.parse::<f64>().is_ok() => Ok(literal),
        _ => Err(format!("Invalid literal: {}", literal)),
    }
}
