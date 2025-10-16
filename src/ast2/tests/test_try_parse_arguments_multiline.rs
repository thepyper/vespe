use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;

#[test]
fn test_try_parse_arguments_multiline() -> Result<()> {
    let mut parser = Parser::new("arg1\narg2");
    let args = _try_parse_arguments(&mut parser)?.unwrap();
    assert_eq!(args.arguments.len(), 1);
    assert_eq!(args.arguments[0].value, "arg1");
    assert_eq!(args.range, create_range(0, 1, 1, 4, 1, 5));
    assert_eq!(parser.get_position(), create_pos(4, 1, 5)); // Should stop before newline
    Ok(())
}
