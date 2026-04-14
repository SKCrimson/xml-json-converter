use crate::xml_to_json::xml_model::XmlNode;

mod xml_lexer;
mod xml_model;
mod xml_parser;

pub fn convert(xml: &str, pretty: bool) -> Result<String, &'static str> {
    let mut lexer = xml_lexer::Lexer::new(xml);
    let tokens = lexer.tokenize()?;
    // println!("Tokens: {:?}", tokens);

    let root_node = xml_parser::Parser::new(tokens).parse()?;

    let json_output = if let XmlNode::Element { name, .. } = &root_node {
        if pretty {
            format!("{{ \"{}\": {} }}", name, root_node.to_pretty_json(4))
        } else {
            format!("{{ \"{}\": {} }}", name, root_node.to_json())
        }
    } else {
        if pretty {
            root_node.to_pretty_json(4)
        } else {
            root_node.to_json()
        }
    };

    Ok(json_output)
}
