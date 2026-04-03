use std::collections::HashMap;

#[derive(Debug)]
enum XmlNode {
    Element {
        name: String,
        attributes: HashMap<String, String>,
        children: Vec<XmlNode>,
    },
    Text(String),
}

#[derive(Debug, PartialEq)]
enum Token<'a> {
    TagOpen(&'a str),      // <name
    TagClose(&'a str),     // </name>
    Attr(Option<&'a str>, &'a str, &'a str), // (namespace, key, value)
    TagEnd,                // >
    TagSelfClose,          // />
    Text(&'a str),         // содержимое
}