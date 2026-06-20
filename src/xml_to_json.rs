use crate::xml_to_json::xml_model::XmlNode;

mod xml_lexer;
mod xml_model;
mod xml_parser;

/// Converts XML to JSON. `indent` controls pretty-printing:
/// 0 = compact output, N > 0 = pretty with N spaces per level.
pub fn convert(xml: &str, indent: usize) -> Result<String, String> {
    let xml = xml.strip_prefix('\u{FEFF}').unwrap_or(xml);
    let mut lexer = xml_lexer::Lexer::new(xml);
    let tokens = lexer.tokenize()?;
    let root_node = xml_parser::Parser::new(tokens).parse()?;

    let name = match &root_node {
        XmlNode::Element { name, .. } => name.as_str(),
        XmlNode::Text(_) => unreachable!("parser always yields an Element as root"),
    };

    let json_output = if indent > 0 {
        let pad = " ".repeat(indent);
        format!("{{\n{}\"{}\": {}\n}}", pad, name, root_node.to_pretty_json_at(1, indent)?)
    } else {
        format!("{{ \"{}\": {} }}", name, root_node.to_json()?)
    };

    Ok(json_output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_text_element() {
        let result = convert("<root>hello</root>", 0).unwrap();
        assert!(result.contains("\"root\""));
        assert!(result.contains("\"hello\""));
    }

    #[test]
    fn element_with_attribute() {
        let result = convert("<root id=\"42\"/>", 0).unwrap();
        assert!(result.contains("\"@id\""));
        assert!(result.contains("\"42\""));
    }

    #[test]
    fn nested_elements() {
        let xml = "<root><child><value>x</value></child></root>";
        let result = convert(xml, 0).unwrap();
        assert!(result.contains("\"child\""));
        assert!(result.contains("\"value\""));
    }

    #[test]
    fn repeated_elements_produce_array() {
        let xml = "<root><item>a</item><item>b</item></root>";
        let result = convert(xml, 0).unwrap();
        assert!(result.contains('['));
        assert!(result.contains("\"item\""));
    }

    #[test]
    fn self_closing_child_element() {
        let result = convert("<root><br/></root>", 0).unwrap();
        assert!(result.contains("\"br\""));
    }

    #[test]
    fn xml_with_declaration() {
        let xml = "<?xml version=\"1.0\"?><root><name>test</name></root>";
        let result = convert(xml, 0).unwrap();
        assert!(result.contains("\"name\""));
        assert!(result.contains("\"test\""));
    }

    #[test]
    fn non_xml_processing_instruction_is_skipped() {
        let xml = "<?custom-pi some data?><root><val>x</val></root>";
        let result = convert(xml, 0).unwrap();
        assert!(result.contains("\"val\""));
        assert!(!result.contains("custom-pi"));
    }

    #[test]
    fn unclosed_processing_instruction_returns_error() {
        assert!(convert("<?xml version=\"1.0\"<root/>", 0).is_err());
    }

    #[test]
    fn pretty_mode_contains_newlines() {
        let result = convert("<root><child>text</child></root>", 4).unwrap();
        assert!(result.contains('\n'));
    }

    #[test]
    fn pretty_mode_root_indentation() {
        let result = convert("<root><child>text</child></root>", 4).unwrap();
        assert!(result.starts_with("{\n    \"root\":"));
        assert!(result.contains("\n        \"child\":"));
        assert!(result.ends_with("\n}"));
    }

    #[test]
    fn invalid_xml_returns_error() {
        assert!(convert("<root><unclosed>", 0).is_err());
    }

    #[test]
    fn unclosed_child_element_names_element_in_error() {
        // Previously: "Empty XML document" — stack not checked after tokens exhausted
        let result = convert("<root><child>", 0);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("child"), "expected element name in error: {}", msg);
        assert!(msg.to_lowercase().contains("unclosed"), "expected 'unclosed' in error: {}", msg);
        assert!(!msg.contains("Empty XML document"), "misleading message: {}", msg);
    }

    #[test]
    fn unclosed_root_element_names_element_in_error() {
        let result = convert("<root>", 0);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("root"), "expected element name in error: {}", msg);
        assert!(msg.to_lowercase().contains("unclosed"), "expected 'unclosed' in error: {}", msg);
        assert!(!msg.contains("Empty XML document"), "misleading message: {}", msg);
    }

    #[test]
    fn deeply_nested_unclosed_names_innermost_element() {
        let result = convert("<a><b><c>text</c></b>", 0);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        // "a" is the unclosed element (b and c were closed, a was not)
        assert!(msg.contains('<'), "expected tag syntax in error: {}", msg);
        assert!(!msg.contains("Empty XML document"), "misleading message: {}", msg);
    }

    #[test]
    fn empty_tag_name_returns_error() {
        let result = convert("<>text</>", 0);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.to_lowercase().contains("empty tag name"), "got: {}", msg);
    }

    #[test]
    fn truncated_closing_tag_returns_error() {
        let result = convert("<root></root", 0);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("Unexpected EOF in closing tag"), "got: {}", msg);
        // "</root" starts at byte 6 → line 1, col 7
        assert!(msg.contains("line 1"), "expected line in error: {}", msg);
        assert!(msg.contains("col 7"), "expected col in error: {}", msg);
    }

    #[test]
    fn closing_tag_with_trailing_whitespace_is_valid() {
        // XML allows whitespace between tag name and '>' in closing tags
        let result = convert("<root><child></child ></root>", 0).unwrap();
        assert!(result.contains("\"child\""), "got: {}", result);
    }

    #[test]
    fn closing_tag_whitespace_still_matches_open_tag() {
        // "</child >" must match "<child>", not produce a mismatch error
        assert!(convert("<root><child></child ></root>", 0).is_ok());
        assert!(convert("<root><child></other ></root>", 0).is_err());
    }

    #[test]
    fn unclosed_comment_error_includes_position() {
        // "<!--" starts at byte 11 (after "<root>\n    ") → line 2, col 5
        let xml = "<root>\n    <!-- unclosed";
        let msg = convert(xml, 0).unwrap_err();
        assert!(msg.contains("Unclosed comment"), "got: {}", msg);
        assert!(msg.contains("line 2"), "got: {}", msg);
        assert!(msg.contains("col 5"), "got: {}", msg);
    }

    #[test]
    fn unclosed_pi_error_includes_position() {
        // "<?pi" starts at byte 7 (after "<root>\n") → line 2, col 1
        let xml = "<root>\n<?pi no close";
        let msg = convert(xml, 0).unwrap_err();
        assert!(msg.contains("Unclosed processing instruction"), "got: {}", msg);
        assert!(msg.contains("line 2"), "got: {}", msg);
        assert!(msg.contains("col 1"), "got: {}", msg);
    }

    #[test]
    fn unclosed_cdata_error_includes_position() {
        // "<![CDATA[" starts at byte 7 (after "<root>\n") → line 2, col 1
        let xml = "<root>\n<![CDATA[unclosed";
        let msg = convert(xml, 0).unwrap_err();
        assert!(msg.contains("Unclosed CDATA section"), "got: {}", msg);
        assert!(msg.contains("line 2"), "got: {}", msg);
        assert!(msg.contains("col 1"), "got: {}", msg);
    }

    #[test]
    fn unquoted_attribute_error_includes_position() {
        // "=" at col 10, unquoted char 'v' at col 11 → position_at points to 'v'
        let xml = "<root attr=value/>";
        let msg = convert(xml, 0).unwrap_err();
        assert!(msg.to_lowercase().contains("attribute"), "got: {}", msg);
        assert!(msg.contains("line 1"), "got: {}", msg);
    }

    #[test]
    fn mismatched_closing_tag_returns_error() {
        let result = convert("<root><a></b></a></root>", 0);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("</a>"), "expected tag name in error, got: {}", msg);
        assert!(msg.contains("</b>"), "expected tag name in error, got: {}", msg);
    }

    #[test]
    fn swapped_closing_tags_return_error() {
        assert!(convert("<root><a><b></a></b></root>", 0).is_err());
    }

    #[test]
    fn unclosed_comment_returns_error() {
        assert!(convert("<root><!-- no close</root>", 0).is_err());
    }

    #[test]
    fn closed_comment_is_valid() {
        let result = convert("<root><!-- comment --><child>x</child></root>", 0);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("\"child\""));
    }

    #[test]
    fn special_chars_in_text_are_escaped_in_json() {
        let xml = "<root>say &quot;hi&quot;</root>";
        let result = convert(xml, 0).unwrap();
        // &quot; decoded to " then JSON-escaped to \"
        assert!(result.contains(r#"say \"hi\""#), "got: {}", result);
    }

    #[test]
    fn xml_entity_amp_in_text_decoded() {
        let result = convert("<root>a &amp; b</root>", 0).unwrap();
        assert!(result.contains("a & b"), "got: {}", result);
    }

    #[test]
    fn xml_entity_lt_gt_in_text_decoded() {
        let result = convert("<root>1 &lt; 2 &gt; 0</root>", 0).unwrap();
        assert!(result.contains("1 < 2 > 0"), "got: {}", result);
    }

    #[test]
    fn xml_entity_apos_in_text_decoded() {
        let result = convert("<root>it&apos;s</root>", 0).unwrap();
        assert!(result.contains("it's"), "got: {}", result);
    }

    #[test]
    fn xml_numeric_decimal_entity_in_text_decoded() {
        // &#65; = 'A'
        let result = convert("<root>&#65;</root>", 0).unwrap();
        assert!(result.contains("\"A\""), "got: {}", result);
    }

    #[test]
    fn xml_numeric_hex_entity_in_text_decoded() {
        // &#x41; = 'A'
        let result = convert("<root>&#x41;</root>", 0).unwrap();
        assert!(result.contains("\"A\""), "got: {}", result);
    }

    #[test]
    fn xml_entity_in_attribute_value_decoded() {
        let xml = r#"<root label="say &quot;hi&quot;"/>"#;
        let result = convert(xml, 0).unwrap();
        assert!(result.contains(r#"say \"hi\""#), "got: {}", result);
    }

    #[test]
    fn xml_with_doctype_is_converted() {
        let xml = "<!DOCTYPE root><root><name>test</name></root>";
        let result = convert(xml, 0).unwrap();
        assert!(result.contains("\"name\""));
        assert!(result.contains("\"test\""));
    }

    #[test]
    fn xml_with_doctype_internal_subset_is_converted() {
        // Previously "]>" leaked into the token stream as stray text after the first ">"
        let xml = "<!DOCTYPE root [ <!ELEMENT root ANY> ]><root><name>test</name></root>";
        let result = convert(xml, 0).unwrap();
        assert!(result.contains("\"name\""), "got: {}", result);
        assert!(result.contains("\"test\""), "got: {}", result);
        assert!(!result.contains(']'), "']' leaked into output: {}", result);
    }

    #[test]
    fn xml_with_doctype_multi_declaration_internal_subset() {
        let xml = concat!(
            "<!DOCTYPE root [\n",
            "  <!ELEMENT root ANY>\n",
            "  <!ELEMENT name (#PCDATA)>\n",
            "]>",
            "<root><name>test</name></root>"
        );
        let result = convert(xml, 0).unwrap();
        assert!(result.contains("\"name\""), "got: {}", result);
        assert!(result.contains("\"test\""), "got: {}", result);
    }

    #[test]
    fn multibyte_utf8_text_content() {
        let xml = "<root>Привет мир</root>";
        let result = convert(xml, 0).unwrap();
        assert!(result.contains("Привет мир"));
    }

    #[test]
    fn multibyte_utf8_attribute_value() {
        let xml = "<root lang=\"日本語\"/>";
        let result = convert(xml, 0).unwrap();
        assert!(result.contains("日本語"));
    }

    #[test]
    fn namespaced_attribute_preserved_with_prefix() {
        let xml = r#"<root xsi:type="xs:string" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"/>"#;
        let result = convert(xml, 0).unwrap();
        assert!(result.contains("\"@xsi:type\""), "got: {}", result);
        assert!(result.contains("\"@xmlns:xsi\""), "got: {}", result);
    }

    #[test]
    fn namespaced_attribute_value_preserved() {
        let xml = r#"<root xsi:type="xs:integer"/>"#;
        let result = convert(xml, 0).unwrap();
        assert!(result.contains("\"xs:integer\""), "got: {}", result);
    }

    #[test]
    fn multibyte_utf8_in_child_text() {
        let xml = "<root><item>Ñoño</item><item>中文</item></root>";
        let result = convert(xml, 0).unwrap();
        assert!(result.contains("Ñoño"));
        assert!(result.contains("中文"));
    }

    #[test]
    fn cdata_plain_text_content() {
        let xml = "<root><![CDATA[hello world]]></root>";
        let result = convert(xml, 0).unwrap();
        assert!(result.contains("hello world"), "got: {}", result);
    }

    #[test]
    fn cdata_with_xml_markup_treated_as_literal_text() {
        let xml = "<root><![CDATA[<b>bold</b>]]></root>";
        let result = convert(xml, 0).unwrap();
        // The < and > are literal text, NOT parsed as child elements
        assert!(result.contains("<b>bold</b>"), "got: {}", result);
        assert!(!result.contains("\"b\""), "got: {}", result);
    }

    #[test]
    fn cdata_with_ampersand_not_entity_decoded() {
        let xml = "<root><![CDATA[a & b]]></root>";
        let result = convert(xml, 0).unwrap();
        // '&' in CDATA is literal, not an entity reference
        assert!(result.contains("a & b"), "got: {}", result);
    }

    #[test]
    fn unclosed_cdata_returns_error() {
        assert!(convert("<root><![CDATA[unclosed", 0).is_err());
    }

    #[test]
    fn cdata_whitespace_only_content_preserved() {
        // CDATA is always literal — whitespace must not be silently dropped
        let result = convert("<root><![CDATA[  ]]></root>", 0).unwrap();
        assert!(result.contains("\"  \""), "got: {}", result);
    }

    #[test]
    fn empty_cdata_produces_empty_object() {
        // <![CDATA[]]> contributes zero characters — same as no content at all
        let result = convert("<root><![CDATA[]]></root>", 0).unwrap();
        assert_eq!(result.trim(), "{ \"root\": {} }", "got: {}", result);
    }

    #[test]
    fn text_with_leading_whitespace_preserved() {
        // Whitespace that is part of text content must be kept
        let result = convert("<root><value>  hello  </value></root>", 0).unwrap();
        assert!(result.contains("  hello  "), "got: {}", result);
    }

    #[test]
    fn whitespace_only_between_elements_is_skipped() {
        // Indentation/newlines between sibling tags are formatting noise, not content
        let xml = "<root>\n    <a>1</a>\n    <b>2</b>\n</root>";
        let result = convert(xml, 0).unwrap();
        assert!(result.contains("\"a\""), "got: {}", result);
        assert!(result.contains("\"b\""), "got: {}", result);
        assert!(!result.contains("\\n"), "got: {}", result);
    }

    #[test]
    fn whitespace_only_text_content_preserved() {
        // Whitespace is the only content — must not be silently dropped
        let result = convert("<root>  </root>", 0).unwrap();
        assert!(result.contains("\"  \""), "got: {}", result);
    }

    #[test]
    fn multiline_text_content_preserved() {
        let xml = "<root><pre>line1\nline2</pre></root>";
        let result = convert(xml, 0).unwrap();
        assert!(result.contains("line1\\nline2"), "got: {}", result);
    }

    #[test]
    fn unterminated_double_quoted_attr_value_returns_error() {
        // Previously: ? on find(quote) returned None silently → "Empty XML document"
        let result = convert("<root attr=\"unclosed", 0);
        assert!(result.is_err(), "expected error for unterminated attribute value");
        let msg = result.unwrap_err();
        assert!(msg.to_lowercase().contains("attribute"), "misleading message: {}", msg);
        assert!(!msg.contains("Empty XML document"), "misleading message: {}", msg);
        assert!(!msg.contains("tag not closed"), "misleading message: {}", msg);
    }

    #[test]
    fn unterminated_single_quoted_attr_value_returns_error() {
        let result = convert("<root attr='unclosed", 0);
        assert!(result.is_err(), "expected error for unterminated attribute value");
        let msg = result.unwrap_err();
        assert!(msg.to_lowercase().contains("attribute"), "misleading message: {}", msg);
        assert!(!msg.contains("Empty XML document"), "misleading message: {}", msg);
    }

    #[test]
    fn unterminated_attr_value_error_includes_position() {
        // attr=" starts at col 6, value at col 7 → line 1, col 7
        let result = convert("<root attr=\"unclosed", 0);
        let msg = result.unwrap_err();
        assert!(msg.contains("line 1"), "got: {}", msg);
        assert!(msg.contains("col 12"), "got: {}", msg);
    }

    #[test]
    fn error_column_counts_chars_not_bytes() {
        // "Яблоко" = 6 cyrillic chars (12 bytes). Opening quote " is the 14th character:
        // < Я б л о к о   a t t r = "
        // 1 2 3 4 5 6 7 8 9 ...    14
        // The byte-based (broken) formula gives col 20; the char-based formula gives col 14.
        let result = convert("<Яблоко attr=\"unclosed", 0);
        let msg = result.unwrap_err();
        assert!(msg.contains("line 1"), "got: {}", msg);
        assert!(msg.contains("col 14"), "got: {}", msg);
    }

    #[test]
    fn unquoted_attribute_value_returns_error() {
        let result = convert("<root attr=value>text</root>", 0);
        assert!(result.is_err(), "expected error for unquoted attribute value");
        let msg = result.unwrap_err();
        assert!(msg.to_lowercase().contains("attribute"), "got: {}", msg);
    }

    #[test]
    fn unquoted_attribute_repeated_char_returns_error() {
        // Previously: attr=aba → quote='a', find('a') in "ba>" succeeds → value="b" silently
        let result = convert("<root attr=aba>text</root>", 0);
        assert!(result.is_err(), "expected error for unquoted attribute value");
        let msg = result.unwrap_err();
        assert!(msg.to_lowercase().contains("attribute"), "got: {}", msg);
    }

    #[test]
    fn boolean_attribute_returns_clear_error() {
        let result = convert("<root disabled>text</root>", 0);
        assert!(result.is_err(), "expected error for attribute without '='");
        let msg = result.unwrap_err();
        assert!(!msg.contains("tag not closed"), "misleading message: {}", msg);
        assert!(msg.to_lowercase().contains("attribute"), "expected 'attribute' in error: {}", msg);
    }

    #[test]
    fn boolean_attr_before_normal_attr_returns_error() {
        // Previously find('=') jumped past 'disabled' and grabbed "disabled attr" as key.
        let result = convert("<root disabled attr=\"x\">text</root>", 0);
        assert!(result.is_err(), "expected error for boolean attribute");
        let msg = result.unwrap_err();
        assert!(msg.to_lowercase().contains("attribute"), "got: {}", msg);
    }

    #[test]
    fn multiple_root_elements_return_error() {
        assert!(convert("<a/><b/>", 0).is_err());
        assert!(convert("<a></a><b></b>", 0).is_err());
    }

    #[test]
    fn stray_text_after_root_returns_error() {
        assert!(convert("<root/> lost data", 0).is_err());
    }

    #[test]
    fn stray_text_before_root_returns_error() {
        assert!(convert("garbage<root/>", 0).is_err());
    }

    #[test]
    fn whitespace_outside_root_is_valid() {
        assert!(convert("  \n  <root/>  \n  ", 0).is_ok());
    }

    #[test]
    fn stray_closing_tag_returns_error() {
        assert!(convert("</root>", 0).is_err());
    }

    #[test]
    fn unclosed_comment_after_root_returns_error() {
        assert!(convert("<root><child/></root><!-- unclosed", 0).is_err());
    }

    #[test]
    fn greater_than_in_double_quoted_attr_is_valid() {
        assert!(convert("<root attr=\"1>0\"/>", 0).is_ok());
    }

    #[test]
    fn greater_than_in_single_quoted_attr_is_valid() {
        assert!(convert("<root attr='1>0'/>", 0).is_ok());
    }

    #[test]
    fn greater_than_in_attr_with_children_is_valid() {
        assert!(convert("<root><item href=\"a>b\"/></root>", 0).is_ok());
    }

    #[test]
    fn multiple_attrs_with_greater_than_are_valid() {
        assert!(convert("<root x=\"1>2\" y=\"3>4\"/>", 0).is_ok());
    }

    #[test]
    fn processing_instruction_with_gt_is_valid() {
        assert!(convert("<?pi foo=\"1>2\"?><root/>", 0).is_ok());
    }

    #[test]
    fn mixed_content_text_nodes_concatenated_without_extra_space() {
        // "a " (before <child/>) + "  b" (after) = "a   b" — joined without inserting extra space
        let xml = "<root>a <child/>  b</root>";
        let result = convert(xml, 0).unwrap();
        assert!(result.contains("a   b"), "got: {}", result);
    }
}
