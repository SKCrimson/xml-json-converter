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

    pub fn tokenize(&mut self) -> Result<Vec<Token<'a>>, String> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token()? {
            tokens.push(token);
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Option<Token<'a>>, String> {
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
                .read_string(start_idx, token_line, token_col)
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

    fn read_string(
        &mut self,
        start_quote_idx: usize,
        line: usize,
        col: usize,
    ) -> Result<&'a str, String> {
        while let Some((idx, c)) = self.consume() {
            if c == '\\' {
                self.consume();
            } else if c == '"' {
                return Ok(&self.input[start_quote_idx + 1..idx]);
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
    ) -> Result<Option<Token<'a>>, String> {
        let mut end_idx = start_idx;

        while let Some(&(_, c)) = self.chars.peek() {
            if c.is_whitespace() || ",}] :".contains(c) {
                break;
            }
            if let Some((i, _)) = self.consume() {
                end_idx = i;
            }
        }

        let s = &self.input[start_idx..=end_idx];

        match s {
            "true" => Ok(Some(Token::BoolVal(true))),
            "false" => Ok(Some(Token::BoolVal(false))),
            "null" => Ok(Some(Token::Null)),
            _ => {
                if s.parse::<f64>().is_ok() {
                    Ok(Some(Token::Number(s)))
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
