use crate::xml_to_json::xml_model::Token;

fn decode_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }

    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '&' {
            out.push(c);
            continue;
        }

        let mut entity = String::new();
        let mut closed = false;
        for ec in chars.by_ref() {
            if ec == ';' {
                closed = true;
                break;
            }
            entity.push(ec);
        }

        if !closed {
            out.push('&');
            out.push_str(&entity);
            continue;
        }

        match entity.as_str() {
            "amp"  => out.push('&'),
            "lt"   => out.push('<'),
            "gt"   => out.push('>'),
            "quot" => out.push('"'),
            "apos" => out.push('\''),
            _ if entity.starts_with('#') => {
                let (radix, digits) = if entity[1..].starts_with(['x', 'X']) {
                    (16u32, &entity[2..])
                } else {
                    (10u32, &entity[1..])
                };
                match u32::from_str_radix(digits, radix).ok().and_then(char::from_u32) {
                    Some(ch) => out.push(ch),
                    None => { out.push('&'); out.push_str(&entity); out.push(';'); }
                }
            }
            _ => { out.push('&'); out.push_str(&entity); out.push(';'); }
        }
    }

    out
}

pub struct Lexer<'a> {
    input: &'a str,
    cursor: usize,
    in_tag: bool,
    error: Option<String>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input,
            cursor: 0,
            in_tag: false,
            error: None,
        }
    }

    /// Walks through the entire string and collects a vector of tokens
    pub fn tokenize(&mut self) -> Result<Vec<Token<'a>>, String> {
        let mut tokens = Vec::new();

        while let Some(token) = self.next_token() {
            tokens.push(token);
        }

        if let Some(err) = self.error.take() {
            return Err(err);
        }

        if self.in_tag {
            return Err("Unexpected end of input: tag not closed".to_string());
        }

        Ok(tokens)
    }

    // Helper method to get the remaining part of the string
    fn remaining(&self) -> &'a str {
        if self.cursor >= self.input.len() {
            ""
        } else {
            &self.input[self.cursor..]
        }
    }

    // Moves the cursor forward
    fn advance(&mut self, n: usize) {
        self.cursor += n;
    }

    fn next_token(&mut self) -> Option<Token<'a>> {
        loop {
            let rest = self.remaining().trim_start();
            if rest.is_empty() {
                return None;
            }

            let diff = self.remaining().len() - rest.len();
            self.advance(diff);
            let rest = self.remaining();

            if !self.in_tag {
                if rest.starts_with("</") {
                    let end = match rest.find('>') {
                        Some(i) => i,
                        None => {
                            self.error = Some("Unexpected EOF in closing tag".to_string());
                            return None;
                        }
                    };
                    let name = &rest[2..end];
                    self.advance(end + 1);
                    return Some(Token::TagClose(name));
                } else if rest.starts_with("<?") {
                    match rest.find("?>") {
                        Some(end) => {
                            self.advance(end + 2);
                            return Some(Token::Declaration);
                        }
                        None => {
                            self.error = Some("Unclosed processing instruction".to_string());
                            return None;
                        }
                    }
                } else if rest.starts_with("<!--") {
                    match rest.find("-->") {
                        Some(end) => {
                            self.advance(end + 3);
                            continue;
                        }
                        None => {
                            self.error = Some("Unclosed comment".to_string());
                            return None;
                        }
                    }
                } else if rest.starts_with("<!") {
                    // Skip <!DOCTYPE> and similar declarations to the closing >
                    let end = rest.find('>').map(|i| i + 1).unwrap_or(rest.len());
                    self.advance(end);
                    continue;
                } else if rest.starts_with('<') {
                    self.in_tag = true;
                    let end = rest[1..]
                        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
                        .map(|i| i + 1)
                        .unwrap_or(rest.len());
                    let name = &rest[1..end];
                    self.advance(end);
                    return Some(Token::TagOpen(name));
                } else {
                    let end = rest.find('<').unwrap_or(rest.len());
                    let text = &rest[..end];
                    self.advance(end);
                    let trimmed = text.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    return Some(Token::Text(decode_entities(trimmed)));
                }
            } else {
                // Inside a tag
                if rest.starts_with("/>") {
                    self.in_tag = false;
                    self.advance(2);
                    return Some(Token::TagSelfClose);
                } else if rest.starts_with('>') {
                    self.in_tag = false;
                    self.advance(1);
                    return Some(Token::TagEnd);
                } else {
                    return self.parse_attribute(rest);
                }
            }
        }
    }

    fn parse_attribute(&mut self, rest: &'a str) -> Option<Token<'a>> {
        // Check: if we reached the end of the tag before parsing an attribute
        if rest.starts_with('>') || rest.starts_with("/>") || rest.starts_with("?>") {
            return None;
        }

        let eq_pos = rest.find('=')?;
        let full_key = rest[..eq_pos].trim();

        let (ns, key) = if let Some(colon_pos) = full_key.find(':') {
            (Some(&full_key[..colon_pos]), &full_key[colon_pos + 1..])
        } else {
            (None, full_key)
        };

        let after_eq = rest[eq_pos + 1..].trim_start();
        let quote = after_eq.chars().next()?;
        let val_start = quote.len_utf8();
        let val_end = after_eq[val_start..].find(quote)? + val_start;
        let value = &after_eq[val_start..val_end];

        let total_consumed = rest.len() - after_eq[val_end + quote.len_utf8()..].len();
        self.advance(total_consumed);

        Some(Token::Attr(ns, key, decode_entities(value)))
    }
}
