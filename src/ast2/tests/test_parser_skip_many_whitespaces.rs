use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;

#[test]
fn test_parser_skip_many_whitespaces() {
    let document = "  \t\rhello";
    let mut parser = Parser::new(document);
    parser.skip_many_whitespaces();
    assert_eq!(parser.get_position(), create_pos(4, 1, 5));
    assert_eq!(parser.remain(), "hello");
}
