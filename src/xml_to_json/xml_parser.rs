use crate::xml_to_json::xml_model::Token;
use crate::xml_to_json::xml_model::XmlNode;

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Parser { tokens }
    }

    pub fn parse(self) -> Result<XmlNode, String> {
        let mut stack: Vec<XmlNode> = Vec::new();
        let mut root: Option<XmlNode> = None;

        for token in self.tokens {
            match token {
                Token::TagOpen(name) => {
                    if stack.is_empty() && root.is_some() {
                        return Err(format!("Second root element <{}>", name));
                    }
                    let new_node = XmlNode::Element {
                        name: name.to_string(),
                        attributes: Vec::new(),
                        children: Vec::new(),
                    };
                    stack.push(new_node);
                }

                Token::ProcessingInstruction => {}

                Token::Attr(key, value) => {
                    if let Some(XmlNode::Element { attributes, .. }) = stack.last_mut() {
                        attributes.push((key.to_string(), value.into_owned()));
                    }
                }

                Token::Text(text) => {
                    if let Some(XmlNode::Element { children, .. }) = stack.last_mut() {
                        children.push(XmlNode::Text(text.into_owned()));
                    } else if !text.trim().is_empty() {
                        return Err(format!(
                            "Unexpected text content outside root element: {:?}",
                            text.as_ref()
                        ));
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
                        if name.as_str() != close_name {
                            return Err(format!(
                                "Expected </{}>, found </{}>",
                                name,
                                close_name
                            ));
                        }
                    }
                    if stack.is_empty() {
                        root = Some(finished_node);
                    } else if let Some(XmlNode::Element { children, .. }) = stack.last_mut() {
                        children.push(finished_node);
                    }
                }
            }
        }

        if let Some(XmlNode::Element { name, .. }) = stack.last() {
            return Err(format!("Unclosed element <{}>", name));
        }

        root.ok_or_else(|| "Empty XML document".to_string())
    }
}
