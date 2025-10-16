use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;

#[test]
fn test_try_parse_tag0_simple() -> Result<()> {
    let mut parser = Parser::new("@answer arg1 arg2");
    let tag = _try_parse_tag0(&mut parser)?.unwrap();
    assert!(matches!(tag.command, CommandKind::Answer));
    assert_eq!(tag.arguments.arguments.len(), 2);
    assert_eq!(tag.arguments.arguments[0].value, "arg1");
    assert_eq!(tag.arguments.arguments[1].value, "arg2");
    assert_eq!(tag.range, create_range(0, 1, 1, 17, 1, 18));
    Ok(())
}
