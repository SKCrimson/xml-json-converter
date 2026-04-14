use std::iter::Peekable;
use std::str::CharIndices;
use crate::json_to_xml::json_model::Token;

pub struct Lexer<'a> {
    input: &'a str, // Храним исходную строку для создания срезов
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

    pub fn tokenize(&mut self) -> Result<Vec<Token<'a>>, &'static str> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token()? {
            tokens.push(token);
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Option<Token<'a>>, &'static str> {
        self.skip_whitespace();

        let (start_idx, c) = match self.consume() {
            Some(pair) => pair,
            None => return Ok(None),
        };

        match c {
            '{' => Ok(Some(Token::BraceOpen)),
            '}' => Ok(Some(Token::BraceClose)),
            '[' => Ok(Some(Token::BracketOpen)),
            ']' => Ok(Some(Token::BracketClose)),
            ':' => Ok(Some(Token::Colon)),
            ',' => Ok(Some(Token::Comma)),
            '"' => self
                .read_string(start_idx)
                .map(|s| Some(Token::StringVal(s))),
            'n' | 't' | 'f' | '-' | '0'..='9' => self.read_literal(start_idx, c),
            _ => Err("Unexpected character encountered"),
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

    fn read_string(&mut self, start_quote_idx: usize) -> Result<&'a str, &'static str> {
        // Мы ищем закрывающую кавычку, чтобы вернуть срез
        while let Some((idx, c)) = self.consume() {
            if c == '\\' {
                self.consume(); // Пропускаем следующий символ (escape)
            } else if c == '"' {
                // Возвращаем срез строки БЕЗ кавычек
                return Ok(&self.input[start_quote_idx + 1..idx]);
            }
        }
        Err("Unterminated string literal")
    }

    fn read_literal(
        &mut self,
        start_idx: usize,
        _first_char: char,
    ) -> Result<Option<Token<'a>>, &'static str> {
        let mut end_idx = start_idx;

        while let Some(&(idx, c)) = self.chars.peek() {
            if c.is_whitespace() || ",}] :".contains(c) {
                break;
            }
            let (i, _) = self.consume().unwrap();
            end_idx = i;
        }

        // Получаем срез всего литерала
        let s = &self.input[start_idx..=end_idx];

        match s {
            "true" => Ok(Some(Token::BoolVal(true))),
            "false" => Ok(Some(Token::BoolVal(false))),
            "null" => Ok(Some(Token::Null)),
            _ => {
                if let Ok(num) = s.parse::<f64>() {
                    Ok(Some(Token::Number(num)))
                } else {
                    Err("Invalid numeric or keyword literal")
                }
            }
        }
    }
}
