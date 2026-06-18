mod json_lexer;
mod json_model;
mod json_parser;

pub fn convert(json: &str, pretty: bool) -> Result<String, &'static str> {
    let mut lexer = json_lexer::Lexer::new(json);
    let tokens = lexer.tokenize()?;

    let root_node = json_parser::Parser::new(tokens).parse()?;

    if pretty {
        Ok(root_node.to_pretty_xml(4))
    } else {
        Ok(root_node.to_xml())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_string_field() {
        let result = convert("{\"name\": \"Alice\"}", false).unwrap();
        assert!(result.contains("<name>Alice</name>"));
    }

    #[test]
    fn nested_object() {
        let result = convert("{\"person\": {\"age\": 30}}", false).unwrap();
        assert!(result.contains("<person>"));
        assert!(result.contains("<age>30</age>"));
    }

    #[test]
    fn array_elements_wrapped_in_item() {
        let result = convert("{\"tags\": [\"a\", \"b\"]}", false).unwrap();
        assert!(result.contains("<item>a</item>"));
        assert!(result.contains("<item>b</item>"));
    }

    #[test]
    fn boolean_true_value() {
        let result = convert("{\"active\": true}", false).unwrap();
        assert!(result.contains("<active>true</active>"));
    }

    #[test]
    fn boolean_false_value() {
        let result = convert("{\"active\": false}", false).unwrap();
        assert!(result.contains("<active>false</active>"));
    }

    #[test]
    fn null_becomes_self_closing_tag() {
        let result = convert("{\"field\": null}", false).unwrap();
        assert!(result.contains("<field />"));
    }

    #[test]
    fn number_value() {
        let result = convert("{\"count\": 42}", false).unwrap();
        assert!(result.contains("<count>42</count>"));
    }

    #[test]
    fn special_chars_in_string_are_escaped() {
        let result = convert("{\"msg\": \"a & b\"}", false).unwrap();
        assert!(result.contains("&amp;"));
    }

    #[test]
    fn less_than_greater_than_escaped() {
        let result = convert("{\"expr\": \"1 < 2 > 0\"}", false).unwrap();
        assert!(result.contains("&lt;"));
        assert!(result.contains("&gt;"));
    }

    #[test]
    fn pretty_mode_contains_newlines() {
        let result = convert("{\"a\": \"b\"}", true).unwrap();
        assert!(result.contains('\n'));
    }

    #[test]
    fn invalid_json_returns_error() {
        assert!(convert("{unclosed", false).is_err());
    }
}
