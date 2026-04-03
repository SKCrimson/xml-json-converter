use std::fs;

pub fn get_content(file_path: &str) -> Result<String, &'static str> {
    let content = fs::read_to_string(file_path)
        .map_err(|_| "Failed to read the file. Please provide a valid XML file.")?;

    if content.len() == 0 {
        return Err("content is empty");
    } else if !is_well_formed(&content) {
        return Err("file is not well-formed");
    }

    Ok(content)
}

#[allow(dead_code)]
fn is_well_formed_old(content: &str) -> bool {
    let mut stack: Vec<&str> = Vec::new();
    let mut root_count = 0;
    let mut i = 0;

    let chars: Vec<char> = content.chars().collect();
    let lenght = chars.len();

    while i < chars.len() {
        if chars[i] == '<' {
            // 1. Обработка декларации <?xml ... ?> (просто пропускаем)
            if i + 1 < lenght && chars[i + 1] == '?' {
                while i < lenght && chars[i] != '>' {
                    i += 1;
                }
                i += 1;
                continue;
            }

            // 2. Определяем, закрывающий это тег </ или нет
            let is_closing = i + 1 < lenght && chars[i + 1] == '/';
            let start = if is_closing { i + 2 } else { i + 1 };

            // 3. Ищем конец тега
            let mut end = start;
            while end < lenght && chars[end] != '>' {
                end += 1;
            }
            if end >= lenght {
                return false;
            }

            // 4. Проверяем, не является ли тег самозакрывающимся <tag />
            let is_self_closing = !is_closing && chars[end - 1] == '/';

            // Извлекаем имя тега (до пробела или до /)
            let tag_content = if is_self_closing {
                &content[start..end - 1]
            } else {
                &content[start..end]
            };

            let tag_name = tag_content.split_whitespace().next().unwrap_or("").trim();

            if is_closing {
                match stack.pop() {
                    Some(last_tag) if last_tag == tag_name => {}
                    _ => return false,
                }
            } else if !is_self_closing {
                // Только если тег НЕ самозакрывающийся, кладем его в стек
                if stack.is_empty() && root_count > 0 {
                    return false;
                }
                if stack.is_empty() {
                    root_count += 1;
                }
                stack.push(tag_name);
            } else {
                // Самозакрывающийся тег: проверяем только корень
                if stack.is_empty() && root_count > 0 {
                    return false;
                }
                if stack.is_empty() {
                    root_count += 1;
                }
                // В стек не кладем!
            }
            i = end + 1;
        } else {
            i += 1;
        }
    }

    stack.is_empty() && root_count == 1
}

fn is_well_formed(xml: &str) -> bool {
    let mut stack = Vec::new();
    let mut root_count = 0;
    let mut chars = xml.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '<' {
            // Заглядываем вперед
            match chars.peek() {
                Some('?') => {
                    // Пропускаем декларацию <?...?>
                    while let Some(inner) = chars.next() {
                        if inner == '>' {
                            break;
                        }
                    }
                }
                Some('/') => {
                    // Обработка закрывающего тега </tag>
                    chars.next(); // Скипаем '/'
                    let mut tag_name = String::new();
                    while let Some(&inner) = chars.peek() {
                        if inner == '>' {
                            break;
                        }
                        tag_name.push(chars.next().unwrap());
                    }
                    chars.next(); // Скипаем '>'

                    if stack.pop() != Some(tag_name.trim().to_string()) {
                        return false;
                    }
                }
                _ => {
                    // Открывающий или самозакрывающийся тег
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
                        tag_content.pop(); // Убираем '/' из имени
                    }

                    chars.next(); // Скипаем '>'

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
                        stack.push(tag_name);
                    } else {
                        // Для самозакрывающегося просто проверяем структуру корня
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
