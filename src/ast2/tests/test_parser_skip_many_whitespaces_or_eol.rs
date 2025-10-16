use crate::ast2::*;
use crate::ast2::tests::utils::*;
use anyhow::Result;

#[test]
fn test_parser_skip_many_whitespaces_or_eol() {
    let document = "  \n\r\thello";
    let mut parser = Parser::new(document);
    parser.skip_many_whitespaces_or_eol();
    assert_eq!(parser.get_position(), create_pos(5, 2, 2));
    assert_eq!(parser.remain(), "hello");
}

