use super::*;

#[test]
fn test_try_parse_argument_single_quoted() {
    let doc = "'arg1' rest";
    let parser = Parser::new(doc);
    let (arg, p_next) = super::_try_parse_argument(&parser).unwrap().unwrap();
    assert_eq!(arg.value, "arg1");
    assert_eq!(p_next.remain(), " rest");
    assert_eq!(arg.range.begin.offset, 0);
    assert_eq!(arg.range.end.offset, "'arg1'".len());
}

#[test]
fn test_try_parse_argument_double_quoted() {
    let doc = r#""arg2" rest"#;
    let parser = Parser::new(doc);
    let (arg, p_next) = super::_try_parse_argument(&parser).unwrap().unwrap();
    assert_eq!(arg.value, "arg2");
    assert_eq!(p_next.remain(), " rest");
    assert_eq!(arg.range.begin.offset, 0);
    assert_eq!(arg.range.end.offset, r#""arg2""#.len());
}

#[test]
fn test_try_parse_argument_nude() {
    let doc = "nude_arg rest";
    let parser = Parser::new(doc);
    let (arg, p_next) = super::_try_parse_argument(&parser).unwrap().unwrap();
    assert_eq!(arg.value, "nude_arg");
    assert_eq!(p_next.remain(), " rest");
    assert_eq!(arg.range.begin.offset, 0);
    assert_eq!(arg.range.end.offset, "nude_arg".len());
}

#[test]
fn test_try_parse_argument_empty() {
    let doc = "";
    let parser = Parser::new(doc);
    let result = super::_try_parse_argument(&parser).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_try_parse_argument_no_match() {
    let doc = "@tag rest";
    let parser = Parser::new(doc);
    let result = super::_try_parse_argument(&parser).unwrap();
    assert!(result.is_none());
}
