use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;

#[test]
fn test_try_parse_arguments_multiple() -> Result<()> {
    let mut parser = Parser::new("arg1 \"arg2 with spaces\" arg3");
    let args = _try_parse_arguments(&mut parser)?.unwrap();
    assert_eq!(args.arguments.len(), 3);
    assert_eq!(args.arguments[0].value, "arg1");
    assert_eq!(args.arguments[1].value, "arg2 with spaces");
    assert_eq!(args.arguments[2].value, "arg3");
    assert_eq!(args.range, create_range(0, 1, 1, 30, 1, 31));
    Ok(())
}
