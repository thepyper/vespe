use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;

#[test]
fn test_parser_new() {
    let document = "Hello";
    let parser = Parser::new(document);
    assert_eq!(parser.get_position(), create_pos(0, 1, 1));
    assert_eq!(parser.remain(), "Hello");
}
