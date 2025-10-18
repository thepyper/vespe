use crate::ast2::{Parser, Ast2Error};

#[test]
fn test_try_parse_arguments_single() {
    let doc = "'arg1' rest";
    let parser = Parser::new(doc);
    let (args, p_next) = super::super::_try_parse_arguments(&parser).unwrap().unwrap();
    assert_eq!(args.arguments.len(), 1);
    assert_eq!(args.arguments[0].value, "arg1");
    assert_eq!(p_next.remain(), " rest");

    let arg1_str = "'arg1'";
    assert_eq!(args.range.begin.offset, 0);
    assert_eq!(args.range.end.offset, arg1_str.len());
}

#[test]
fn test_try_parse_arguments_multiple() {
    let doc = "'arg1' \"arg2\" nude_arg rest";
    let parser = Parser::new(doc);
    let (args, p_next) = super::super::_try_parse_arguments(&parser).unwrap().unwrap();
    assert_eq!(args.arguments.len(), 3);
    assert_eq!(args.arguments[0].value, "arg1");
    assert_eq!(args.arguments[1].value, "arg2");
    assert_eq!(args.arguments[2].value, "nude_arg");
    assert_eq!(p_next.remain(), " rest");

    let arg1_str = "'arg1' ";
    let arg2_str = "\"arg2\" ";
    let arg3_str = "nude_arg";
    assert_eq!(args.range.begin.offset, 0);
    assert_eq!(args.range.end.offset, arg1_str.len() + arg2_str.len() + arg3_str.len());
}

#[test]
fn test_try_parse_arguments_empty() {
    let doc = " rest";
    let parser = Parser::new(doc);
    let result = super::super::_try_parse_arguments(&parser).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_try_parse_arguments_with_anchor_end() {
    let doc = "'arg1' --> rest";
    let parser = Parser::new(doc);
    let (args, p_next) = super::super::_try_parse_arguments(&parser).unwrap().unwrap();
    assert_eq!(args.arguments.len(), 1);
    assert_eq!(args.arguments[0].value, "arg1");
    assert_eq!(p_next.remain(), " --> rest");

    let arg1_str = "'arg1'";
    assert_eq!(args.range.begin.offset, 0);
    assert_eq!(args.range.end.offset, arg1_str.len());
}
