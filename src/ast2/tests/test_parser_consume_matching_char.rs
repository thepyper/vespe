use crate::ast2::*;
use crate::ast2::tests::utils::*;
use anyhow::Result;

#[test]
fn test_parser_consume_matching_char() {
    let document = "abc";
    let mut parser = Parser::new(document);
    assert!(parser.consume_matching_char('a').is_some());
    assert_eq!(parser.get_position(), create_pos(1, 1, 2));
    assert!(parser.consume_matching_char('c').is_none());
    assert_eq!(parser.get_position(), create_pos(1, 1, 2));
}
