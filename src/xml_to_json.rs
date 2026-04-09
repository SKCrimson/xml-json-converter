mod xml_lexer;
mod xml_parser;

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug)]
enum XmlNode {
    Element {
        name: String,
        attributes: HashMap<String, String>,
        children: Vec<XmlNode>,
    },
    Text(String),
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
enum Token<'a> {
    TagOpen(&'a str),                        // <name
    TagClose(&'a str),                       // </name>
    Attr(Option<&'a str>, &'a str, &'a str), // (namespace, key, value)
    TagEnd,                                  // >
    TagSelfClose,                            // />
    Text(&'a str),                           // content
}

pub fn convert(xml: &str) -> Result<String, &'static str> {
    let mut lexer = xml_lexer::Lexer::new(xml);
    let tokens = lexer.tokenize()?;

    let root = xml_parser::Parser::new(tokens).parse()?;
    println!("Parsed XML Node: {:#?}", root);

    // let json = to_json(&root);

    //println!("XML Content: {}", xml);

    let json = r#"{"root":{"child":"value"}}"#.to_string(); // Placeholder for testing
    Ok(json)
}
