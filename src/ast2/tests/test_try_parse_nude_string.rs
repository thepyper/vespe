use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;

#[test]
fn test_try_parse_nude_string() -> Result<()> {
    let mut parser = Parser::new("hello_world-1.0/path");
    assert_eq!(
        _try_parse_nude_string(&mut parser)?,
        Some("hello_world-1.0/path".to_string())
    );
    assert_eq!(parser.get_position(), create_pos(20, 1, 21));

    let mut parser = Parser::new(" hello");
    assert_eq!(_try_parse_nude_string(&mut parser)?, None);
    assert_eq!(parser.get_position(), create_pos(0, 1, 1));
    Ok(())
}
