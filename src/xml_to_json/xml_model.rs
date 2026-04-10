use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    TagOpen(&'a str),                         // <name
    TagClose(&'a str),                        // </name>
    Attr(Option<&'a str>, &'a str, &'a str),  // (namespace, key, value)
    TagEnd,                                   // >
    TagSelfClose,                             // />
    EmptyTag,                                 //
    Text(&'a str),                            // content
}

#[derive(Debug)]
pub enum XmlNode {
    Element {
        name: String,
        attributes: HashMap<String, String>,
        children: Vec<XmlNode>,
    },
    Text(String),
}

impl XmlNode {
    pub fn to_json(&self) -> String {
        match self {
            XmlNode::Text(s) => format!("\"{}\"", Self::escape_json(s)),
            XmlNode::Element {
                name: _,
                attributes,
                children,
            } => {
                let mut parts = Vec::new();

                // 1. Process attributes
                for (key, value) in attributes {
                    parts.push(format!("\"@{}\": \"{}\"", key, Self::escape_json(value)));
                }

                // 2. Group child nodes by name to find duplicates (arrays)
                let mut grouped_children: HashMap<String, Vec<&XmlNode>> = HashMap::new();
                let mut text_content = Vec::new();

                for child in children {
                    match child {
                        XmlNode::Element { name, .. } => {
                            grouped_children
                                .entry(name.clone())
                                .or_default()
                                .push(child);
                        }
                        XmlNode::Text(t) => text_content.push(t),
                    }
                }

                // 3. Process grouped child nodes
                for (name, nodes) in grouped_children {
                    if nodes.len() > 1 {
                        // Array of elements
                        let items: Vec<String> = nodes.iter().map(|n| n.to_json()).collect();
                        parts.push(format!("\"{}\": [{}]", name, items.join(", ")));
                    } else {
                        // Single element
                        parts.push(format!("\"{}\": {}", name, nodes[0].to_json()));
                    }
                }

                // 4. If there is only text and no attributes/children, return just a string
                if parts.is_empty() && !text_content.is_empty() {
                    let joined_text = text_content
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(" ");
                    return format!("\"{}\"", Self::escape_json(&joined_text));
                }

                // If there is text along with other elements, add it as a special field
                if !text_content.is_empty() && !parts.is_empty() {
                    let joined_text = text_content
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(" ");
                    parts.push(format!(
                        "\"#text\": \"{}\"",
                        Self::escape_json(&joined_text)
                    ));
                }

                format!("{{ {} }}", parts.join(", "))
            }
        }
    }

    // Helper function for safely building JSON strings
    fn escape_json(s: &str) -> String {
        s.replace('\\', "\\\\") // First escape the backslash itself
            .replace('"', "\\\"") // Then escape quotes
            .replace('\n', "\\n") // (Optional) line breaks
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }
}
