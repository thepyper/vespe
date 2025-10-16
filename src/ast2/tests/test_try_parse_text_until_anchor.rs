use crate::ast2::*;
use crate::ast2::tests::utils::*;
use anyhow::Result;

#[test]
fn test_try_parse_text_until_anchor() -> Result<()> {
    let mut parser = Parser::new("Text before anchor.\n<!-- anchor -->\n");
    let text = _try_parse_text(&mut parser)?.unwrap();
    assert_eq!(text.range, create_range(0, 1, 1, 20, 2, 1));
    assert_eq!(parser.remain(), "<!-- anchor -->\n");
    Ok(())
}

