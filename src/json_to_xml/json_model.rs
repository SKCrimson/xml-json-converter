use std::collections::HashMap;

pub(super) const MAX_DEPTH: usize = 512;

#[derive(Debug, PartialEq)]
pub enum Token {
    BraceOpen,
    BraceClose,
    BracketOpen,
    BracketClose,
    Colon,
    Comma,
    StringVal(String),
    Number(String),
    BoolVal(bool),
    Null,
}

#[derive(Debug)]
pub enum JsonNode {
    Object(Vec<(String, JsonNode)>),
    Array(Vec<JsonNode>),
    StringVal(String),
    Number(String),
    BoolVal(bool),
    Null,
}

impl JsonNode {
    pub fn to_xml(&self) -> Result<String, String> {
        self.to_xml_impl("root", 0, None)
    }

    pub fn to_pretty_xml(&self, indent_size: usize) -> Result<String, String> {
        self.to_xml_impl("root", 0, Some(indent_size))
    }

    fn to_xml_impl(&self, label: &str, depth: usize, indent: Option<usize>) -> Result<String, String> {
        if depth >= MAX_DEPTH {
            return Err("Nesting depth limit exceeded".to_string());
        }
        let safe_label = sanitize_tag_name(label);
        let pad = indent.map(|sz| " ".repeat(depth * sz)).unwrap_or_default();

        match self {
            JsonNode::Object(pairs) => {
                let mut seen_tags: HashMap<String, usize> = HashMap::new();
                let mut inner = String::new();
                for (key, val) in pairs {
                    let base = sanitize_tag_name(key);
                    let n = seen_tags.entry(base.clone()).or_insert(0);
                    *n += 1;
                    let tag = if *n == 1 { base } else { format!("{}_{}", base, n) };
                    if let JsonNode::Array(elements) = val {
                        for el in elements {
                            if indent.is_some() { inner.push('\n'); }
                            inner.push_str(&el.to_xml_impl(&tag, depth + 1, indent)?);
                        }
                    } else {
                        if indent.is_some() { inner.push('\n'); }
                        inner.push_str(&val.to_xml_impl(&tag, depth + 1, indent)?);
                    }
                }
                let close_pad = if indent.is_some() { format!("\n{}", pad) } else { String::new() };
                Ok(format!("{}<{}>{}{}</{}>", pad, safe_label, inner, close_pad, safe_label))
            }
            JsonNode::Array(elements) => {
                let mut inner = String::new();
                for el in elements {
                    if indent.is_some() { inner.push('\n'); }
                    inner.push_str(&el.to_xml_impl("item", depth + 1, indent)?);
                }
                let close_pad = if indent.is_some() { format!("\n{}", pad) } else { String::new() };
                Ok(format!("{}<{}>{}{}</{}>", pad, safe_label, inner, close_pad, safe_label))
            }
            JsonNode::StringVal(s) => Ok(format!(
                "{}<{}>{}</{}>", pad, safe_label, escape_xml_text(s), safe_label
            )),
            JsonNode::Number(n) => Ok(format!("{}<{}>{}</{}>", pad, safe_label, n, safe_label)),
            JsonNode::BoolVal(b) => Ok(format!("{}<{}>{}</{}>", pad, safe_label, b, safe_label)),
            JsonNode::Null => Ok(format!("{}<{} />", pad, safe_label)),
        }
    }

}

fn escape_xml_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    // Buffer consecutive ']' so that ]]> (forbidden in XML 1.0 §2.4) can be
    // detected and its '>' escaped without a second pass over the output.
    let mut brackets = 0usize;

    for c in text.chars() {
        match c {
            ']' => brackets += 1,
            '>' if brackets >= 2 => {
                for _ in 0..brackets - 2 { out.push(']'); }
                out.push_str("]]&gt;");
                brackets = 0;
            }
            _ => {
                for _ in 0..brackets { out.push(']'); }
                brackets = 0;
                match c {
                    '&' => out.push_str("&amp;"),
                    '<' => out.push_str("&lt;"),
                    c   => out.push(c),
                }
            }
        }
    }
    for _ in 0..brackets { out.push(']'); }
    out
}

fn sanitize_tag_name(name: &str) -> String {
    if name.is_empty() {
        return "_".to_string();
    }

    let is_valid = |c: char| c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == ':';

    let mut sanitized = if name.chars().all(is_valid) {
        name.to_string()
    } else {
        name.replace(|c: char| !is_valid(c), "_")
    };

    // An XML tag cannot start with a digit, '-', or '.'
    if let Some(first) = sanitized.chars().next() {
        if first.is_numeric() || first == '-' || first == '.' {
            sanitized.insert(0, '_');
        }
    }
    sanitized
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
        let node = JsonNode::Number("42".to_string());
        let xml = node.to_xml().unwrap();
        assert!(xml.contains("<root>") && xml.contains("42") && xml.contains("</root>"));
    }

    #[test]
    fn large_integer_preserved_exactly() {
        let node = JsonNode::Number("9007199254740993".to_string());
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
    fn greater_than_is_literal_in_text() {
        let node = JsonNode::StringVal("a > b".to_string());
        assert!(node.to_xml().unwrap().contains("a > b"));
    }

    #[test]
    fn cdata_end_sequence_in_text_is_escaped() {
        // "]]>" is forbidden in XML text content (XML 1.0 §2.4)
        let node = JsonNode::StringVal("a]]>b".to_string());
        let xml = node.to_xml().unwrap();
        assert!(!xml.contains("]]>"), "]]> must not appear raw in XML: {}", xml);
        assert!(xml.contains("]]&gt;"), "got: {}", xml);
    }

    #[test]
    fn multiple_cdata_end_sequences_all_escaped() {
        let node = JsonNode::StringVal("]]>x]]>".to_string());
        let xml = node.to_xml().unwrap();
        assert!(!xml.contains("]]>"), "]]> must not appear raw in XML: {}", xml);
        assert_eq!(xml.matches("]]&gt;").count(), 2, "got: {}", xml);
    }

    #[test]
    fn standalone_gt_not_escaped_by_cdata_fix() {
        // The ]]> fix must not affect lone '>'
        let node = JsonNode::StringVal("]>".to_string());
        let xml = node.to_xml().unwrap();
        assert!(xml.contains("]>"), "got: {}", xml);
        assert!(!xml.contains("&gt;"), "got: {}", xml);
    }

    #[test]
    fn quotes_are_literal_in_text() {
        let node = JsonNode::StringVal("say \"hi\"".to_string());
        assert!(node.to_xml().unwrap().contains("say \"hi\""));
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

    #[test]
    fn tag_name_with_dot_in_middle_is_preserved() {
        let node = JsonNode::Object(vec![("my.class".to_string(), JsonNode::StringVal("v".to_string()))]);
        let xml = node.to_xml().unwrap();
        assert!(xml.contains("<my.class>"), "got: {}", xml);
    }

    #[test]
    fn tag_name_starting_with_dot_gets_underscore_prefix() {
        let node = JsonNode::Object(vec![(".hidden".to_string(), JsonNode::StringVal("v".to_string()))]);
        let xml = node.to_xml().unwrap();
        assert!(xml.contains("<_.hidden>"), "got: {}", xml);
    }

    #[test]
    fn tag_name_with_namespace_prefix_preserved() {
        let node = JsonNode::Object(vec![("xsi:type".to_string(), JsonNode::StringVal("xs:string".to_string()))]);
        let xml = node.to_xml().unwrap();
        assert!(xml.contains("<xsi:type>"), "got: {}", xml);
    }

    #[test]
    fn at_prefix_from_xml_attr_becomes_underscore_colon_preserved() {
        // "@xsi:type" key (from XML→JSON round-trip): '@' → '_', ':' preserved
        let node = JsonNode::Object(vec![("@xsi:type".to_string(), JsonNode::StringVal("xs:string".to_string()))]);
        let xml = node.to_xml().unwrap();
        assert!(xml.contains("<_xsi:type>"), "got: {}", xml);
        assert!(!xml.contains("<_xsi_type>"), "colon was incorrectly replaced: {}", xml);
    }

    #[test]
    fn colliding_keys_second_gets_numeric_suffix() {
        // "_id" and "@id" both sanitize to "_id"; second occurrence gets "_id_2"
        let node = JsonNode::Object(vec![
            ("_id".to_string(), JsonNode::StringVal("first".to_string())),
            ("@id".to_string(), JsonNode::StringVal("second".to_string())),
        ]);
        let xml = node.to_xml().unwrap();
        assert!(xml.contains("<_id>first</_id>"), "got: {}", xml);
        assert!(xml.contains("<_id_2>second</_id_2>"), "got: {}", xml);
    }

    #[test]
    fn triple_collision_gets_sequential_suffixes() {
        // "@k", "#k", and "_k" all sanitize to "_k"
        let node = JsonNode::Object(vec![
            ("_k".to_string(), JsonNode::StringVal("a".to_string())),
            ("@k".to_string(), JsonNode::StringVal("b".to_string())),
            ("#k".to_string(), JsonNode::StringVal("c".to_string())),
        ]);
        let xml = node.to_xml().unwrap();
        assert!(xml.contains("<_k>a</_k>"), "got: {}", xml);
        assert!(xml.contains("<_k_2>b</_k_2>"), "got: {}", xml);
        assert!(xml.contains("<_k_3>c</_k_3>"), "got: {}", xml);
    }

    #[test]
    fn digit_prefix_collision_resolved() {
        // "1key" sanitizes to "_1key"; if "_1key" is also present, second gets "_1key_2"
        let node = JsonNode::Object(vec![
            ("_1key".to_string(), JsonNode::StringVal("original".to_string())),
            ("1key".to_string(), JsonNode::StringVal("sanitized".to_string())),
        ]);
        let xml = node.to_xml().unwrap();
        assert!(xml.contains("<_1key>original</_1key>"), "got: {}", xml);
        assert!(xml.contains("<_1key_2>sanitized</_1key_2>"), "got: {}", xml);
    }

    #[test]
    fn unique_keys_unaffected_by_collision_fix() {
        // Non-colliding keys must still produce their original sanitized names
        let node = JsonNode::Object(vec![
            ("name".to_string(), JsonNode::StringVal("Alice".to_string())),
            ("@role".to_string(), JsonNode::StringVal("admin".to_string())),
        ]);
        let xml = node.to_xml().unwrap();
        assert!(xml.contains("<name>Alice</name>"), "got: {}", xml);
        assert!(xml.contains("<_role>admin</_role>"), "got: {}", xml);
        assert!(!xml.contains("_2"), "unexpected suffix: {}", xml);
    }
}
