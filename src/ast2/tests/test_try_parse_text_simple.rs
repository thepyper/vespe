use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;

#[test]
fn test_try_parse_text_simple() -> Result<()> {
    let mut parser = Parser::new("Some text.");
    let text = _try_parse_text(&mut parser)?.unwrap();
    assert_eq!(text.range, create_range(0, 1, 1, 10, 1, 11));
    Ok(())
}
