#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    BraceOpen,
    BraceClose,
    BracketOpen,
    BracketClose,
    Colon,
    Comma,
    StringVal(&'a str), // Store a reference, not a copy
    Number(&'a str),
    BoolVal(bool),
    Null,
}

#[derive(Debug)]
pub enum JsonNode<'a> {
    Object(Vec<(&'a str, JsonNode<'a>)>),
    Array(Vec<JsonNode<'a>>),
    StringVal(&'a str),
    Number(&'a str),
    BoolVal(bool),
    Null,
}

impl<'a> JsonNode<'a> {
    /// Public method to convert to an XML string.
    /// Uses "root" as the root element name.
    pub fn to_xml(&self) -> String {
        self.to_xml_internal("root")
    }

    /// Public method for pretty output
    pub fn to_pretty_xml(&self, indent_size: usize) -> String {
        self.to_xml_recursive("root", 0, indent_size)
    }

    /// Internal recursive method
    fn to_xml_internal(&self, label: &str) -> String {
        // Sanitize the tag name (XML does not allow spaces and special chars in names)
        let safe_label = self.sanitize_tag_name(label);

        match self {
            JsonNode::Object(pairs) => {
                let mut inner = String::new();
                for (key, val) in pairs {
                    // Object keys become names of nested tags
                    inner.push_str(&val.to_xml_internal(key));
                }
                format!("<{}>{}</{}>", safe_label, inner, safe_label)
            }
            JsonNode::Array(elements) => {
                let mut inner = String::new();
                for el in elements {
                    // Array elements use the parent name or "item"
                    // In XML, it is common to use either a singular name or <item>
                    inner.push_str(&el.to_xml_internal("item"));
                }
                format!("<{}>{}</{}>", safe_label, inner, safe_label)
            }
            JsonNode::StringVal(s) => {
                format!(
                    "<{}>{}</{}>",
                    safe_label,
                    self.escape_xml_text(s),
                    safe_label
                )
            }
            JsonNode::Number(n) => {
                format!("<{}>{}</{}>", safe_label, n, safe_label)
            }
            JsonNode::BoolVal(b) => {
                format!("<{}>{}</{}>", safe_label, b, safe_label)
            }
            JsonNode::Null => {
                format!("<{} />", safe_label) // Self-closing tag for null
            }
        }
    }

    fn to_xml_recursive(&self, label: &str, depth: usize, indent_size: usize) -> String {
        let safe_label = self.sanitize_tag_name(label);
        let indent = " ".repeat(depth * indent_size);

        match self {
            JsonNode::Object(pairs) => {
                let mut inner = String::new();
                for (key, val) in pairs {
                    inner.push_str("\n");
                    inner.push_str(&val.to_xml_recursive(key, depth + 1, indent_size));
                }
                format!(
                    "{}<{}>{}\n{}</{}>",
                    indent, safe_label, inner, indent, safe_label
                )
            }
            JsonNode::Array(elements) => {
                let mut inner = String::new();
                for el in elements {
                    inner.push_str("\n");
                    inner.push_str(&el.to_xml_recursive("item", depth + 1, indent_size));
                }
                format!(
                    "{}<{}>{}\n{}</{}>",
                    indent, safe_label, inner, indent, safe_label
                )
            }
            JsonNode::StringVal(s) => {
                format!(
                    "{}<{}>{}</{}>",
                    indent,
                    safe_label,
                    self.escape_xml_text(s),
                    safe_label
                )
            }
            JsonNode::Number(n) => {
                format!("{}<{}>{}</{}>", indent, safe_label, n, safe_label)
            }
            JsonNode::BoolVal(b) => {
                format!("{}<{}>{}</{}>", indent, safe_label, b, safe_label)
            }
            JsonNode::Null => {
                format!("{}<{} />", indent, safe_label)
            }
        }
    }

    /// Helper function to escape special characters in XML text
    fn escape_xml_text(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    /// Helper function to validate tag names
    fn sanitize_tag_name(&self, name: &str) -> String {
        let mut sanitized =
            name.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_");

        // An XML tag cannot start with a digit
        if let Some(first) = sanitized.chars().next() {
            if first.is_numeric() {
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
        let node = JsonNode::StringVal("hello");
        assert_eq!(node.to_xml(), "<root>hello</root>");
    }

    #[test]
    fn number_to_xml() {
        let node = JsonNode::Number("42");
        let xml = node.to_xml();
        assert!(xml.contains("<root>") && xml.contains("42") && xml.contains("</root>"));
    }

    #[test]
    fn large_integer_preserved_exactly() {
        let node = JsonNode::Number("9007199254740993");
        assert!(node.to_xml().contains("9007199254740993"));
    }

    #[test]
    fn bool_true_to_xml() {
        let node = JsonNode::BoolVal(true);
        assert_eq!(node.to_xml(), "<root>true</root>");
    }

    #[test]
    fn bool_false_to_xml() {
        let node = JsonNode::BoolVal(false);
        assert_eq!(node.to_xml(), "<root>false</root>");
    }

    #[test]
    fn null_to_xml_is_self_closing() {
        let node = JsonNode::Null;
        assert_eq!(node.to_xml(), "<root />");
    }

    #[test]
    fn object_key_becomes_tag() {
        let node = JsonNode::Object(vec![("name", JsonNode::StringVal("Alice"))]);
        let xml = node.to_xml();
        assert!(xml.contains("<name>Alice</name>"));
        assert!(xml.starts_with("<root>"));
        assert!(xml.ends_with("</root>"));
    }

    #[test]
    fn array_elements_use_item_tag() {
        let node = JsonNode::Array(vec![JsonNode::StringVal("x"), JsonNode::StringVal("y")]);
        let xml = node.to_xml();
        assert!(xml.contains("<item>x</item>"));
        assert!(xml.contains("<item>y</item>"));
    }

    #[test]
    fn ampersand_is_escaped() {
        let node = JsonNode::StringVal("a & b");
        assert!(node.to_xml().contains("&amp;"));
    }

    #[test]
    fn less_than_is_escaped() {
        let node = JsonNode::StringVal("a < b");
        assert!(node.to_xml().contains("&lt;"));
    }

    #[test]
    fn greater_than_is_escaped() {
        let node = JsonNode::StringVal("a > b");
        assert!(node.to_xml().contains("&gt;"));
    }

    #[test]
    fn quotes_are_escaped() {
        let node = JsonNode::StringVal("say \"hi\"");
        assert!(node.to_xml().contains("&quot;"));
    }

    #[test]
    fn tag_name_spaces_replaced_with_underscore() {
        let node = JsonNode::Object(vec![("my key", JsonNode::StringVal("v"))]);
        let xml = node.to_xml();
        assert!(xml.contains("<my_key>"));
    }

    #[test]
    fn tag_name_starting_with_digit_gets_underscore_prefix() {
        let node = JsonNode::Object(vec![("1key", JsonNode::StringVal("v"))]);
        let xml = node.to_xml();
        assert!(xml.contains("<_1key>"));
    }

    #[test]
    fn pretty_xml_contains_newlines() {
        let node = JsonNode::Object(vec![("a", JsonNode::StringVal("b"))]);
        assert!(node.to_pretty_xml(4).contains('\n'));
    }

    #[test]
    fn pretty_xml_no_trailing_space_before_newline() {
        let node = JsonNode::Object(vec![("a", JsonNode::StringVal("b"))]);
        assert!(!node.to_pretty_xml(4).contains(" \n"));
    }
}
