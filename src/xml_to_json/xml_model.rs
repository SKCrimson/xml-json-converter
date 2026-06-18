use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    TagOpen(&'a str),                        // <name
    TagClose(&'a str),                       // </name>
    Attr(Option<&'a str>, &'a str, &'a str), // (namespace, key, value)
    TagEnd,                                  // >
    TagSelfClose,                            // />
    EmptyTag,                                //
    Text(&'a str),                           // content
}

#[derive(Debug)]
pub enum XmlNode {
    Element {
        name: String,
        attributes: Vec<(String, String)>,
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

                // 1. Attributes (order preserved from document)
                for (key, value) in attributes {
                    parts.push(format!("\"@{}\": \"{}\"", key, Self::escape_json(value)));
                }

                // 2. Group children by name, preserving order of first appearance
                let mut seen_order: Vec<&str> = Vec::new();
                let mut grouped_children: HashMap<&str, Vec<&XmlNode>> = HashMap::new();
                let mut text_content = Vec::new();

                for child in children {
                    match child {
                        XmlNode::Element { name, .. } => {
                            if !grouped_children.contains_key(name.as_str()) {
                                seen_order.push(name.as_str());
                            }
                            grouped_children
                                .entry(name.as_str())
                                .or_default()
                                .push(child);
                        }
                        XmlNode::Text(t) => text_content.push(t),
                    }
                }

                // 3. Process grouped child nodes in document order
                for name in &seen_order {
                    let nodes = &grouped_children[name];
                    if nodes.len() > 1 {
                        let items: Vec<String> = nodes.iter().map(|n| n.to_json()).collect();
                        parts.push(format!("\"{}\": [{}]", name, items.join(", ")));
                    } else {
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

    pub fn to_pretty_json(&self, indent_size: usize) -> String {
        self.to_json_recursive(0, indent_size)
    }

    fn to_json_recursive(&self, depth: usize, indent_size: usize) -> String {
        let current_indent = " ".repeat(depth * indent_size);
        let next_indent = " ".repeat((depth + 1) * indent_size);

        match self {
            XmlNode::Text(s) => format!("\"{}\"", Self::escape_json(s)),

            XmlNode::Element {
                attributes,
                children,
                ..
            } => {
                let mut parts = Vec::new();

                // 1. Attributes (order preserved from document)
                for (key, value) in attributes {
                    parts.push(format!("\"@{}\": \"{}\"", key, Self::escape_json(value)));
                }

                // 2. Group children by name, preserving order of first appearance
                let mut seen_order: Vec<&str> = Vec::new();
                let mut grouped_children: HashMap<&str, Vec<&XmlNode>> = HashMap::new();
                let mut text_content = Vec::new();

                for child in children {
                    match child {
                        XmlNode::Element { name, .. } => {
                            if !grouped_children.contains_key(name.as_str()) {
                                seen_order.push(name.as_str());
                            }
                            grouped_children
                                .entry(name.as_str())
                                .or_default()
                                .push(child);
                        }
                        XmlNode::Text(t) => text_content.push(t),
                    }
                }

                // 3. Process grouped child nodes in document order
                for name in &seen_order {
                    let nodes = &grouped_children[name];
                    if nodes.len() > 1 {
                        let items: Vec<String> = nodes
                            .iter()
                            .map(|n| n.to_json_recursive(depth + 2, indent_size))
                            .collect();

                        let array_body =
                            items.join(&format!(",\n{}", " ".repeat((depth + 2) * indent_size)));
                        parts.push(format!(
                            "\"{}\": [\n{}{}\n{}]",
                            name,
                            " ".repeat((depth + 2) * indent_size),
                            array_body,
                            next_indent
                        ));
                    } else {
                        parts.push(format!(
                            "\"{}\": {}",
                            name,
                            nodes[0].to_json_recursive(depth + 1, indent_size)
                        ));
                    }
                }

                // 4. Text content
                let joined_text = text_content
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");

                if parts.is_empty() && !text_content.is_empty() {
                    return format!("\"{}\"", Self::escape_json(&joined_text));
                }

                if !text_content.is_empty() {
                    parts.push(format!(
                        "\"#text\": \"{}\"",
                        Self::escape_json(&joined_text)
                    ));
                }

                let mut result = String::from("{\n");
                for (i, part) in parts.iter().enumerate() {
                    result.push_str(&next_indent);
                    result.push_str(part);
                    if i < parts.len() - 1 {
                        result.push(',');
                    }
                    result.push('\n');
                }
                result.push_str(&current_indent);
                result.push('}');
                result
            }
        }
    }

    fn escape_json(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn elem(name: &str, children: Vec<XmlNode>) -> XmlNode {
        XmlNode::Element {
            name: name.to_string(),
            attributes: Vec::new(),
            children,
        }
    }

    fn elem_attr(name: &str, attrs: &[(&str, &str)]) -> XmlNode {
        let attributes = attrs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        XmlNode::Element {
            name: name.to_string(),
            attributes,
            children: Vec::new(),
        }
    }

    #[test]
    fn text_node_to_json() {
        let node = XmlNode::Text("hello".to_string());
        assert_eq!(node.to_json(), "\"hello\"");
    }

    #[test]
    fn text_node_escapes_quotes() {
        let node = XmlNode::Text("say \"hi\"".to_string());
        let json = node.to_json();
        assert!(json.contains("\\\""));
    }

    #[test]
    fn text_node_escapes_backslash() {
        let node = XmlNode::Text("C:\\path".to_string());
        let json = node.to_json();
        assert!(json.contains("\\\\"));
    }

    #[test]
    fn element_with_only_text_returns_string() {
        let node = elem("root", vec![XmlNode::Text("value".to_string())]);
        assert_eq!(node.to_json(), "\"value\"");
    }

    #[test]
    fn element_with_attribute_has_at_prefix() {
        let node = elem_attr("root", &[("id", "1")]);
        let json = node.to_json();
        assert!(json.contains("\"@id\": \"1\""));
    }

    #[test]
    fn repeated_children_produce_array() {
        let node = elem(
            "root",
            vec![
                elem("item", vec![XmlNode::Text("a".to_string())]),
                elem("item", vec![XmlNode::Text("b".to_string())]),
            ],
        );
        let json = node.to_json();
        assert!(json.contains('['));
        assert!(json.contains("\"item\""));
    }

    #[test]
    fn unique_children_produce_object_keys() {
        let node = elem(
            "root",
            vec![
                elem("a", vec![XmlNode::Text("1".to_string())]),
                elem("b", vec![XmlNode::Text("2".to_string())]),
            ],
        );
        let json = node.to_json();
        assert!(json.contains("\"a\""));
        assert!(json.contains("\"b\""));
        assert!(!json.contains('['));
    }

    #[test]
    fn child_order_preserved() {
        let node = elem(
            "root",
            vec![
                elem("a", vec![XmlNode::Text("1".to_string())]),
                elem("b", vec![XmlNode::Text("2".to_string())]),
                elem("c", vec![XmlNode::Text("3".to_string())]),
            ],
        );
        let json = node.to_json();
        let pos_a = json.find("\"a\"").unwrap();
        let pos_b = json.find("\"b\"").unwrap();
        let pos_c = json.find("\"c\"").unwrap();
        assert!(pos_a < pos_b && pos_b < pos_c);
    }

    #[test]
    fn mixed_text_and_children_adds_hash_text() {
        let node = XmlNode::Element {
            name: "root".to_string(),
            attributes: Vec::new(),
            children: vec![
                XmlNode::Text("note".to_string()),
                elem("child", vec![]),
            ],
        };
        let json = node.to_json();
        assert!(json.contains("\"#text\""));
        assert!(json.contains("\"note\""));
    }

    #[test]
    fn pretty_json_contains_newlines() {
        let node = elem("root", vec![elem("child", vec![XmlNode::Text("x".to_string())])]);
        let pretty = node.to_pretty_json(4);
        assert!(pretty.contains('\n'));
    }

    #[test]
    fn empty_element_produces_empty_object() {
        let node = elem("root", vec![]);
        let json = node.to_json();
        assert!(json.contains('{'));
        assert!(json.contains('}'));
    }
}
