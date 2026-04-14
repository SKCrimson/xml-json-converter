mod json_lexer;
mod json_model;
mod json_parser;

pub fn convert(json: &str) -> Result<String, &'static str> {
    let mut lexer = json_lexer::Lexer::new(json);
    let tokens = lexer.tokenize()?;

    let root_node = json_parser::Parser::new(tokens).parse()?;

    Ok(root_node.to_xml())
}
