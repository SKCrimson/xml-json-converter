mod xml_lexer;
mod xml_model;
mod xml_parser;

pub fn convert(xml: &str) -> Result<String, &'static str> {
    let mut lexer = xml_lexer::Lexer::new(xml);
    let tokens = lexer.tokenize()?;

    let root = xml_parser::Parser::new(tokens).parse()?;

    let json = root.to_json();

    Ok(json)
}
