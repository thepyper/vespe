use crate::ast2::*;
use crate::ast2::tests::utils::*;
use anyhow::Result;

#[test]
fn test_try_parse_command_kind() -> Result<()> {
    let mut parser = Parser::new("answer");
    assert!(matches!(_try_parse_command_kind(&mut parser)?,
              Some(CommandKind::Answer)));
    assert_eq!(parser.get_position(), create_pos(6, 1, 7));
    Ok(())
}
