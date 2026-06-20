mod json_lexer;
mod json_model;
mod json_parser;

/// Converts JSON to XML. `indent` controls pretty-printing:
/// 0 = compact output, N > 0 = pretty with N spaces per level.
pub fn convert(json: &str, indent: usize) -> Result<String, String> {
    let json = json.strip_prefix('\u{FEFF}').unwrap_or(json);
    let mut lexer = json_lexer::Lexer::new(json);
    let tokens = lexer.tokenize()?;

    let root_node = json_parser::Parser::new(tokens).parse()?;

    const DECLARATION: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>";

    if indent > 0 {
        Ok(format!("{}\n{}", DECLARATION, root_node.to_pretty_xml(indent)?))
    } else {
        Ok(format!("{}{}", DECLARATION, root_node.to_xml()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_string_field() {
        let result = convert("{\"name\": \"Alice\"}", 0).unwrap();
        assert!(result.contains("<name>Alice</name>"));
    }

    #[test]
    fn nested_object() {
        let result = convert("{\"person\": {\"age\": 30}}", 0).unwrap();
        assert!(result.contains("<person>"));
        assert!(result.contains("<age>30</age>"));
    }

    #[test]
    fn array_in_object_uses_key_as_repeated_tag() {
        let result = convert("{\"tags\": [\"a\", \"b\"]}", 0).unwrap();
        assert!(result.contains("<tags>a</tags>"), "got: {}", result);
        assert!(result.contains("<tags>b</tags>"), "got: {}", result);
        assert!(!result.contains("<item>"), "got: {}", result);
    }

    #[test]
    fn round_trip_repeated_elements() {
        let xml = "<root><item>a</item><item>b</item></root>";
        let json = crate::xml_to_json::convert(xml, 0).unwrap();
        let result = convert(&json, 0).unwrap();
        assert!(result.contains("<item>a</item>"), "got: {}", result);
        assert!(result.contains("<item>b</item>"), "got: {}", result);
        // no nested wrapping
        assert!(!result.contains("<item><item>"), "got: {}", result);
    }

    #[test]
    fn boolean_true_value() {
        let result = convert("{\"active\": true}", 0).unwrap();
        assert!(result.contains("<active>true</active>"));
    }

    #[test]
    fn boolean_false_value() {
        let result = convert("{\"active\": false}", 0).unwrap();
        assert!(result.contains("<active>false</active>"));
    }

    #[test]
    fn null_becomes_self_closing_tag() {
        let result = convert("{\"field\": null}", 0).unwrap();
        assert!(result.contains("<field />"));
    }

    #[test]
    fn number_value() {
        let result = convert("{\"count\": 42}", 0).unwrap();
        assert!(result.contains("<count>42</count>"));
    }

    #[test]
    fn special_chars_in_string_are_escaped() {
        let result = convert("{\"msg\": \"a & b\"}", 0).unwrap();
        assert!(result.contains("&amp;"));
    }

    #[test]
    fn less_than_escaped_greater_than_literal() {
        let result = convert("{\"expr\": \"1 < 2 > 0\"}", 0).unwrap();
        assert!(result.contains("&lt;"));
        assert!(result.contains("> 0"));
    }

    #[test]
    fn cdata_end_sequence_in_value_is_escaped() {
        // XML 1.0 §2.4 forbids "]]>" in text content
        let result = convert("{\"data\": \"a]]>b\"}", 0).unwrap();
        assert!(!result.contains("]]>"), "]]> must not appear raw in output: {}", result);
        assert!(result.contains("]]&gt;"), "got: {}", result);
    }

    #[test]
    fn lone_gt_not_escaped_by_cdata_fix() {
        let result = convert("{\"x\": \"]>\"}", 0).unwrap();
        assert!(result.contains("]>"), "got: {}", result);
    }

    #[test]
    fn pretty_mode_contains_newlines() {
        let result = convert("{\"a\": \"b\"}", 4).unwrap();
        assert!(result.contains('\n'));
    }

    #[test]
    fn utf8_bom_prefix_is_stripped() {
        let json_with_bom = "\u{FEFF}{\"a\": \"b\"}";
        let result = convert(json_with_bom, 0);
        assert!(result.is_ok(), "BOM should be silently stripped: {:?}", result);
        assert!(result.unwrap().contains("<a>b</a>"));
    }

    #[test]
    fn invalid_json_returns_error() {
        assert!(convert("{unclosed", 0).is_err());
    }

    #[test]
    fn trailing_object_returns_error() {
        // Previously the second object was silently dropped
        let result = convert("{\"a\": 1} {\"b\": 2}", 0);
        assert!(result.is_err(), "expected error for two root objects");
        let msg = result.unwrap_err();
        assert!(msg.contains("'{'"), "expected token description in error: {}", msg);
    }

    #[test]
    fn trailing_array_returns_error() {
        let result = convert("[1, 2] [3, 4]", 0);
        assert!(result.is_err(), "expected error for two root arrays");
        let msg = result.unwrap_err();
        assert!(msg.contains("'['"), "expected token description in error: {}", msg);
    }

    #[test]
    fn trailing_number_returns_error() {
        let result = convert("42 43", 0);
        assert!(result.is_err(), "expected error for two root numbers");
        let msg = result.unwrap_err();
        assert!(msg.contains("number"), "expected token description in error: {}", msg);
    }

    #[test]
    fn double_comma_returns_error() {
        assert!(convert("[1,, 2]", 0).is_err());
    }

    #[test]
    fn mismatched_brackets_return_error() {
        assert!(convert("[1, 2}", 0).is_err());
    }

    #[test]
    fn trailing_comma_returns_error() {
        assert!(convert("{\"a\": 1,}", 0).is_err());
    }

    #[test]
    fn invalid_keyword_literal_returns_error() {
        assert!(convert("{\"x\": treu}", 0).is_err());
    }

    #[test]
    fn invalid_number_literal_returns_error() {
        assert!(convert("{\"x\": 1.2.3}", 0).is_err());
    }

    #[test]
    fn large_integer_not_rounded() {
        let result = convert("{\"id\": 9007199254740993}", 0).unwrap();
        assert!(result.contains("<id>9007199254740993</id>"));
    }

    #[test]
    fn output_has_xml_declaration() {
        let result = convert("{\"a\": \"b\"}", 0).unwrap();
        assert!(result.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    }

    #[test]
    fn pretty_output_has_xml_declaration_on_first_line() {
        let result = convert("{\"a\": \"b\"}", 4).unwrap();
        let first_line = result.lines().next().unwrap();
        assert_eq!(first_line, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    }

    #[test]
    fn escaped_quote_in_string_decoded() {
        let result = convert(r#"{"msg": "say \"hi\""}"#, 0).unwrap();
        assert!(result.contains("<msg>say \"hi\"</msg>"), "got: {}", result);
    }

    #[test]
    fn escaped_backslash_in_string_decoded() {
        let result = convert(r#"{"path": "C:\\Users"}"#, 0).unwrap();
        assert!(result.contains(r"<path>C:\Users</path>"), "got: {}", result);
    }

    #[test]
    fn escaped_newline_in_string_decoded() {
        let result = convert("{\"text\": \"line1\\nline2\"}", 0).unwrap();
        assert!(result.contains("<text>line1\nline2</text>"), "got: {}", result);
    }

    #[test]
    fn escaped_tab_in_string_decoded() {
        let result = convert("{\"text\": \"col1\\tcol2\"}", 0).unwrap();
        assert!(result.contains("<text>col1\tcol2</text>"), "got: {}", result);
    }

    #[test]
    fn unicode_escape_latin_decoded() {
        // JSON A decodes to 'A'
        let json = "{\"char\": \"\\u0041\"}";
        let result = convert(json, 0).unwrap();
        assert!(result.contains("<char>A</char>"), "got: {}", result);
    }

    #[test]
    fn unicode_escape_cyrillic_decoded() {
        // JSON Р decodes to 'Р' (cyrillic capital R)
        let json = "{\"letter\": \"\\u0420\"}";
        let result = convert(json, 0).unwrap();
        assert!(result.contains("<letter>\u{0420}</letter>"), "got: {}", result);
    }

    #[test]
    fn surrogate_pair_emoji_in_value() {
        // 😀 = 😀 (U+1F600)
        let json = "{\"icon\": \"\\uD83D\\uDE00\"}";
        let result = convert(json, 0).unwrap();
        assert!(result.contains("<icon>😀</icon>"), "got: {}", result);
    }

    #[test]
    fn surrogate_pair_supplementary_in_value() {
        // 𐀀 = 𐀀 (U+10000, Linear B)
        let json = "{\"ch\": \"\\uD800\\uDC00\"}";
        let result = convert(json, 0).unwrap();
        assert!(result.contains("<ch>\u{10000}</ch>"), "got: {}", result);
    }

    #[test]
    fn lone_high_surrogate_in_value_returns_error() {
        assert!(convert("{\"x\": \"\\uD800\"}", 0).is_err());
    }

    #[test]
    fn lone_low_surrogate_in_value_returns_error() {
        assert!(convert("{\"x\": \"\\uDC00\"}", 0).is_err());
    }
}
