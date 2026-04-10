use crate::xml_to_json::xml_model::Token;

pub struct Lexer<'a> {
    input: &'a str,
    cursor: usize,
    in_tag: bool, // Состояние: находимся ли мы внутри угловых скобок
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input,
            cursor: 0,
            in_tag: false,
        }
    }

    /// Проходит по всей строке и собирает вектор токенов
    pub fn tokenize(&mut self) -> Result<Vec<Token<'a>>, &'static str> {
        let mut tokens = Vec::new();

        while let Some(token) = self.next_token() {
            // Опционально: логгирование для отладки
            println!("Token: {:?}", token);
            tokens.push(token);
        }

        // Базовая проверка: если мы закончили, но остались "внутри тега",
        // значит XML оборван (например, "<root ")
        if self.in_tag {
            return Err("Unexpected end of input: tag not closed");
        }

        Ok(tokens)
    }

    // Вспомогательный метод для получения оставшейся части строки
    fn remaining(&self) -> &'a str {
        if self.cursor >= self.input.len() {
            ""
        } else {
            &self.input[self.cursor..]
        }
    }

    // Продвигает курсор вперед
    fn advance(&mut self, n: usize) {
        self.cursor += n;
    }

    fn next_token(&mut self) -> Option<Token<'a>> {
        let rest = self.remaining().trim_start();
        if rest.is_empty() {
            return None;
        }

        let diff = self.remaining().len() - rest.len();
        self.advance(diff);
        let rest = self.remaining();

        if !self.in_tag {
            if rest.starts_with("</") {
                // ... (логика закрывающего тега без изменений)
                let end = rest.find('>')?;
                let name = &rest[2..end];
                self.advance(end + 1);
                Some(Token::TagClose(name))
            } else if rest.starts_with("<?") {
                // Специальная обработка декларации <?xml
                self.in_tag = true;
                let end = rest[2..]
                    .find(|c: char| c.is_whitespace() || c == '?')
                    .map(|i| i + 2)
                    .unwrap_or(rest.len());
                let name = &rest[1..end]; // оставим "?" в имени для отличия
                self.advance(end);
                Some(Token::TagOpen(name))
            } else if rest.starts_with("<!--") {
                // Специальная обработка комментариев <!--
                let end = rest.find("-->")?;
                self.advance(end + 3);
                Some(Token::EmptyTag)
            } else if rest.starts_with("<!") {
                // Специальная обработка декларации <!DOCTYPE
                let end = rest[2..]
                    .find(|c: char| c.is_whitespace() || c == '>')
                    .map(|i| i + 2)
                    .unwrap_or(rest.len());
                self.advance(end + 3);
                Some(Token::EmptyTag)
            } else if rest.starts_with('<') {
                self.in_tag = true;
                let end = rest[1..]
                    .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
                    .map(|i| i + 1)
                    .unwrap_or(rest.len());
                let name = &rest[1..end];
                self.advance(end);
                Some(Token::TagOpen(name))
            } else {
                // ... (логика текста без изменений)
                let end = rest.find('<').unwrap_or(rest.len());
                let text = &rest[..end];
                self.advance(end);
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    self.next_token()
                } else {
                    Some(Token::Text(trimmed))
                }
            }
        } else {
            // Внутри тега
            if rest.starts_with("?>") {
                // Закрытие декларации
                self.in_tag = false;
                self.advance(2);
                Some(Token::TagEnd)
            } else if rest.starts_with("/>") {
                self.in_tag = false;
                self.advance(2);
                Some(Token::TagSelfClose)
            } else if rest.starts_with('>') {
                self.in_tag = false;
                self.advance(1);
                Some(Token::TagEnd)
            } else {
                self.parse_attribute(rest)
            }
        }
    }

    fn parse_attribute(&mut self, rest: &'a str) -> Option<Token<'a>> {
        // Проверка: если мы наткнулись на конец тега перед парсингом атрибута
        if rest.starts_with('>') || rest.starts_with("/>") || rest.starts_with("?>") {
            return None;
        }

        let eq_pos = rest.find('=')?;
        let full_key = rest[..eq_pos].trim();

        // Важно: если ключ содержит '?', значит мы ошибочно парсим конец декларации
        // Но с новой логикой в next_token мы должны это проскакивать.

        let (ns, key) = if let Some(colon_pos) = full_key.find(':') {
            (Some(&full_key[..colon_pos]), &full_key[colon_pos + 1..])
        } else {
            (None, full_key)
        };

        let after_eq = rest[eq_pos + 1..].trim_start();
        let quote = after_eq.chars().next()?;
        let val_start = 1;
        let val_end = after_eq[val_start..].find(quote)? + val_start;
        let value = &after_eq[val_start..val_end];

        let total_consumed = rest.len() - after_eq[val_end + 1..].len();
        self.advance(total_consumed);

        Some(Token::Attr(ns, key, value))
    }
}
