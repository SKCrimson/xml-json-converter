use crate::json_to_xml::json_model::JsonNode;
use crate::json_to_xml::json_model::Token;

const MAX_DEPTH: usize = 512;

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<JsonNode<'a>, String> {
        self.parse_value(0)
    }

    fn parse_value(&mut self, depth: usize) -> Result<JsonNode<'a>, String> {
        if depth >= MAX_DEPTH {
            return Err("Nesting depth limit exceeded".to_string());
        }
        let token = self.consume().ok_or_else(|| "Empty token stream".to_string())?;

        match token {
            Token::BraceOpen => self.parse_object(depth + 1),
            Token::BracketOpen => self.parse_array(depth + 1),
            Token::StringVal(s) => Ok(JsonNode::StringVal(s.clone())),
            Token::Number(n) => Ok(JsonNode::Number(*n)),
            Token::BoolVal(b) => Ok(JsonNode::BoolVal(*b)),
            Token::Null => Ok(JsonNode::Null),
            _ => Err("Invalid start of JSON".to_string()),
        }
    }

    fn peek(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.pos)
    }

    fn consume(&mut self) -> Option<&Token<'a>> {
        let token = self.tokens.get(self.pos);
        self.pos += 1;
        token
    }

    fn expect(&mut self, expected: Token<'a>, error_msg: &'static str) -> Result<(), String> {
        match self.consume() {
            Some(t) if t == &expected => Ok(()),
            _ => Err(error_msg.to_string()),
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<JsonNode<'a>, String> {
        let mut pairs = Vec::new();

        if let Some(Token::BraceClose) = self.peek() {
            self.consume();
            return Ok(JsonNode::Object(pairs));
        }

        loop {
            let key = match self.consume() {
                Some(Token::StringVal(s)) => s.clone(),
                _ => return Err("Expected string key in object".to_string()),
            };

            self.expect(Token::Colon, "Expected ':' after object key")?;

            let value = self.parse_value(depth)?;
            pairs.push((key, value));

            match self.consume() {
                Some(Token::BraceClose) => break,
                Some(Token::Comma) => continue,
                _ => return Err("Expected ',' or '}' in object".to_string()),
            }
        }
        Ok(JsonNode::Object(pairs))
    }

    fn parse_array(&mut self, depth: usize) -> Result<JsonNode<'a>, String> {
        let mut elements = Vec::new();

        if let Some(Token::BracketClose) = self.peek() {
            self.consume();
            return Ok(JsonNode::Array(elements));
        }

        loop {
            elements.push(self.parse_value(depth)?);

            match self.consume() {
                Some(Token::BracketClose) => break,
                Some(Token::Comma) => continue,
                _ => return Err("Expected ',' or ']' in array".to_string()),
            }
        }

        Ok(JsonNode::Array(elements))
    }
}
