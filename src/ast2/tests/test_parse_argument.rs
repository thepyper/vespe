use crate::ast2::parser::Parser;
use crate::ast2::parser::arguments;

#[test]
fn test_try_parse_argument_single_quoted() {
    let (arg, p_next) = arguments::_try_parse_argument(&parser).unwrap().unwrap();
    assert_eq!(arg.value, "arg1");
    assert_eq!(p_next.remain(), " rest");
    assert_eq!(arg.range.begin.offset, 0);
    assert_eq!(arg.range.end.offset, "'arg1'".len());
}

#[test]
fn test_try_parse_argument_double_quoted() {
    let (arg, p_next) = arguments::_try_parse_argument(&parser).unwrap().unwrap();
    assert_eq!(arg.value, "arg2");
    assert_eq!(p_next.remain(), " rest");
    assert_eq!(arg.range.begin.offset, 0);
    assert_eq!(arg.range.end.offset, r#""arg2""#.len());
}

#[test]
fn test_try_parse_argument_nude() {
    let (arg, p_next) = arguments::_try_parse_argument(&parser).unwrap().unwrap();
    assert_eq!(arg.value, "nude_arg");
    assert_eq!(p_next.remain(), " rest");
    assert_eq!(arg.range.begin.offset, 0);
    assert_eq!(arg.range.end.offset, "nude_arg".len());
}

#[test]
fn test_try_parse_argument_empty() {
    let result = arguments::_try_parse_argument(&parser).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_try_parse_argument_no_match() {
    let result = arguments::_try_parse_argument(&parser).unwrap();
    assert!(result.is_none());
}
