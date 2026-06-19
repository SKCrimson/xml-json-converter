use std::collections::HashMap;

const MAX_DEPTH: usize = 512;

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    TagOpen(&'a str),                        // <name
    TagClose(&'a str),                       // </name>
    Attr(&'a str, String),                    // (full-name, decoded-value)
    TagSelfClose,                            // />
    ProcessingInstruction,                   // <?...?>
    Text(String),                            // content (entity-decoded)
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
    pub fn to_json(&self) -> Result<String, String> {
        self.to_json_impl(0, None)
    }

    pub(super) fn to_pretty_json_at(&self, depth: usize, indent_size: usize) -> Result<String, String> {
        self.to_json_impl(depth, Some(indent_size))
    }

    fn to_json_impl(&self, depth: usize, indent: Option<usize>) -> Result<String, String> {
        if depth >= MAX_DEPTH {
            return Err("Nesting depth limit exceeded".to_string());
        }
        match self {
            XmlNode::Text(s) => Ok(format!("\"{}\"", Self::escape_json(s))),
            XmlNode::Element { attributes, children, .. } => {
                let mut parts: Vec<String> = Vec::new();

                // 1. Attributes (order preserved from document)
                for (key, value) in attributes {
                    parts.push(format!("\"@{}\": \"{}\"", key, Self::escape_json(value)));
                }

                // 2. Group children by name, preserving order of first appearance
                let mut groups: Vec<(&str, Vec<&XmlNode>)> = Vec::new();
                let mut group_index: HashMap<&str, usize> = HashMap::new();
                let mut text_content: Vec<&String> = Vec::new();

                for child in children {
                    match child {
                        XmlNode::Element { name, .. } => {
                            if let Some(&idx) = group_index.get(name.as_str()) {
                                groups[idx].1.push(child);
                            } else {
                                group_index.insert(name.as_str(), groups.len());
                                groups.push((name.as_str(), vec![child]));
                            }
                        }
                        XmlNode::Text(t) => text_content.push(t),
                    }
                }

                // 3. Serialize grouped children in document order
                for (name, nodes) in &groups {
                    if nodes.len() > 1 {
                        let items: Vec<String> = nodes
                            .iter()
                            .map(|n| n.to_json_impl(depth + 1, indent))
                            .collect::<Result<Vec<_>, _>>()?;
                        let array_val = match indent {
                            Some(sz) => {
                                let item_pad = " ".repeat((depth + 2) * sz);
                                let close_pad = " ".repeat((depth + 1) * sz);
                                let sep = format!(",\n{}", item_pad);
                                format!("[\n{}{}\n{}]", item_pad, items.join(&sep), close_pad)
                            }
                            None => format!("[{}]", items.join(", ")),
                        };
                        parts.push(format!("\"{}\": {}", name, array_val));
                    } else {
                        parts.push(format!(
                            "\"{}\": {}",
                            name,
                            nodes[0].to_json_impl(depth + 1, indent)?
                        ));
                    }
                }

                // 4. Text content — when element children are present, whitespace-only
                // runs are formatting noise (indentation); drop them. When there are no
                // element children, whitespace IS the content and must be kept.
                let effective_text: Vec<&String> = if groups.is_empty() {
                    text_content.clone()
                } else {
                    text_content.iter().filter(|t| !t.trim().is_empty()).copied().collect()
                };

                let joined = effective_text.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("");

                if parts.is_empty() && !effective_text.is_empty() {
                    return Ok(format!("\"{}\"", Self::escape_json(&joined)));
                }
                if !effective_text.is_empty() {
                    parts.push(format!("\"#text\": \"{}\"", Self::escape_json(&joined)));
                }

                // 5. Assemble the object
                Ok(match indent {
                    Some(sz) => {
                        let current_pad = " ".repeat(depth * sz);
                        let next_pad = " ".repeat((depth + 1) * sz);
                        let mut out = String::from("{\n");
                        for (i, part) in parts.iter().enumerate() {
                            out.push_str(&next_pad);
                            out.push_str(part);
                            if i + 1 < parts.len() {
                                out.push(',');
                            }
                            out.push('\n');
                        }
                        out.push_str(&current_pad);
                        out.push('}');
                        out
                    }
                    None => if parts.is_empty() {
                        "{}".to_string()
                    } else {
                        format!("{{ {} }}", parts.join(", "))
                    },
                })
            }
        }
    }

    fn escape_json(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for c in s.chars() {
            match c {
                '\\' => out.push_str("\\\\"),
                '"' => out.push_str("\\\""),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if (c as u32) < 0x20 => {
                    out.push_str(&format!("\\u{:04X}", c as u32));
                }
                c => out.push(c),
            }
        }
        out
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
        assert_eq!(node.to_json().unwrap(), "\"hello\"");
    }

    #[test]
    fn text_node_escapes_quotes() {
        let node = XmlNode::Text("say \"hi\"".to_string());
        let json = node.to_json().unwrap();
        assert!(json.contains("\\\""));
    }

    #[test]
    fn text_node_escapes_backslash() {
        let node = XmlNode::Text("C:\\path".to_string());
        let json = node.to_json().unwrap();
        assert!(json.contains("\\\\"));
    }

    #[test]
    fn element_with_only_text_returns_string() {
        let node = elem("root", vec![XmlNode::Text("value".to_string())]);
        assert_eq!(node.to_json().unwrap(), "\"value\"");
    }

    #[test]
    fn element_with_attribute_has_at_prefix() {
        let node = elem_attr("root", &[("id", "1")]);
        let json = node.to_json().unwrap();
        assert!(json.contains("\"@id\": \"1\""));
    }

    #[test]
    fn attribute_order_preserved() {
        let node = elem_attr("root", &[("first", "1"), ("second", "2"), ("third", "3")]);
        let json = node.to_json().unwrap();
        let pos_first = json.find("\"@first\"").unwrap();
        let pos_second = json.find("\"@second\"").unwrap();
        let pos_third = json.find("\"@third\"").unwrap();
        assert!(pos_first < pos_second && pos_second < pos_third);
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
        let json = node.to_json().unwrap();
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
        let json = node.to_json().unwrap();
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
        let json = node.to_json().unwrap();
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
        let json = node.to_json().unwrap();
        assert!(json.contains("\"#text\""));
        assert!(json.contains("\"note\""));
    }

    #[test]
    fn multiple_text_nodes_joined_without_separator() {
        // "a " and " b" already carry their own whitespace — join must not add more
        let node = XmlNode::Element {
            name: "p".to_string(),
            attributes: Vec::new(),
            children: vec![
                XmlNode::Text("a ".to_string()),
                elem("b", vec![]),
                XmlNode::Text(" c".to_string()),
            ],
        };
        let json = node.to_json().unwrap();
        assert!(json.contains("\"a  c\""), "got: {}", json);
    }

    #[test]
    fn pretty_json_contains_newlines() {
        let node = elem("root", vec![elem("child", vec![XmlNode::Text("x".to_string())])]);
        let pretty = node.to_pretty_json_at(0, 4).unwrap();
        assert!(pretty.contains('\n'));
    }

    #[test]
    fn empty_element_produces_empty_object() {
        let node = elem("root", vec![]);
        let json = node.to_json().unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn control_chars_are_escaped_as_unicode() {
        let node = XmlNode::Text("\x01\x1F".to_string());
        let json = node.to_json().unwrap();
        assert!(json.contains("\\u0001"), "got: {}", json);
        assert!(json.contains("\\u001F"), "got: {}", json);
    }

    #[test]
    fn tab_newline_carriage_use_short_escapes() {
        let node = XmlNode::Text("\t\n\r".to_string());
        let json = node.to_json().unwrap();
        assert!(json.contains("\\t"));
        assert!(json.contains("\\n"));
        assert!(json.contains("\\r"));
        assert!(!json.contains("\\u0009"));
        assert!(!json.contains("\\u000A"));
        assert!(!json.contains("\\u000D"));
    }

    #[test]
    fn exceeds_max_depth_returns_error() {
        let mut node = XmlNode::Element {
            name: "a".to_string(),
            attributes: vec![],
            children: vec![],
        };
        for _ in 0..(MAX_DEPTH + 1) {
            node = XmlNode::Element {
                name: "a".to_string(),
                attributes: vec![],
                children: vec![node],
            };
        }
        assert!(node.to_json().is_err());
    }
}
