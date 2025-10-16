use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;

#[test]
fn test_try_parse_argument_quoted() -> Result<()> {
    let mut parser = Parser::new("\"hello world\"");
    let arg = _try_parse_argument(&mut parser)?.unwrap();
    assert_eq!(arg.value, "hello world");
    assert_eq!(arg.range, create_range(0, 1, 1, 13, 1, 14));
    Ok(())
}
