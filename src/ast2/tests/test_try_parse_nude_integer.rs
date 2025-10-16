use crate::ast2::*;
use crate::ast2::tests::utils::*;
use anyhow::Result;

#[test]
fn test_try_parse_nude_integer() -> Result<()> {
    let mut parser = Parser::new("123");
    assert_eq!(_try_parse_nude_integer(&mut parser)?,
               Some(123));
    assert_eq!(parser.get_position(), create_pos(3, 1, 4));

    let mut parser = Parser::new("abc");
    assert_eq!(_try_parse_nude_integer(&mut parser)?,
               None);
    assert_eq!(parser.get_position(), create_pos(0, 1, 1));

    let mut parser = Parser::new("");
    assert_eq!(_try_parse_nude_integer(&mut parser)?,
               None);
    assert_eq!(parser.get_position(), create_pos(0, 1, 1));
    Ok(())
}
