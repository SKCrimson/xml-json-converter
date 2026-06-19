const MAX_DEPTH: usize = 512;

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    BraceOpen,
    BraceClose,
    BracketOpen,
    BracketClose,
    Colon,
    Comma,
    StringVal(String),
    Number(&'a str),
    BoolVal(bool),
    Null,
}

#[derive(Debug)]
pub enum JsonNode<'a> {
    Object(Vec<(String, JsonNode<'a>)>),
    Array(Vec<JsonNode<'a>>),
    StringVal(String),
    Number(&'a str),
    BoolVal(bool),
    Null,
}

impl<'a> JsonNode<'a> {
    pub fn to_xml(&self) -> Result<String, String> {
        self.to_xml_internal("root", 0)
    }

    pub fn to_pretty_xml(&self, indent_size: usize) -> Result<String, String> {
        self.to_xml_recursive("root", 0, indent_size)
    }

    fn to_xml_internal(&self, label: &str, depth: usize) -> Result<String, String> {
        if depth >= MAX_DEPTH {
            return Err("Nesting depth limit exceeded".to_string());
        }
        let safe_label = self.sanitize_tag_name(label);

        match self {
            JsonNode::Object(pairs) => {
                let mut inner = String::new();
                for (key, val) in pairs {
                    if let JsonNode::Array(elements) = val {
                        for el in elements {
                            inner.push_str(&el.to_xml_internal(key, depth + 1)?);
                        }
                    } else {
                        inner.push_str(&val.to_xml_internal(key, depth + 1)?);
                    }
                }
                Ok(format!("<{}>{}</{}>", safe_label, inner, safe_label))
            }
            JsonNode::Array(elements) => {
                let mut inner = String::new();
                for el in elements {
                    inner.push_str(&el.to_xml_internal("item", depth + 1)?);
                }
                Ok(format!("<{}>{}</{}>", safe_label, inner, safe_label))
            }
            JsonNode::StringVal(s) => Ok(format!(
                "<{}>{}</{}>",
                safe_label,
                self.escape_xml_text(s),
                safe_label
            )),
            JsonNode::Number(n) => Ok(format!("<{}>{}</{}>", safe_label, n, safe_label)),
            JsonNode::BoolVal(b) => Ok(format!("<{}>{}</{}>", safe_label, b, safe_label)),
            JsonNode::Null => Ok(format!("<{} />", safe_label)),
        }
    }

    fn to_xml_recursive(
        &self,
        label: &str,
        depth: usize,
        indent_size: usize,
    ) -> Result<String, String> {
        if depth >= MAX_DEPTH {
            return Err("Nesting depth limit exceeded".to_string());
        }
        let safe_label = self.sanitize_tag_name(label);
        let indent = " ".repeat(depth * indent_size);

        match self {
            JsonNode::Object(pairs) => {
                let mut inner = String::new();
                for (key, val) in pairs {
                    if let JsonNode::Array(elements) = val {
                        for el in elements {
                            inner.push_str("\n");
                            inner.push_str(&el.to_xml_recursive(key, depth + 1, indent_size)?);
                        }
                    } else {
                        inner.push_str("\n");
                        inner.push_str(&val.to_xml_recursive(key, depth + 1, indent_size)?);
                    }
                }
                Ok(format!(
                    "{}<{}>{}\n{}</{}>",
                    indent, safe_label, inner, indent, safe_label
                ))
            }
            JsonNode::Array(elements) => {
                let mut inner = String::new();
                for el in elements {
                    inner.push_str("\n");
                    inner.push_str(&el.to_xml_recursive("item", depth + 1, indent_size)?);
                }
                Ok(format!(
                    "{}<{}>{}\n{}</{}>",
                    indent, safe_label, inner, indent, safe_label
                ))
            }
            JsonNode::StringVal(s) => Ok(format!(
                "{}<{}>{}</{}>",
                indent,
                safe_label,
                self.escape_xml_text(s),
                safe_label
            )),
            JsonNode::Number(n) => {
                Ok(format!("{}<{}>{}</{}>", indent, safe_label, n, safe_label))
            }
            JsonNode::BoolVal(b) => {
                Ok(format!("{}<{}>{}</{}>", indent, safe_label, b, safe_label))
            }
            JsonNode::Null => Ok(format!("{}<{} />", indent, safe_label)),
        }
    }

    fn escape_xml_text(&self, text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        for c in text.chars() {
            match c {
                '&'  => out.push_str("&amp;"),
                '<'  => out.push_str("&lt;"),
                '>'  => out.push_str("&gt;"),
                '"'  => out.push_str("&quot;"),
                '\'' => out.push_str("&apos;"),
                c    => out.push(c),
            }
        }
        out
    }

    fn sanitize_tag_name(&self, name: &str) -> String {
        if name.is_empty() {
            return "_".to_string();
        }

        let mut sanitized =
            name.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_");

        // An XML tag cannot start with a digit or '-'
        if let Some(first) = sanitized.chars().next() {
            if first.is_numeric() || first == '-' {
                sanitized.insert(0, '_');
            }
        }
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_val_to_xml() {
        let node = JsonNode::StringVal("hello".to_string());
        assert_eq!(node.to_xml().unwrap(), "<root>hello</root>");
    }

    #[test]
    fn number_to_xml() {
        let node = JsonNode::Number("42");
        let xml = node.to_xml().unwrap();
        assert!(xml.contains("<root>") && xml.contains("42") && xml.contains("</root>"));
    }

    #[test]
    fn large_integer_preserved_exactly() {
        let node = JsonNode::Number("9007199254740993");
        assert!(node.to_xml().unwrap().contains("9007199254740993"));
    }

    #[test]
    fn bool_true_to_xml() {
        let node = JsonNode::BoolVal(true);
        assert_eq!(node.to_xml().unwrap(), "<root>true</root>");
    }

    #[test]
    fn bool_false_to_xml() {
        let node = JsonNode::BoolVal(false);
        assert_eq!(node.to_xml().unwrap(), "<root>false</root>");
    }

    #[test]
    fn null_to_xml_is_self_closing() {
        let node = JsonNode::Null;
        assert_eq!(node.to_xml().unwrap(), "<root />");
    }

    #[test]
    fn object_key_becomes_tag() {
        let node = JsonNode::Object(vec![("name".to_string(), JsonNode::StringVal("Alice".to_string()))]);
        let xml = node.to_xml().unwrap();
        assert!(xml.contains("<name>Alice</name>"));
        assert!(xml.starts_with("<root>"));
        assert!(xml.ends_with("</root>"));
    }

    #[test]
    fn array_elements_use_item_tag() {
        let node = JsonNode::Array(vec![
            JsonNode::StringVal("x".to_string()),
            JsonNode::StringVal("y".to_string()),
        ]);
        let xml = node.to_xml().unwrap();
        assert!(xml.contains("<item>x</item>"));
        assert!(xml.contains("<item>y</item>"));
    }

    #[test]
    fn array_value_in_object_produces_repeated_siblings() {
        let node = JsonNode::Object(vec![(
            "tag".to_string(),
            JsonNode::Array(vec![
                JsonNode::StringVal("a".to_string()),
                JsonNode::StringVal("b".to_string()),
            ]),
        )]);
        let xml = node.to_xml().unwrap();
        assert!(xml.contains("<tag>a</tag>"), "got: {}", xml);
        assert!(xml.contains("<tag>b</tag>"), "got: {}", xml);
        assert!(!xml.contains("<item>"), "got: {}", xml);
    }

    #[test]
    fn ampersand_is_escaped() {
        let node = JsonNode::StringVal("a & b".to_string());
        assert!(node.to_xml().unwrap().contains("&amp;"));
    }

    #[test]
    fn less_than_is_escaped() {
        let node = JsonNode::StringVal("a < b".to_string());
        assert!(node.to_xml().unwrap().contains("&lt;"));
    }

    #[test]
    fn greater_than_is_escaped() {
        let node = JsonNode::StringVal("a > b".to_string());
        assert!(node.to_xml().unwrap().contains("&gt;"));
    }

    #[test]
    fn quotes_are_escaped() {
        let node = JsonNode::StringVal("say \"hi\"".to_string());
        assert!(node.to_xml().unwrap().contains("&quot;"));
    }

    #[test]
    fn tag_name_spaces_replaced_with_underscore() {
        let node = JsonNode::Object(vec![("my key".to_string(), JsonNode::StringVal("v".to_string()))]);
        let xml = node.to_xml().unwrap();
        assert!(xml.contains("<my_key>"));
    }

    #[test]
    fn tag_name_starting_with_digit_gets_underscore_prefix() {
        let node = JsonNode::Object(vec![("1key".to_string(), JsonNode::StringVal("v".to_string()))]);
        let xml = node.to_xml().unwrap();
        assert!(xml.contains("<_1key>"));
    }

    #[test]
    fn tag_name_starting_with_dash_gets_underscore_prefix() {
        let node = JsonNode::Object(vec![("-key".to_string(), JsonNode::StringVal("v".to_string()))]);
        let xml = node.to_xml().unwrap();
        assert!(xml.contains("<_-key>"), "got: {}", xml);
    }

    #[test]
    fn tag_name_with_dash_in_middle_is_preserved() {
        let node = JsonNode::Object(vec![("my-key".to_string(), JsonNode::StringVal("v".to_string()))]);
        let xml = node.to_xml().unwrap();
        assert!(xml.contains("<my-key>"), "got: {}", xml);
    }

    #[test]
    fn pretty_xml_contains_newlines() {
        let node = JsonNode::Object(vec![("a".to_string(), JsonNode::StringVal("b".to_string()))]);
        assert!(node.to_pretty_xml(4).unwrap().contains('\n'));
    }

    #[test]
    fn pretty_xml_no_trailing_space_before_newline() {
        let node = JsonNode::Object(vec![("a".to_string(), JsonNode::StringVal("b".to_string()))]);
        assert!(!node.to_pretty_xml(4).unwrap().contains(" \n"));
    }

    #[test]
    fn exceeds_max_depth_returns_error() {
        let mut node = JsonNode::Object(vec![]);
        for _ in 0..(MAX_DEPTH + 1) {
            node = JsonNode::Object(vec![("a".to_string(), node)]);
        }
        assert!(node.to_xml().is_err());
    }

    #[test]
    fn empty_key_becomes_underscore_tag() {
        let node = JsonNode::Object(vec![("".to_string(), JsonNode::StringVal("v".to_string()))]);
        let xml = node.to_xml().unwrap();
        assert!(xml.contains("<_>v</_>"), "got: {}", xml);
    }

    #[test]
    fn empty_key_is_valid_xml_structure() {
        let node = JsonNode::Object(vec![("".to_string(), JsonNode::Null)]);
        let xml = node.to_xml().unwrap();
        assert!(!xml.contains("<>"), "empty tag name leaked: {}", xml);
    }
}
