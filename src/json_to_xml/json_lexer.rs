use crate::json_to_xml::json_model::Token;
use std::iter::Peekable;
use std::str::CharIndices;

pub struct Lexer<'a> {
    input: &'a str,
    chars: Peekable<CharIndices<'a>>,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input,
            chars: input.char_indices().peekable(),
            line: 1,
            col: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token()? {
            tokens.push(token);
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Option<Token>, String> {
        self.skip_whitespace();

        let (start_idx, c) = match self.consume() {
            Some(pair) => pair,
            None => return Ok(None),
        };

        let token_line = self.line;
        let token_col = self.col;

        match c {
            '{' => Ok(Some(Token::BraceOpen)),
            '}' => Ok(Some(Token::BraceClose)),
            '[' => Ok(Some(Token::BracketOpen)),
            ']' => Ok(Some(Token::BracketClose)),
            ':' => Ok(Some(Token::Colon)),
            ',' => Ok(Some(Token::Comma)),
            '"' => self
                .read_string(token_line, token_col)
                .map(|s| Some(Token::StringVal(s))),
            'n' | 't' | 'f' | '-' | '0'..='9' => {
                self.read_literal(start_idx, token_line, token_col)
            }
            _ => Err(format!(
                "Unexpected character '{}' at line {}, col {}",
                c, token_line, token_col
            )),
        }
    }

    fn consume(&mut self) -> Option<(usize, char)> {
        let (idx, c) = self.chars.next()?;
        if c == '\n' {
            self.line += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }
        Some((idx, c))
    }

    fn skip_whitespace(&mut self) {
        while let Some(&(_, c)) = self.chars.peek() {
            if c.is_whitespace() {
                self.consume();
            } else {
                break;
            }
        }
    }

    fn read_hex4(&mut self, line: usize, col: usize) -> Result<u32, String> {
        let mut hex = String::with_capacity(4);
        for _ in 0..4 {
            match self.consume() {
                Some((_, h)) => hex.push(h),
                None => return Err(format!(
                    "Unexpected end in unicode escape at line {}, col {}",
                    line, col
                )),
            }
        }
        u32::from_str_radix(&hex, 16).map_err(|_| {
            format!("Invalid unicode escape \\u{} at line {}, col {}", hex, line, col)
        })
    }

    fn read_string(&mut self, line: usize, col: usize) -> Result<String, String> {
        let mut out = String::new();
        while let Some((_, c)) = self.consume() {
            if c == '\\' {
                match self.consume() {
                    Some((_, '"')) => out.push('"'),
                    Some((_, '\\')) => out.push('\\'),
                    Some((_, '/')) => out.push('/'),
                    Some((_, 'n')) => out.push('\n'),
                    Some((_, 'r')) => out.push('\r'),
                    Some((_, 't')) => out.push('\t'),
                    Some((_, 'b')) => out.push('\x08'),
                    Some((_, 'f')) => out.push('\x0C'),
                    Some((_, 'u')) => {
                        let code = self.read_hex4(line, col)?;
                        let ch = if (0xD800..=0xDBFF).contains(&code) {
                            // High surrogate — RFC 8259 §7: must be followed by \uDC00–\uDFFF
                            if self.consume().map(|(_, c)| c) != Some('\\') {
                                return Err(format!(
                                    "High surrogate \\u{:04X} not followed by \\u at line {}, col {}",
                                    code, line, col
                                ));
                            }
                            if self.consume().map(|(_, c)| c) != Some('u') {
                                return Err(format!(
                                    "High surrogate \\u{:04X} not followed by \\u at line {}, col {}",
                                    code, line, col
                                ));
                            }
                            let low = self.read_hex4(line, col)?;
                            if !(0xDC00..=0xDFFF).contains(&low) {
                                return Err(format!(
                                    "Expected low surrogate after \\u{:04X}, got \\u{:04X} at line {}, col {}",
                                    code, low, line, col
                                ));
                            }
                            let codepoint = 0x10000 + ((code - 0xD800) << 10) + (low - 0xDC00);
                            char::from_u32(codepoint)
                                .ok_or_else(|| format!("Invalid surrogate pair at line {}, col {}", line, col))?
                        } else if (0xDC00..=0xDFFF).contains(&code) {
                            return Err(format!(
                                "Lone low surrogate \\u{:04X} at line {}, col {}",
                                code, line, col
                            ));
                        } else {
                            char::from_u32(code).ok_or_else(|| {
                                format!("Invalid unicode codepoint U+{:04X} at line {}, col {}", code, line, col)
                            })?
                        };
                        out.push(ch);
                    }
                    Some((_, c)) => return Err(format!(
                        "Invalid escape '\\{}' at line {}, col {}",
                        c, line, col
                    )),
                    None => return Err(format!(
                        "Unexpected end after '\\' at line {}, col {}",
                        line, col
                    )),
                }
            } else if c == '"' {
                return Ok(out);
            } else {
                out.push(c);
            }
        }
        Err(format!(
            "Unterminated string literal at line {}, col {}",
            line, col
        ))
    }

    fn read_literal(
        &mut self,
        start_idx: usize,
        line: usize,
        col: usize,
    ) -> Result<Option<Token>, String> {
        let mut end_idx = start_idx;
        // First char (n/t/f/digit/minus) is always ASCII — length is 1.
        let mut last_char_len: usize = 1;

        while let Some(&(_, c)) = self.chars.peek() {
            if c.is_whitespace() || ",}] :".contains(c) {
                break;
            }
            if let Some((i, ch)) = self.consume() {
                end_idx = i;
                last_char_len = ch.len_utf8();
            }
        }

        let s = &self.input[start_idx..end_idx + last_char_len];

        match s {
            "true" => Ok(Some(Token::BoolVal(true))),
            "false" => Ok(Some(Token::BoolVal(false))),
            "null" => Ok(Some(Token::Null)),
            _ => {
                if s.parse::<f64>().is_ok() {
                    Ok(Some(Token::Number(s.to_string())))
                } else {
                    Err(format!(
                        "Invalid literal '{}' at line {}, col {}",
                        s, line, col
                    ))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multibyte_char_after_digit_does_not_panic() {
        // "1€" — '€' is 3 bytes; the slice must not cut it in the middle.
        // The literal is invalid JSON, but the error must be a clean message, not a panic.
        let result = Lexer::new("1€").tokenize();
        assert!(result.is_err());
    }

    #[test]
    fn valid_integer_literal() {
        let tokens = Lexer::new("42").tokenize().unwrap();
        assert_eq!(tokens, vec![Token::Number("42".to_string())]);
    }

    #[test]
    fn valid_bool_true() {
        let tokens = Lexer::new("true").tokenize().unwrap();
        assert_eq!(tokens, vec![Token::BoolVal(true)]);
    }

    #[test]
    fn valid_null_literal() {
        let tokens = Lexer::new("null").tokenize().unwrap();
        assert_eq!(tokens, vec![Token::Null]);
    }

    #[test]
    fn surrogate_pair_emoji_decoded() {
        // 😀 → 😀 (U+1F600)
        let tokens = Lexer::new("\"\\uD83D\\uDE00\"").tokenize().unwrap();
        assert_eq!(tokens, vec![Token::StringVal("😀".to_string())]);
    }

    #[test]
    fn surrogate_pair_linear_b_decoded() {
        // 𐀀 → 𐀀 (U+10000, first supplementary character)
        let tokens = Lexer::new("\"\\uD800\\uDC00\"").tokenize().unwrap();
        assert_eq!(tokens, vec![Token::StringVal("\u{10000}".to_string())]);
    }

    #[test]
    fn lone_high_surrogate_returns_error() {
        assert!(Lexer::new("\"\\uD800\"").tokenize().is_err());
    }

    #[test]
    fn lone_low_surrogate_returns_error() {
        assert!(Lexer::new("\"\\uDC00\"").tokenize().is_err());
    }

    #[test]
    fn high_surrogate_followed_by_non_surrogate_u_returns_error() {
        // \uD800A — second escape is not a low surrogate
        assert!(Lexer::new("\"\\uD800\\u0041\"").tokenize().is_err());
    }
}
