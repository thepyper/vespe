use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;

#[test]
fn test_parser_consume_char_if() {
    let document = "123abc";
    let mut parser = Parser::new(document);
    assert_eq!(parser.consume_char_if(|c| c.is_ascii_digit()), Some('1'));
    assert_eq!(parser.get_position(), create_pos(1, 1, 2));
    assert_eq!(parser.consume_char_if(|c| c.is_ascii_digit()), Some('2'));
    assert_eq!(parser.get_position(), create_pos(2, 1, 3));
    assert_eq!(parser.consume_char_if(|c| c.is_ascii_alphabetic()), None);
    assert_eq!(parser.get_position(), create_pos(2, 1, 3));
}
