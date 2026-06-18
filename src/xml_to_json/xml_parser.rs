use crate::xml_to_json::xml_model::Token;
use crate::xml_to_json::xml_model::XmlNode;

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<XmlNode, String> {
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
                        attributes: Vec::new(),
                        children: Vec::new(),
                    };
                    stack.push(new_node);
                }

                Token::Attr(_ns, key, value) => {
                    if let Some(XmlNode::Element { attributes, .. }) = stack.last_mut() {
                        attributes.push((key.to_string(), value.to_string()));
                    }
                }

                Token::Text(text) => {
                    if let Some(XmlNode::Element { children, .. }) = stack.last_mut() {
                        children.push(XmlNode::Text(text.to_string()));
                    }
                }

                Token::TagSelfClose => {
                    let finished_node = stack.pop().ok_or_else(|| "Unbalanced tags".to_string())?;
                    if stack.is_empty() {
                        root = Some(finished_node);
                    } else if let Some(XmlNode::Element { children, .. }) = stack.last_mut() {
                        children.push(finished_node);
                    }
                }

                Token::TagClose(close_name) => {
                    let finished_node = stack.pop().ok_or_else(|| "Unbalanced tags".to_string())?;
                    if let XmlNode::Element { ref name, .. } = finished_node {
                        if name != close_name.trim() {
                            return Err("Mismatched closing tag".to_string());
                        }
                    }
                    if stack.is_empty() {
                        root = Some(finished_node);
                    } else if let Some(XmlNode::Element { children, .. }) = stack.last_mut() {
                        children.push(finished_node);
                    }
                }

                Token::EmptyTag => {
                    // An empty tag, such as <br/> or a comment <!-- ... -->
                    // It has already been handled in the lexer, so we just ignore it
                }

                Token::TagEnd => { /* Just continue */ }
            }
        }

        root.ok_or_else(|| "Empty XML document".to_string())
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
