use super::*;

#[test]
fn test_simple_nude_string() {
    let part1 = "nude_string";
    let part2 = " after";
    let text = format!("{}{}", part1, part2);
    let parser = Parser::new(&text);

    let result = _try_parse_nude_string(&parser).unwrap();
    assert!(result.is_some());

    let (value, next_parser) = result.unwrap();
    assert_eq!(value, part1);
    assert_eq!(next_parser.get_offset(), part1.len());
    assert_eq!(next_parser.remain(), part2);
}

#[test]
fn test_no_nude_string() {
    let text = " 'enclosed_string'";
    let parser = Parser::new(&text);

    let result = _try_parse_nude_string(&parser).unwrap();
    assert!(result.is_none());
}
