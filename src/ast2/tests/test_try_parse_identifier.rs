use crate::ast2::*;
use crate::ast2::tests::utils::*;
use anyhow::Result;

#[test]
fn test_try_parse_identifier() -> Result<()> {
    let mut parser = Parser::new("my_variable123");
    assert_eq!(_try_parse_identifier(&mut parser)?,
               Some("my_variable123".to_string()));
    assert_eq!(parser.get_position(), create_pos(14, 1, 15));

    let mut parser = Parser::new("123variable");
    assert_eq!(_try_parse_identifier(&mut parser)?,
               None);
    assert_eq!(parser.get_position(), create_pos(0, 1, 1));
    Ok(())
}
