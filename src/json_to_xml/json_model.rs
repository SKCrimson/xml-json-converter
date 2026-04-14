#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    BraceOpen,
    BraceClose,
    BracketOpen,
    BracketClose,
    Colon,
    Comma,
    StringVal(&'a str), // Store a reference, not a copy
    Number(f64),
    BoolVal(bool),
    Null,
}

#[derive(Debug)]
pub enum JsonNode<'a> {
    Object(Vec<(&'a str, JsonNode<'a>)>),
    Array(Vec<JsonNode<'a>>),
    StringVal(&'a str),
    Number(f64),
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
                    "{}<{}>{} \n{}</{}>",
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
                    "{}<{}>{} \n{}</{}>",
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
