use crate::xml_to_json::xml_model::XmlNode;

mod xml_lexer;
mod xml_model;
mod xml_parser;

pub fn convert(xml: &str, pretty: bool) -> Result<String, String> {
    let mut lexer = xml_lexer::Lexer::new(xml);
    let tokens = lexer.tokenize()?;
    // println!("Tokens: {:?}", tokens);

    let root_node = xml_parser::Parser::new(tokens).parse()?;

    let json_output = if let XmlNode::Element { name, .. } = &root_node {
        if pretty {
            format!("{{\n    \"{}\": {}\n}}", name, root_node.to_pretty_json_at(1, 4)?)
        } else {
            format!("{{ \"{}\": {} }}", name, root_node.to_json()?)
        }
    } else if pretty {
        root_node.to_pretty_json(4)?
    } else {
        root_node.to_json()?
    };

    Ok(json_output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_text_element() {
        let result = convert("<root>hello</root>", false).unwrap();
        assert!(result.contains("\"root\""));
        assert!(result.contains("\"hello\""));
    }

    #[test]
    fn element_with_attribute() {
        let result = convert("<root id=\"42\"/>", false).unwrap();
        assert!(result.contains("\"@id\""));
        assert!(result.contains("\"42\""));
    }

    #[test]
    fn nested_elements() {
        let xml = "<root><child><value>x</value></child></root>";
        let result = convert(xml, false).unwrap();
        assert!(result.contains("\"child\""));
        assert!(result.contains("\"value\""));
    }

    #[test]
    fn repeated_elements_produce_array() {
        let xml = "<root><item>a</item><item>b</item></root>";
        let result = convert(xml, false).unwrap();
        assert!(result.contains('['));
        assert!(result.contains("\"item\""));
    }

    #[test]
    fn self_closing_child_element() {
        let result = convert("<root><br/></root>", false).unwrap();
        assert!(result.contains("\"br\""));
    }

    #[test]
    fn xml_with_declaration() {
        let xml = "<?xml version=\"1.0\"?><root><name>test</name></root>";
        let result = convert(xml, false).unwrap();
        assert!(result.contains("\"name\""));
        assert!(result.contains("\"test\""));
    }

    #[test]
    fn non_xml_processing_instruction_is_skipped() {
        let xml = "<?custom-pi some data?><root><val>x</val></root>";
        let result = convert(xml, false).unwrap();
        assert!(result.contains("\"val\""));
        assert!(!result.contains("custom-pi"));
    }

    #[test]
    fn unclosed_processing_instruction_returns_error() {
        assert!(convert("<?xml version=\"1.0\"<root/>", false).is_err());
    }

    #[test]
    fn pretty_mode_contains_newlines() {
        let result = convert("<root><child>text</child></root>", true).unwrap();
        assert!(result.contains('\n'));
    }

    #[test]
    fn pretty_mode_root_indentation() {
        let result = convert("<root><child>text</child></root>", true).unwrap();
        assert!(result.starts_with("{\n    \"root\":"));
        assert!(result.contains("\n        \"child\":"));
        assert!(result.ends_with("\n}"));
    }

    #[test]
    fn invalid_xml_returns_error() {
        assert!(convert("<root><unclosed>", false).is_err());
    }

    #[test]
    fn truncated_closing_tag_returns_error() {
        let result = convert("<root></root", false);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("Unexpected EOF in closing tag"),
            "expected specific EOF error, not a generic parser error"
        );
    }

    #[test]
    fn mismatched_closing_tag_returns_error() {
        let result = convert("<root><a></b></a></root>", false);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("</a>"), "expected tag name in error, got: {}", msg);
        assert!(msg.contains("</b>"), "expected tag name in error, got: {}", msg);
    }

    #[test]
    fn swapped_closing_tags_return_error() {
        assert!(convert("<root><a><b></a></b></root>", false).is_err());
    }

    #[test]
    fn unclosed_comment_returns_error() {
        assert!(convert("<root><!-- no close</root>", false).is_err());
    }

    #[test]
    fn closed_comment_is_valid() {
        let result = convert("<root><!-- comment --><child>x</child></root>", false);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("\"child\""));
    }

    #[test]
    fn special_chars_in_text_are_escaped_in_json() {
        let xml = "<root>say &quot;hi&quot;</root>";
        let result = convert(xml, false).unwrap();
        // &quot; decoded to " then JSON-escaped to \"
        assert!(result.contains(r#"say \"hi\""#), "got: {}", result);
    }

    #[test]
    fn xml_entity_amp_in_text_decoded() {
        let result = convert("<root>a &amp; b</root>", false).unwrap();
        assert!(result.contains("a & b"), "got: {}", result);
    }

    #[test]
    fn xml_entity_lt_gt_in_text_decoded() {
        let result = convert("<root>1 &lt; 2 &gt; 0</root>", false).unwrap();
        assert!(result.contains("1 < 2 > 0"), "got: {}", result);
    }

    #[test]
    fn xml_entity_apos_in_text_decoded() {
        let result = convert("<root>it&apos;s</root>", false).unwrap();
        assert!(result.contains("it's"), "got: {}", result);
    }

    #[test]
    fn xml_numeric_decimal_entity_in_text_decoded() {
        // &#65; = 'A'
        let result = convert("<root>&#65;</root>", false).unwrap();
        assert!(result.contains("\"A\""), "got: {}", result);
    }

    #[test]
    fn xml_numeric_hex_entity_in_text_decoded() {
        // &#x41; = 'A'
        let result = convert("<root>&#x41;</root>", false).unwrap();
        assert!(result.contains("\"A\""), "got: {}", result);
    }

    #[test]
    fn xml_entity_in_attribute_value_decoded() {
        let xml = r#"<root label="say &quot;hi&quot;"/>"#;
        let result = convert(xml, false).unwrap();
        assert!(result.contains(r#"say \"hi\""#), "got: {}", result);
    }

    #[test]
    fn xml_with_doctype_is_converted() {
        let xml = "<!DOCTYPE root><root><name>test</name></root>";
        let result = convert(xml, false).unwrap();
        assert!(result.contains("\"name\""));
        assert!(result.contains("\"test\""));
    }

    #[test]
    fn multibyte_utf8_text_content() {
        let xml = "<root>Привет мир</root>";
        let result = convert(xml, false).unwrap();
        assert!(result.contains("Привет мир"));
    }

    #[test]
    fn multibyte_utf8_attribute_value() {
        let xml = "<root lang=\"日本語\"/>";
        let result = convert(xml, false).unwrap();
        assert!(result.contains("日本語"));
    }

    #[test]
    fn namespaced_attribute_preserved_with_prefix() {
        let xml = r#"<root xsi:type="xs:string" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"/>"#;
        let result = convert(xml, false).unwrap();
        assert!(result.contains("\"@xsi:type\""), "got: {}", result);
        assert!(result.contains("\"@xmlns:xsi\""), "got: {}", result);
    }

    #[test]
    fn namespaced_attribute_value_preserved() {
        let xml = r#"<root xsi:type="xs:integer"/>"#;
        let result = convert(xml, false).unwrap();
        assert!(result.contains("\"xs:integer\""), "got: {}", result);
    }

    #[test]
    fn multibyte_utf8_in_child_text() {
        let xml = "<root><item>Ñoño</item><item>中文</item></root>";
        let result = convert(xml, false).unwrap();
        assert!(result.contains("Ñoño"));
        assert!(result.contains("中文"));
    }
}
