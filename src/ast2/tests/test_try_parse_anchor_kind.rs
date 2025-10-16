use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;

#[test]
fn test_try_parse_anchor_kind() -> Result<()> {
    let mut parser = Parser::new("begin");
    assert!(matches!(
        _try_parse_anchor_kind(&mut parser)?,
        Some(AnchorKind::Begin)
    ));
    assert_eq!(parser.get_position(), create_pos(5, 1, 6));
    Ok(())
}
