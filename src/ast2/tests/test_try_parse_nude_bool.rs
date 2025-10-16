use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;

#[test]
fn test_try_parse_nude_bool() -> Result<()> {
    let mut parser = Parser::new("true");
    assert_eq!(_try_parse_nude_bool(&mut parser)?, Some(true));
    assert_eq!(parser.get_position(), create_pos(4, 1, 5));

    let mut parser = Parser::new("false");
    assert_eq!(_try_parse_nude_bool(&mut parser)?, Some(false));
    assert_eq!(parser.get_position(), create_pos(5, 1, 6));

    let mut parser = Parser::new("other");
    assert_eq!(_try_parse_nude_bool(&mut parser)?, None);
    assert_eq!(parser.get_position(), create_pos(0, 1, 1));
    Ok(())
}
