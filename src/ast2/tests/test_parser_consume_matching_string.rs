use crate::ast2::*;
use crate::ast2::tests::utils::*;
use anyhow::Result;

#[test]
fn test_parser_consume_matching_string() {
    let document = "hello world";
    let mut parser = Parser::new(document);
    assert!(parser.consume_matching_string("hello").is_some());
    assert_eq!(parser.get_position(), create_pos(5, 1, 6));
    assert!(parser.consume_matching_string("world").is_none());
    assert_eq!(parser.get_position(), create_pos(5, 1, 6));
}
