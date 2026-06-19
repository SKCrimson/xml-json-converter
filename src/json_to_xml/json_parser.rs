use crate::json_to_xml::json_model::JsonNode;
use crate::json_to_xml::json_model::Token;
use super::json_model::MAX_DEPTH;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<JsonNode, String> {
        let node = self.parse_value(0)?;
        if let Some(token) = self.peek() {
            let what = match token {
                Token::BraceOpen    => "'{'",
                Token::BracketOpen  => "'['",
                Token::StringVal(_) => "string",
                Token::Number(_)    => "number",
                Token::BoolVal(_)   => "boolean",
                Token::Null         => "'null'",
                _                   => "token",
            };
            return Err(format!("Unexpected {} after top-level JSON value", what));
        }
        Ok(node)
    }

    fn parse_value(&mut self, depth: usize) -> Result<JsonNode, String> {
        if depth >= MAX_DEPTH {
            return Err("Nesting depth limit exceeded".to_string());
        }
        let token = self.consume().ok_or_else(|| "Empty token stream".to_string())?;

        match token {
            Token::BraceOpen => self.parse_object(depth + 1),
            Token::BracketOpen => self.parse_array(depth + 1),
            Token::StringVal(s) => Ok(JsonNode::StringVal(s.clone())),
            Token::Number(n) => Ok(JsonNode::Number(n.clone())),
            Token::BoolVal(b) => Ok(JsonNode::BoolVal(*b)),
            Token::Null => Ok(JsonNode::Null),
            _ => Err("Invalid start of JSON".to_string()),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn consume(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.pos);
        self.pos += 1;
        token
    }

    fn expect(&mut self, expected: Token, error_msg: &'static str) -> Result<(), String> {
        match self.consume() {
            Some(t) if t == &expected => Ok(()),
            _ => Err(error_msg.to_string()),
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<JsonNode, String> {
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

    fn parse_array(&mut self, depth: usize) -> Result<JsonNode, String> {
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
