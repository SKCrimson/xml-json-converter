use crate::json_to_xml::json_model::JsonNode;
use crate::json_to_xml::json_model::Token;

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    pos: usize,
}
impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<JsonNode<'a>, &'static str> {
        let token = self.consume().ok_or("Empty token stream")?;

        match token {
            Token::BraceOpen => self.parse_object(),
            Token::BracketOpen => self.parse_array(),
            Token::StringVal(s) => Ok(JsonNode::StringVal(*s)), // Добавляем *
            Token::Number(n) => Ok(JsonNode::Number(*n)),       // Добавляем *
            Token::BoolVal(b) => Ok(JsonNode::BoolVal(*b)),     // Добавляем *
            Token::Null => Ok(JsonNode::Null),
            _ => Err("Invalid start of JSON"),
        }
    }

    // Вспомогательные методы для навигации
    fn peek(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.pos)
    }

    fn consume(&mut self) -> Option<&Token<'a>> {
        let token = self.tokens.get(self.pos);
        self.pos += 1;
        token
    }

    fn expect(&mut self, token: Token<'a>) -> Result<(), &'static str> {
        if self.peek() == Some(&token) {
            self.consume();
            Ok(())
        } else {
            Err("Unexpected token")
        }
    }

    fn parse_object(&mut self) -> Result<JsonNode<'a>, &'static str> {
        let mut pairs = Vec::new();

        if let Some(Token::BraceClose) = self.peek() {
            self.consume();
            return Ok(JsonNode::Object(pairs));
        }

        loop {
            // Извлекаем ключ.
            // ВАЖНО: используем match, чтобы вытащить именно ссылку &'a str
            let key = match self.consume() {
                Some(Token::StringVal(s)) => *s, // s здесь это &&'a str, разыменовываем до &'a str
                _ => return Err("Expected string key in object"),
            };

            // Ожидаем двоеточие
            match self.consume() {
                Some(Token::Colon) => (),
                _ => return Err("Expected ':' after key"),
            };

            // Рекурсивно парсим значение
            let value = self.parse()?;
            pairs.push((key, value));

            // Проверяем разделитель или конец
            match self.consume() {
                Some(Token::BraceClose) => break,
                Some(Token::Comma) => continue,
                _ => return Err("Expected ',' or '}' in object"),
            }
        }

        Ok(JsonNode::Object(pairs))
    }

    fn parse_array(&mut self) -> Result<JsonNode<'a>, &'static str> {
        let mut elements = Vec::new();

        if let Some(Token::BracketClose) = self.peek() {
            self.consume();
            return Ok(JsonNode::Array(elements));
        }

        loop {
            elements.push(self.parse()?);

            match self.consume() {
                Some(Token::BracketClose) => break,
                Some(Token::Comma) => continue,
                _ => return Err("Expected ',' or ']' in array"),
            }
        }

        Ok(JsonNode::Array(elements))
    }
}
