use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;

#[test]
fn test_parser_consume_matching_string() {
    let document = "hello world";
    let mut parser = Parser::new(document);
    assert!(parser.consume_matching_string("hello"));
    assert_eq!(parser.get_position(), create_pos(5, 1, 6));
    assert!(!parser.consume_matching_string("world"));
    assert_eq!(parser.get_position(), create_pos(5, 1, 6));
}
