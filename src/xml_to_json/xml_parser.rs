use crate::xml_to_json::xml_model::Token;
use crate::xml_to_json::xml_model::XmlNode;
use std::collections::HashMap;

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<XmlNode, &'static str> {
        let mut stack: Vec<XmlNode> = Vec::new();
        let mut root: Option<XmlNode> = None;

        while self.pos < self.tokens.len() {
            let token = &self.tokens[self.pos];
            self.pos += 1;

            match token {
                Token::TagOpen(name) => {
                    // Ignore declarations like <?xml ... ?>
                    if name.starts_with('?') {
                        self.skip_until_tag_end();
                        continue;
                    }

                    let new_node = XmlNode::Element {
                        name: name.to_string(),
                        attributes: HashMap::new(),
                        children: Vec::new(),
                    };
                    stack.push(new_node);
                }

                Token::Attr(_ns, key, value) => {
                    if let Some(XmlNode::Element { attributes, .. }) = stack.last_mut() {
                        attributes.insert(key.to_string(), value.to_string());
                    }
                }

                Token::Text(text) => {
                    if let Some(XmlNode::Element { children, .. }) = stack.last_mut() {
                        children.push(XmlNode::Text(text.to_string()));
                    }
                }

                Token::TagSelfClose | Token::TagClose(_) => {
                    let finished_node = {
                        let this = stack.pop();
                        match this {
                            Some(v) => Ok(v),
                            None => {
                                eprintln!(
                                    "Stack is empty when trying to pop. Current token: {:?}",
                                    token
                                );
                                Err("Unbalanced tags")
                            }
                        }
                    }?;

                    if stack.is_empty() {
                        root = Some(finished_node);
                    } else {
                        if let Some(XmlNode::Element { children, .. }) = stack.last_mut() {
                            children.push(finished_node);
                        }
                    }
                }

                Token::EmptyTag => {
                    // An empty tag, such as <br/> or a comment <!-- ... -->
                    // It has already been handled in the lexer, so we just ignore it
                }

                Token::TagEnd => { /* Just continue */ }
            }
        }

        root.ok_or("Empty XML document")
    }

    fn skip_until_tag_end(&mut self) {
        while self.pos < self.tokens.len() {
            if let Token::TagEnd = self.tokens[self.pos] {
                self.pos += 1;
                break;
            }
            self.pos += 1;
        }
    }
}
