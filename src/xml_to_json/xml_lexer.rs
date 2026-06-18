use crate::xml_to_json::xml_model::Token;

pub struct Lexer<'a> {
    input: &'a str,
    cursor: usize,
    in_tag: bool, // State: whether we are inside angle brackets
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input,
            cursor: 0,
            in_tag: false,
        }
    }

    /// Walks through the entire string and collects a vector of tokens
    pub fn tokenize(&mut self) -> Result<Vec<Token<'a>>, &'static str> {
        let mut tokens = Vec::new();

        while let Some(token) = self.next_token() {
            // Optional: logging for debugging
            // println!("Token: {:?}", token);
            if token == Token::EmptyTag {
                // Skip empty tags (like comments or declarations)
                continue;
            }

            tokens.push(token);
        }

        // Basic check: if we reached the end but are still "inside a tag",
        // then the XML is truncated (for example, "<root ")
        if self.in_tag {
            return Err("Unexpected end of input: tag not closed");
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
        let rest = self.remaining().trim_start();
        if rest.is_empty() {
            return None;
        }

        let diff = self.remaining().len() - rest.len();
        self.advance(diff);
        let rest = self.remaining();

        if !self.in_tag {
            if rest.starts_with("</") {
                // ... (closing tag logic unchanged)
                let end = rest.find('>')?;
                let name = &rest[2..end];
                self.advance(end + 1);
                Some(Token::TagClose(name))
            } else if rest.starts_with("<?") {
                // Special handling for the <?xml declaration
                self.in_tag = true;
                let end = rest[2..]
                    .find(|c: char| c.is_whitespace() || c == '?')
                    .map(|i| i + 2)
                    .unwrap_or(rest.len());
                let name = &rest[1..end]; // keep "?" in the name for distinction
                self.advance(end);
                Some(Token::TagOpen(name))
            } else if rest.starts_with("<!--") {
                // Special handling for comments <!--
                let end = rest.find("-->")?;
                self.advance(end + 3);
                Some(Token::EmptyTag)
            } else if rest.starts_with("<!") {
                // Skip <!DOCTYPE> and similar declarations to the closing >
                let end = rest.find('>').map(|i| i + 1).unwrap_or(rest.len());
                self.advance(end);
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
                // ... (text logic unchanged)
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
            // Inside a tag
            if rest.starts_with("?>") {
                // Closing the declaration
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
        // Check: if we reached the end of the tag before parsing an attribute
        if rest.starts_with('>') || rest.starts_with("/>") || rest.starts_with("?>") {
            return None;
        }

        let eq_pos = rest.find('=')?;
        let full_key = rest[..eq_pos].trim();

        // Important: if the key contains '?', we are mistakenly parsing the end of the declaration
        // But with the new logic in next_token, this should be skipped.

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
