use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;

#[test]
fn test_parser_advance_newline() {
    let document = "H\nello";
    let mut parser = Parser::new(document);
    assert_eq!(parser.advance(), Some('H'));
    assert_eq!(parser.get_position(), create_pos(1, 1, 2));
    assert_eq!(parser.advance(), Some('\n'));
    assert_eq!(parser.get_position(), create_pos(2, 2, 1));
}
