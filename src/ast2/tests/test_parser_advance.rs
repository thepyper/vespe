use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;

#[test]
fn test_parser_advance() {
    let document = "Hello";
    let mut parser = Parser::new(document);
    assert_eq!(parser.advance(), Some('H'));
    assert_eq!(parser.get_position(), create_pos(1, 1, 2));
    assert_eq!(parser.advance(), Some('e'));
    assert_eq!(parser.get_position(), create_pos(2, 1, 3));
    parser.advance();
    parser.advance();
    parser.advance();
    assert_eq!(parser.advance(), None);
    assert_eq!(parser.get_position(), create_pos(5, 1, 6));
}
