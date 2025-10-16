use crate::ast2::*;
use crate::ast2::tests::utils::*;
use anyhow::Result;

#[test]
fn test_try_parse_argument_unquoted() -> Result<()> {
    let mut parser = Parser::new("hello_world-1.0/path");
    let arg = _try_parse_argument(&mut parser)?.unwrap();
    assert_eq!(arg.value, "hello_world-1.0/path");
    assert_eq!(arg.range, create_range(0, 1, 1, 20, 1, 21));
    Ok(())
}
