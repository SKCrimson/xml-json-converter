use std::borrow::Cow;
use crate::xml_to_json::xml_model::Token;

fn decode_entities<'a>(s: &'a str) -> Cow<'a, str> {
    if !s.contains('&') {
        return Cow::Borrowed(s);
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

    Cow::Owned(out)
}

fn find_doctype_subset_end(s: &str) -> usize {
    // Scan the DTD internal subset body (everything after the opening '['),
    // returning the number of bytes consumed up to and including the closing "]>".
    // Tracks quoted strings and <!-- comments --> so that "]>" inside either
    // does not close the subset prematurely.
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut in_quote: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        match in_quote {
            Some(q) => {
                if b == q { in_quote = None; }
                i += 1;
            }
            None => {
                if bytes[i..].starts_with(b"<!--") {
                    match s[i..].find("-->") {
                        Some(end) => { i += end + 3; }
                        None => return s.len(),
                    }
                } else {
                    match b {
                        b'"' | b'\'' => { in_quote = Some(b); i += 1; }
                        b']' if bytes.get(i + 1) == Some(&b'>') => return i + 2,
                        _ => i += 1,
                    }
                }
            }
        }
    }
    s.len()
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
            let (line, col) = self.position();
            return Err(format!("Unexpected end of input: tag not closed at line {}, col {}", line, col));
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

    fn position_at(&self, byte_offset: usize) -> (usize, usize) {
        let prefix = &self.input[..byte_offset.min(self.input.len())];
        let line = prefix.bytes().filter(|&b| b == b'\n').count() + 1;
        let col = match prefix.rfind('\n') {
            Some(i) => prefix[i + 1..].chars().count() + 1,
            None    => prefix.chars().count() + 1,
        };
        (line, col)
    }

    fn position(&self) -> (usize, usize) {
        self.position_at(self.cursor)
    }

    fn next_token(&mut self) -> Option<Token<'a>> {
        loop {
            let rest = self.remaining();
            if rest.is_empty() {
                return None;
            }

            if self.in_tag {
                // Skip whitespace between attributes
                let trimmed = rest.trim_start();
                let diff = rest.len() - trimmed.len();
                self.advance(diff);
                let rest = self.remaining();

                if rest.starts_with("/>") {
                    self.in_tag = false;
                    self.advance(2);
                    return Some(Token::TagSelfClose);
                } else if rest.starts_with('>') {
                    self.in_tag = false;
                    self.advance(1);
                    continue;
                } else {
                    return self.parse_attribute(rest);
                }
            } else if rest.starts_with("</") {
                let end = match rest.find('>') {
                    Some(i) => i,
                    None => {
                        let (line, col) = self.position();
                        self.error = Some(format!("Unexpected EOF in closing tag at line {}, col {}", line, col));
                        return None;
                    }
                };
                let name = rest[2..end].trim();
                self.advance(end + 1);
                return Some(Token::TagClose(name));
            } else if rest.starts_with("<?") {
                match rest.find("?>") {
                    Some(end) => {
                        self.advance(end + 2);
                        return Some(Token::ProcessingInstruction);
                    }
                    None => {
                        let (line, col) = self.position();
                        self.error = Some(format!("Unclosed processing instruction at line {}, col {}", line, col));
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
                        let (line, col) = self.position();
                        self.error = Some(format!("Unclosed comment at line {}, col {}", line, col));
                        return None;
                    }
                }
            } else if rest.starts_with("<![CDATA[") {
                const PREFIX: usize = "<![CDATA[".len();
                match rest.find("]]>") {
                    Some(end) => {
                        let content = &rest[PREFIX..end];
                        self.advance(end + "]]>".len());
                        if !content.is_empty() {
                            return Some(Token::Text(Cow::Borrowed(content)));
                        }
                        continue;
                    }
                    None => {
                        let (line, col) = self.position();
                        self.error = Some(format!("Unclosed CDATA section at line {}, col {}", line, col));
                        return None;
                    }
                }
            } else if rest.starts_with("<!") {
                // Skip <!DOCTYPE> and similar declarations.
                // If "[" appears before the first ">", an internal subset is present:
                // scan past it with find_doctype_subset_end so that "]>" inside
                // quoted strings or comments does not close the subset prematurely.
                let end = match rest.find('[') {
                    Some(bracket) if rest.find('>').map_or(true, |gt| bracket < gt) => {
                        bracket + 1 + find_doctype_subset_end(&rest[bracket + 1..])
                    }
                    _ => rest.find('>').map(|i| i + 1).unwrap_or(rest.len()),
                };
                self.advance(end);
                continue;
            } else if rest.starts_with('<') {
                self.in_tag = true;
                let end = rest[1..]
                    .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
                    .map(|i| i + 1)
                    .unwrap_or(rest.len());
                let name = &rest[1..end];
                if name.is_empty() {
                    let (line, col) = self.position();
                    self.error = Some(format!("Empty tag name at line {}, col {}", line, col));
                    return None;
                }
                self.advance(end);
                return Some(Token::TagOpen(name));
            } else {
                let end = rest.find('<').unwrap_or(rest.len());
                let text = &rest[..end];
                self.advance(end);
                if text.is_empty() {
                    continue;
                }
                return Some(Token::Text(decode_entities(text)));
            }
        }
    }

    fn parse_attribute(&mut self, rest: &'a str) -> Option<Token<'a>> {
        // Attribute name: stop at whitespace, '=', '>', or '/'
        let name_end = rest
            .find(|c: char| c.is_whitespace() || c == '=' || c == '>' || c == '/')
            .unwrap_or(rest.len());
        let key = &rest[..name_end];
        let mut pos = name_end;

        // Skip optional whitespace, then require '='
        pos += rest[pos..].len() - rest[pos..].trim_start().len();
        if !rest[pos..].starts_with('=') {
            let (line, col) = self.position_at(self.cursor + pos);
            self.error = Some(format!("Attribute without '=' in tag at line {}, col {}", line, col));
            return None;
        }
        pos += 1; // '='

        // Skip optional whitespace, then require an opening quote
        pos += rest[pos..].len() - rest[pos..].trim_start().len();
        let quote = rest[pos..].chars().next()?;
        if quote != '"' && quote != '\'' {
            let (line, col) = self.position_at(self.cursor + pos);
            self.error = Some(format!(
                "Attribute value must be quoted with '\"' or \"'\" (found '{}' after '=') at line {}, col {}",
                quote, line, col
            ));
            return None;
        }
        pos += 1; // opening quote (both '"' and '\'' are single-byte ASCII)

        // Find the matching closing quote and extract the value
        let val_end = match rest[pos..].find(quote) {
            Some(i) => i + pos,
            None => {
                // pos is already past the opening quote; point back to the quote itself
                let (line, col) = self.position_at(self.cursor + pos - 1);
                self.error = Some(format!(
                    "Unterminated attribute value at line {}, col {}",
                    line, col
                ));
                return None;
            }
        };
        let value = &rest[pos..val_end];
        pos = val_end + 1; // closing quote

        self.advance(pos);
        Some(Token::Attr(key, decode_entities(value)))
    }
}
