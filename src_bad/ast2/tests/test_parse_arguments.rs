
use crate::ast2::parser::Parser;
use crate::ast2::parser::arguments;

#[test]
fn test_try_parse_arguments_single() {
    let parser = Parser::new("'arg1' ");
    let (args, p_next) = arguments::_try_parse_arguments(&parser).unwrap().unwrap();
    assert_eq!(args.arguments.len(), 1);
    assert_eq!(args.arguments[0].value, "arg1");
    assert_eq!(p_next.remain(), " ");

    let arg1_str = "'arg1'";
    assert_eq!(args.range.begin.offset, 0);
    assert_eq!(args.range.end.offset, arg1_str.len());
}

#[test]
fn test_try_parse_arguments_multiple() {
    let parser = Parser::new("'arg1' \"arg2\" nude_arg ");
    let (args, p_next) = arguments::_try_parse_arguments(&parser).unwrap().unwrap();
    assert_eq!(args.arguments.len(), 3);
    assert_eq!(args.arguments[0].value, "arg1");
    assert_eq!(args.arguments[1].value, "arg2");
    assert_eq!(args.arguments[2].value, "nude_arg");
    assert_eq!(p_next.remain(), " ");

    let arg1_str = "'arg1' ";
    let arg2_str = "\"arg2\" ";
    let arg3_str = "nude_arg";
    assert_eq!(args.range.begin.offset, 0);
    assert_eq!(
        args.range.end.offset,
        arg1_str.len() + arg2_str.len() + arg3_str.len()
    );
}

#[test]
fn test_try_parse_arguments_empty() {
    let parser = Parser::new("");
    let result = arguments::_try_parse_arguments(&parser).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_try_parse_arguments_with_anchor_end() {
    let parser = Parser::new("'arg1' --> rest");
    let (args, p_next) = arguments::_try_parse_arguments(&parser).unwrap().unwrap();
    assert_eq!(args.arguments.len(), 1);
    assert_eq!(args.arguments[0].value, "arg1");
    assert_eq!(p_next.remain(), " --> rest");

    let arg1_str = "'arg1'";
    assert_eq!(args.range.begin.offset, 0);
    assert_eq!(args.range.end.offset, arg1_str.len());
}
