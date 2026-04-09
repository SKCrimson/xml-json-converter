use crate::xml_to_json::xml_model::XmlNode;

mod xml_lexer;
mod xml_model;
mod xml_parser;

pub fn convert(xml: &str) -> Result<String, &'static str> {
    let mut lexer = xml_lexer::Lexer::new(xml);
    let tokens = lexer.tokenize()?;

    let root_node = xml_parser::Parser::new(tokens).parse()?;

    let json_output = if let XmlNode::Element { name, .. } = &root_node {
        format!("{{ \"{}\": {} }}", name, root_node.to_json())
    } else {
        root_node.to_json()
    };

    Ok(json_output)
}
