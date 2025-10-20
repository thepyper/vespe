use serde_json::json;

use super::parser::Parser;
use super::error::{Ast2Error, Result};
use super::types::{CommandKind, AnchorKind, Parameters, Argument, Arguments};
use super::parse_primitives::{_try_parse_identifier, _try_parse_enclosed_string};

pub(crate) fn _try_parse_command_kind<'doc>(parser: &Parser<'doc>) -> Result<Option<(CommandKind, Parser<'doc>)>> {
    let tags_list = vec![
        ("tag", CommandKind::Tag),
        ("include", CommandKind::Include),
        ("inline", CommandKind::Inline),
        ("answer", CommandKind::Answer),
        ("summarize", CommandKind::Summarize),
        ("derive", CommandKind::Derive),
        ("repeat", CommandKind::Repeat),
    ];

    for (name, kind) in tags_list {
        if let Some(new_parser) = parser.consume_matching_string_immutable(name) {
            return Ok(Some((kind, new_parser)));
        }
    }

    Ok(None)
}

pub(crate) fn _try_parse_anchor_kind<'doc>(parser: &Parser<'doc>) -> Result<Option<(AnchorKind, Parser<'doc>)>> {
    let tags_list = vec![("begin", AnchorKind::Begin), ("end", AnchorKind::End)];

    for (name, kind) in tags_list {
        if let Some(new_parser) = parser.consume_matching_string_immutable(name) {
            return Ok(Some((kind, new_parser)));
        }
    }

    Ok(None)
}

pub(crate) fn _try_parse_parameters<'doc>(parser: &Parser<'doc>) -> Result<Option<(Parameters, Parser<'doc>)>> {
    let begin = parser.get_position();

    // Must start with '['
    let mut p_current = match parser.consume_matching_char_immutable('[') {
        Some(p) => p,
        None => return Ok(None),
    };
    p_current = p_current.skip_many_whitespaces_or_eol_immutable();

    // Check for empty parameters: []
    if let Some(p_final) = p_current.consume_matching_char_immutable(']') {
        let end = p_final.get_position();
        return Ok(Some((
            Parameters {
                parameters: serde_json::Map::new(),
                range: super::types::Range { begin, end }, // Use super::types::Range
            },
            p_final,
        )));
    }

    let mut parameters_map = serde_json::Map::new();

    // Loop to parse key-value pairs
    loop {
        // Parse a parameter
        let ((key, value), p_after_param) = match _try_parse_parameter(&p_current)? {
            Some((param, p_next)) => (param, p_next),
            None => {
                // This means we couldn't parse a parameter where one was expected.
                return Err(Ast2Error::ParameterNotParsed {
                    position: p_current.get_position(),
                });
            }
        };
        parameters_map.insert(key, value);
        p_current = p_after_param.skip_many_whitespaces_or_eol_immutable();

        // After a parameter, we expect either a ']' (end) or a ',' (continue)
        if let Some(p_final) = p_current.consume_matching_char_immutable(']') {
            // End of parameters
            let end = p_final.get_position();
            return Ok(Some((
                Parameters {
                    parameters: parameters_map,
                    range: super::types::Range { begin, end }, // Use super::types::Range
                },
                p_final,
            )));
        } else if let Some(p_after_comma) = p_current.consume_matching_char_immutable(',') {
            // Comma found, continue loop
            p_current = p_after_comma.skip_many_whitespaces_or_eol_immutable();
        } else {
            // Neither ']' nor ',' found after a parameter. Syntax error.
            return Err(Ast2Error::MissingCommaInParameters { // Or missing closing brace
                position: p_current.get_position(),
            });
        }
    }
}

pub(crate) fn _try_parse_parameter<'doc>(
    parser: &Parser<'doc>,
) -> Result<Option<((String, serde_json::Value), Parser<'doc>)>> {
    let p_initial = parser.skip_many_whitespaces_or_eol_immutable();

    let (key, p1) = match _try_parse_identifier(&p_initial)? {
        Some((k, p)) => (k, p),
        None => return Ok(None), // Not an error, just didn't find an identifier
    };

    let p2 = p1.skip_many_whitespaces_or_eol_immutable();

    let p3 = match p2.consume_matching_char_immutable('=') {
        Some(p) => p,
        None => return Ok(None), // No colon, so not a parameter. Let the caller decide what to do.
    };

    let p4 = p3.skip_many_whitespaces_or_eol_immutable();

    let (value, p5) = match _try_parse_value(&p4) {
        Ok(Some((v, p))) => (v, p),
        Ok(None) => {
            // Here, a key and colon were found, so a value is expected.
            // This IS a syntax error.
            return Err(Ast2Error::MissingParameterValue {
                position: p4.get_position(),
            });
        }
        Err(e) => return Err(e),
    };

    Ok(Some(((key, value), p5)))
}

pub(crate) fn _try_parse_arguments<'doc>(parser: &Parser<'doc>) -> Result<Option<(Arguments, Parser<'doc>)>> {
    let begin = parser.get_position();
    let mut p_current = parser.clone();
    let mut arguments = Vec::new();

    loop {
        let p_current_after_whitespaces = p_current.skip_many_whitespaces_immutable();

        // Check for anchor end, a special case for arguments
        if p_current_after_whitespaces.remain().starts_with("-->") {
            break;
        }

        match _try_parse_argument(&p_current_after_whitespaces)? {
            Some((arg, p_next)) => {
                arguments.push(arg);
                p_current = p_next;
            }
            None => break, // No more arguments to parse
        }
    }

    if arguments.is_empty() {
        return Ok(None);
    }

    let end = p_current.get_position();

    Ok(Some((
        Arguments {
            arguments,
            range: super::types::Range { begin, end }, // Use super::types::Range
        },
        p_current,
    )))
}

pub(crate) fn _try_parse_argument<'doc>(parser: &Parser<'doc>) -> Result<Option<(Argument, Parser<'doc>)>> {
    let begin = parser.get_position();

    if let Some(p1) = parser.consume_matching_char_immutable('\'') {
        if let Some((value, p)) = _try_parse_enclosed_string(&p1, "'")? {
            let end = p.get_position();
            let arg = Argument { value, range: super::types::Range { begin, end } }; // Use super::types::Range
            return Ok(Some((arg, p)));
        }
    }
    
    if let Some(p1) = parser.consume_matching_char_immutable('"') {
        if let Some((value, p)) = _try_parse_enclosed_string(&p1, "\"")? {
            let end = p.get_position();
            let arg = Argument { value, range: super::types::Range { begin, end } }; // Use super::types::Range
            return Ok(Some((arg, p)));
        }
    }

    if let Some((value, p)) = super::parse_primitives::_try_parse_nude_string(parser)? { // Use super::parse_primitives
        let end = p.get_position();
        let arg = Argument { value, range: super::types::Range { begin, end } }; // Use super::types::Range
        return Ok(Some((arg, p)));
    }

    Ok(None)
}

pub(crate) fn _try_parse_value<'doc>(parser: &Parser<'doc>) -> Result<Option<(serde_json::Value, Parser<'doc>)>> {
    if let Some(p1) = parser.consume_matching_char_immutable('"') {
        // try to parse a double-quoted string
        _try_parse_enclosed_value(&p1, "\"")
    } else if let Some(p1) = parser.consume_matching_char_immutable('\'') {
        // try to parse a single-quoted string
        _try_parse_enclosed_value(&p1, "'")
    } else {
        // try to parse a "nude" value (unquoted)
        _try_parse_nude_value(parser)
    }
}

pub(crate) fn _try_parse_enclosed_value<'doc>(
    parser: &Parser<'doc>,
    closure: &str,
) -> Result<Option<(serde_json::Value, Parser<'doc>)>> {
    match _try_parse_enclosed_string(parser, closure)? {
        Some((s, p)) => Ok(Some((serde_json::Value::String(s), p))),
        None => Ok(None),
    }
}

pub(crate) fn _try_parse_nude_value<'doc>(parser: &Parser<'doc>) -> Result<Option<(serde_json::Value, Parser<'doc>)>> {
    if let Some((x, p)) = super::parse_primitives::_try_parse_nude_float(parser)? { // Use super::parse_primitives
        return Ok(Some((json!(x), p)));
    } 
    if let Some((x, p)) = super::parse_primitives::_try_parse_nude_integer(parser)? { // Use super::parse_primitives
        return Ok(Some((json!(x), p)));
    } 
    if let Some((x, p)) = super::parse_primitives::_try_parse_nude_bool(parser)? { // Use super::parse_primitives
        return Ok(Some((json!(x), p)));
    } 
    if let Some((x, p)) = super::parse_primitives::_try_parse_nude_string(parser)? { // Use super::parse_primitives
        return Ok(Some((json!(x), p)));
        }
        Ok(None)
    }
    
    #[cfg(test)]
    mod tests {
        use crate::ast2::parser::Parser;
        use crate::ast2::error::{Ast2Error, Result};
        use crate::ast2::types::{CommandKind, AnchorKind, Parameters, Argument, Arguments};
        use crate::ast2::parse_primitives::{_try_parse_nude_float, _try_parse_nude_integer, _try_parse_nude_bool, _try_parse_nude_string, _try_parse_nude_value, _try_parse_identifier, _try_parse_enclosed_string};
        use serde_json::json;
        use super::{_try_parse_value, _try_parse_enclosed_value, _try_parse_argument, _try_parse_arguments, _try_parse_parameter, _try_parse_parameters, _try_parse_command_kind, _try_parse_anchor_kind};
    
        #[test]
        fn test_try_parse_identifier_valid() {
            let doc = "_my_identifier123 rest";
            let parser = Parser::new(doc);
                            let (identifier, p_next) = _try_parse_identifier(&parser).unwrap().unwrap();    
            assert_eq!(identifier, "_my_identifier123");
            assert_eq!(p_next.remain(), " rest");
        }
    
        #[test]
        fn test_try_parse_identifier_starts_with_digit() {
            let doc = "123identifier";
            let parser = Parser::new(doc);
            let result = _try_parse_identifier(&parser).unwrap();
    
            assert!(result.is_none());
            assert_eq!(parser.remain(), "123identifier");
        }
    
        #[test]
        fn test_try_parse_identifier_empty() {
            let doc = "";
            let parser = Parser::new(doc);
            let result = _try_parse_identifier(&parser).unwrap();
    
            assert!(result.is_none());
        }
    
            #[test]
            fn test_try_parse_identifier_with_invalid_char() {
                let doc = "my-identifier";
                let parser = Parser::new(doc);
            let (identifier, p_next) = _try_parse_identifier(&parser).unwrap().unwrap();
        
                assert_eq!(identifier, "my");
                assert_eq!(p_next.remain(), "-identifier");
            }
        
            #[test]
            fn test_try_parse_nude_integer() {
                let doc = "123 rest";
                let parser = Parser::new(doc);
                let (value, p_next) = _try_parse_nude_integer(&parser).unwrap().unwrap();
                assert_eq!(value, 123);
                assert_eq!(p_next.remain(), " rest");
        
                let doc_no_int = "abc";
                let parser_no_int = Parser::new(doc_no_int);
                assert!(_try_parse_nude_integer(&parser_no_int).unwrap().is_none());
        
                let doc_empty = "";
                let parser_empty = Parser::new(doc_empty);
                assert!(_try_parse_nude_integer(&parser_empty).unwrap().is_none());
            }
        
            #[test]
            fn test_try_parse_nude_float() {
                let doc = "123.45 rest";
                let parser = Parser::new(doc);
                let (value, p_next) = _try_parse_nude_float(&parser).unwrap().unwrap();
                assert_eq!(value, 123.45);
                assert_eq!(p_next.remain(), " rest");
        
                let doc_no_float = "123 rest";
                let parser_no_float = Parser::new(doc_no_float);
                assert!(_try_parse_nude_float(&parser_no_float).unwrap().is_none());
        
                let doc_just_dot = ". rest";
                let parser_just_dot = Parser::new(doc_just_dot);
                assert!(_try_parse_nude_float(&parser_just_dot).unwrap().is_none());
        
                let doc_empty = "";
                let parser_empty = Parser::new(doc_empty);
                assert!(_try_parse_nude_float(&parser_empty).unwrap().is_none());
            }
        
            #[test]
            fn test_try_parse_nude_bool() {
                let doc_true = "true rest";
                let parser_true = Parser::new(doc_true);
                let (value_true, p_next_true) = _try_parse_nude_bool(&parser_true).unwrap().unwrap();
                assert_eq!(value_true, true);
                assert_eq!(p_next_true.remain(), " rest");
        
                let doc_false = "false rest";
                let parser_false = Parser::new(doc_false);
                let (value_false, p_next_false) = _try_parse_nude_bool(&parser_false).unwrap().unwrap();
                assert_eq!(value_false, false);
                assert_eq!(p_next_false.remain(), " rest");
        
                let doc_no_bool = "other rest";
                let parser_no_bool = Parser::new(doc_no_bool);
                assert!(_try_parse_nude_bool(&parser_no_bool).unwrap().is_none());
            }
        
            #[test]
            fn test_try_parse_nude_string() {
                let doc = "hello/world.txt_123 rest";
                let parser = Parser::new(doc);
                let (value, p_next) = _try_parse_nude_string(&parser).unwrap().unwrap();
                assert_eq!(value, "hello/world.txt_123");
                assert_eq!(p_next.remain(), " rest");
        
                let doc_empty = "";
                let parser_empty = Parser::new(doc_empty);
                assert!(_try_parse_nude_string(&parser_empty).unwrap().is_none());
        
                let doc_with_space = "hello world";
                let parser_with_space = Parser::new(doc_with_space);
                let (value_space, p_next_space) = _try_parse_nude_string(&parser_with_space).unwrap().unwrap();
                assert_eq!(value_space, "hello");
                assert_eq!(p_next_space.remain(), " world");
            }
        
    #[test]
    fn test_try_parse_value_nude() {
        let doc_int = "123 rest";
        let parser_int = Parser::new(doc_int);
        let (value_int, p_next_int) = _try_parse_value(&parser_int).unwrap().unwrap();
        assert_eq!(value_int, json!(123));
        assert_eq!(p_next_int.remain(), " rest");

        let doc_float = "123.45 rest";
        let parser_float = Parser::new(doc_float);
        let (value_float, p_next_float) = _try_parse_value(&parser_float).unwrap().unwrap();
        assert_eq!(value_float, json!(123.45));
        assert_eq!(p_next_float.remain(), " rest");

        let doc_bool = "true rest";
        let parser_bool = Parser::new(doc_bool);
        let (value_bool, p_next_bool) = _try_parse_value(&parser_bool).unwrap().unwrap();
        assert_eq!(value_bool, json!(true));
        assert_eq!(p_next_bool.remain(), " rest");

        let doc_string = "nude_string rest";
        let parser_string = Parser::new(doc_string);
        let (value_string, p_next_string) = _try_parse_value(&parser_string).unwrap().unwrap();
        assert_eq!(value_string, json!("nude_string"));
        assert_eq!(p_next_string.remain(), " rest");

        let doc_empty = "";
        let parser_empty = Parser::new(doc_empty);
        assert!(_try_parse_value(&parser_empty).unwrap().is_none());
    }

    #[test]
    fn test_try_parse_enclosed_string_double_quote() {
        let doc = r#""hello world" rest"#;
        let parser = Parser::new(doc);
        let p_after_opening_quote = parser.consume_matching_char_immutable('"').unwrap(); // Consume opening quote
        let (value, p_next) = _try_parse_enclosed_string(&p_after_opening_quote, "\"").unwrap().unwrap();
        assert_eq!(value, "hello world");
        assert_eq!(p_next.remain(), " rest");

        let doc_escaped = r#""hello \"world\"" rest"#;
        let parser_escaped = Parser::new(doc_escaped);
        let p_after_opening_quote_escaped = parser_escaped.consume_matching_char_immutable('"').unwrap(); // Consume opening quote
        let (value_escaped, p_next_escaped) = _try_parse_enclosed_string(&p_after_opening_quote_escaped, "\"").unwrap().unwrap();
        assert_eq!(value_escaped, "hello \"world\""); // Expect unescaped
        assert_eq!(p_next_escaped.remain(), " rest");

        let doc_unclosed = r#""hello"#;
        let parser_unclosed = Parser::new(doc_unclosed);
        let p_after_opening_quote_unclosed = parser_unclosed.consume_matching_char_immutable('"').unwrap(); // Consume opening quote
        let result = _try_parse_enclosed_string(&p_after_opening_quote_unclosed, "\"");
        assert!(matches!(result, Err(Ast2Error::UnclosedString { .. })));
    }

    #[test]
    fn test_try_parse_enclosed_string_single_quote() {
        let doc = "'hello world' rest";
        let parser = Parser::new(doc);
        let p_after_opening_quote = parser.consume_matching_char_immutable('"').unwrap(); // Consume opening quote
        let (value, p_next) = _try_parse_enclosed_string(&p_after_opening_quote, "'").unwrap().unwrap();
        assert_eq!(value, "hello world");
        assert_eq!(p_next.remain(), " rest");

        let doc_escaped = "'hello \'world\'' rest";
        let parser_escaped = Parser::new(doc_escaped);
        let p_after_opening_quote_escaped = parser_escaped.consume_matching_char_immutable('"').unwrap(); // Consume opening quote
        let (value_escaped, p_next_escaped) = _try_parse_enclosed_string(&p_after_opening_quote_escaped, "'").unwrap().unwrap();
        assert_eq!(value_escaped, "hello 'world'"); // Expect unescaped
        assert_eq!(p_next_escaped.remain(), " rest");
    }

    #[test]
    fn test_try_parse_enclosed_value_double_quote() {
        let doc = r#""json value" rest"#;
        let parser = Parser::new(doc);
        let p_after_opening_quote = parser.consume_matching_char_immutable('"').unwrap(); // Consume opening quote
        let (value, p_next) = _try_parse_enclosed_value(&p_after_opening_quote, "\"").unwrap().unwrap();
        assert_eq!(value, json!("json value"));
        assert_eq!(p_next.remain(), " rest");
    }

    #[test]
    fn test_try_parse_enclosed_value_single_quote() {
        let doc = "'json value' rest";
        let parser = Parser::new(doc);
        let p_after_opening_quote = parser.consume_matching_char_immutable('"').unwrap(); // Consume opening quote
        let (value, p_next) = _try_parse_enclosed_value(&p_after_opening_quote, "'").unwrap().unwrap();
        assert_eq!(value, json!("json value"));
        assert_eq!(p_next.remain(), " rest");
    }

    #[test]
    fn test_try_parse_value_enclosed() {
        let doc_double = r#""double quoted" rest"#;
        let parser_double = Parser::new(doc_double);
        let (value_double, p_next_double) = _try_parse_value(&parser_double).unwrap().unwrap();
        assert_eq!(value_double, json!("double quoted"));
        assert_eq!(p_next_double.remain(), " rest");

        let doc_single = "'single quoted' rest";
        let parser_single = Parser::new(doc_single);
        let (value_single, p_next_single) = _try_parse_value(&parser_single).unwrap().unwrap();
        assert_eq!(value_single, json!("single quoted"));
        assert_eq!(p_next_single.remain(), " rest");
    }

    #[test]
    fn test_try_parse_argument_single_quoted() {
        let doc = "'arg1' rest";
        let parser = Parser::new(doc);
        let (arg, p_next) = _try_parse_argument(&parser).unwrap().unwrap();
        assert_eq!(arg.value, "arg1");
        assert_eq!(p_next.remain(), " rest");
        assert_eq!(arg.range.begin.offset, 0);
        assert_eq!(arg.range.end.offset, "'arg1'".len());
    }

    #[test]
    fn test_try_parse_argument_double_quoted() {
        let doc = r#""arg2" rest"#;
        let parser = Parser::new(doc);
        let (arg, p_next) = _try_parse_argument(&parser).unwrap().unwrap();
        assert_eq!(arg.value, "arg2");
        assert_eq!(p_next.remain(), " rest");
        assert_eq!(arg.range.begin.offset, 0);
        assert_eq!(arg.range.end.offset, r#""arg2""#.len());
    }

    #[test]
    fn test_try_parse_argument_nude() {
        let doc = "nude_arg rest";
        let parser = Parser::new(doc);
        let (arg, p_next) = _try_parse_argument(&parser).unwrap().unwrap();
        assert_eq!(arg.value, "nude_arg");
        assert_eq!(p_next.remain(), " rest");
        assert_eq!(arg.range.begin.offset, 0);
        assert_eq!(arg.range.end.offset, "nude_arg".len());
    }

    #[test]
    fn test_try_parse_argument_empty() {
        let doc = "";
        let parser = Parser::new(doc);
        let result = _try_parse_argument(&parser).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_try_parse_argument_no_match() {
        let doc = "@tag rest";
        let parser = Parser::new(doc);
        let result = _try_parse_argument(&parser).unwrap();
        assert!(result.is_none());
    }

    // Tests from test_parse_arguments.rs
    #[test]
    fn test_try_parse_arguments_single() {
        let doc = "'arg1' ";
        let parser = Parser::new(doc);
        let (args, p_next) = _try_parse_arguments(&parser).unwrap().unwrap();
        assert_eq!(args.arguments.len(), 1);
        assert_eq!(args.arguments[0].value, "arg1");
        assert_eq!(p_next.remain(), " ");

        let arg1_str = "'arg1'";
        assert_eq!(args.range.begin.offset, 0);
        assert_eq!(args.range.end.offset, arg1_str.len());
    }

    #[test]
    fn test_try_parse_arguments_multiple() {
        let doc = "'arg1' \"arg2\" nude_arg ";
        let parser = Parser::new(doc);
        let (args, p_next) = _try_parse_arguments(&parser).unwrap().unwrap();
        assert_eq!(args.arguments.len(), 3);
        assert_eq!(args.arguments[0].value, "arg1");
        assert_eq!(args.arguments[1].value, "arg2");
        assert_eq!(args.arguments[2].value, "nude_arg");
        assert_eq!(p_next.remain(), " ");

        let arg1_str = "'arg1' ";
        let arg2_str = "\"arg2\" ";
        let arg3_str = "nude_arg";
        assert_eq!(args.range.begin.offset, 0);
        assert_eq!(args.range.end.offset, arg1_str.len() + arg2_str.len() + arg3_str.len());
    }

    #[test]
    fn test_try_parse_arguments_empty() {
        let doc = "";
        let parser = Parser::new(doc);
        let result = _try_parse_arguments(&parser).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_try_parse_arguments_with_anchor_end() {
        let doc = "'arg1' --> rest";
        let parser = Parser::new(doc);
        let (args, p_next) = _try_parse_arguments(&parser).unwrap().unwrap();
        assert_eq!(args.arguments.len(), 1);
        assert_eq!(args.arguments[0].value, "arg1");
        assert_eq!(p_next.remain(), " --> rest");

        let arg1_str = "'arg1'";
        assert_eq!(args.range.begin.offset, 0);
        assert_eq!(args.range.end.offset, arg1_str.len());
    }

    // Tests from test_parse_parameters.rs
    #[test]
    fn test_try_parse_parameter_valid() {
        let doc = "key=value rest";
        let parser = Parser::new(doc);
        let ((key, value), p_next) = _try_parse_parameter(&parser).unwrap().unwrap();
        assert_eq!(key, "key");
        assert_eq!(value, json!("value"));
        assert_eq!(p_next.remain(), " rest");
    }

    #[test]
    fn test_try_parse_parameter_with_spaces() {
        let doc = "  key  =  \"value with spaces\"  rest";
        let parser = Parser::new(doc);
        let ((key, value), p_next) = _try_parse_parameter(&parser).unwrap().unwrap();
        assert_eq!(key, "key");
        assert_eq!(value, json!("value with spaces"));
        assert_eq!(p_next.remain(), "  rest");
    }

    #[test]
    fn test_try_parse_parameter_missing_value() {
        let doc = "key= rest";
        let parser = Parser::new(doc);
        let ((key, value), p_next) = _try_parse_parameter(&parser).unwrap().unwrap();
        assert_eq!(key, "key");
        assert_eq!(value, json!("rest"));
        assert_eq!(p_next.remain(), "");
    }

    #[test]
    fn test_try_parse_parameter_no_equal() {
        let doc = "key value rest";
        let parser = Parser::new(doc);
        let result = _try_parse_parameter(&parser).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_try_parse_parameters_empty() {
        let doc = "[] rest";
        let parser = Parser::new(doc);
        let (params, p_next) = _try_parse_parameters(&parser).unwrap().unwrap();
        assert!(params.parameters.is_empty());
        assert_eq!(p_next.remain(), " rest");

        let begin_str = "[";
        let end_str = "]";
        assert_eq!(params.range.begin.offset, 0);
        assert_eq!(params.range.end.offset, begin_str.len() + end_str.len());
    }

    #[test]
    fn test_try_parse_parameters_single() {
        let doc = "[key=value] rest";
        let parser = Parser::new(doc);
        let (params, p_next) = _try_parse_parameters(&parser).unwrap().unwrap();
        assert_eq!(params.parameters.len(), 1);
        assert_eq!(params.parameters["key"], json!("value"));
        assert_eq!(p_next.remain(), " rest");

        let full_str = "[key=value]";
        assert_eq!(params.range.begin.offset, 0);
        assert_eq!(params.range.end.offset, full_str.len());
    }

    #[test]
    fn test_try_parse_parameters_multiple() {
        let doc = "[key1=value1, key2=\"value 2\"] rest";
        let parser = Parser::new(doc);
        let (params, p_next) = _try_parse_parameters(&parser).unwrap().unwrap();
        assert_eq!(params.parameters.len(), 2);
        assert_eq!(params.parameters["key1"], json!("value1"));
        assert_eq!(params.parameters["key2"], json!("value 2"));
        assert_eq!(p_next.remain(), " rest");

        let full_str = "[key1=value1, key2=\"value 2\"]";
        assert_eq!(params.range.begin.offset, 0);
        assert_eq!(params.range.end.offset, full_str.len());
    }

    #[test]
    fn test_try_parse_parameters_missing_comma() {
        let doc = "[key1=value1 key2=value2]";
        let parser = Parser::new(doc);
        let result = _try_parse_parameters(&parser);
        assert!(matches!(result, Err(Ast2Error::MissingCommaInParameters { .. })));
    }

    #[test]
    fn test_try_parse_parameters_unclosed() {
        let doc = "[key=value";
        let parser = Parser::new(doc);
        let result = _try_parse_parameters(&parser);
        assert!(matches!(result, Err(Ast2Error::MissingCommaInParameters { .. }))); // Currently reports missing comma
    }

    #[test]
    fn test_try_parse_parameters_no_opening_bracket() {
        let doc = "key=value] rest";
        let parser = Parser::new(doc);
        let result = _try_parse_parameters(&parser).unwrap();
        assert!(result.is_none());
    }

    // Tests from test_parse_kinds.rs
    #[test]
    fn test_try_parse_command_kind_valid() {
        let doc = "tag rest";
        let parser = Parser::new(doc);
        let (kind, p_next) = _try_parse_command_kind(&parser).unwrap().unwrap();
        assert_eq!(kind, CommandKind::Tag);
        assert_eq!(p_next.remain(), " rest");

        let doc = "include rest";
        let parser = Parser::new(doc);
        let (kind, p_next) = _try_parse_command_kind(&parser).unwrap().unwrap();
        assert_eq!(kind, CommandKind::Include);
        assert_eq!(p_next.remain(), " rest");

        let doc = "answer rest";
        let parser = Parser::new(doc);
        let (kind, p_next) = _try_parse_command_kind(&parser).unwrap().unwrap();
        assert_eq!(kind, CommandKind::Answer);
        assert_eq!(p_next.remain(), " rest");
    }

    #[test]
    fn test_try_parse_command_kind_invalid() {
        let doc = "invalid_command rest";
        let parser = Parser::new(doc);
        let result = _try_parse_command_kind(&parser).unwrap();
        assert!(result.is_none());
        assert_eq!(parser.remain(), "invalid_command rest");
    }

    #[test]
    fn test_try_parse_anchor_kind_valid() {
        let doc = "begin rest";
        let parser = Parser::new(doc);
        let (kind, p_next) = _try_parse_anchor_kind(&parser).unwrap().unwrap();
        assert_eq!(kind, AnchorKind::Begin);
        assert_eq!(p_next.remain(), " rest");

        let doc = "end rest";
        let parser = Parser::new(doc);
        let (kind, p_next) = _try_parse_anchor_kind(&parser).unwrap().unwrap();
        assert_eq!(kind, AnchorKind::End);
        assert_eq!(p_next.remain(), " rest");
    }

    #[test]
    fn test_try_parse_anchor_kind_invalid() {
        let doc = "invalid_anchor rest";
        let parser = Parser::new(doc);
        let result = _try_parse_anchor_kind(&parser).unwrap();
        assert!(result.is_none());
        assert_eq!(parser.remain(), "invalid_anchor rest");
    }
}