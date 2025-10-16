use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;

#[test]
fn test_try_parse_enclosed_value_unterminated() -> Result<()> {
    let mut parser = Parser::new("\"hello");
    let result = _try_parse_enclosed_value(&mut parser, '\"');
    assert!(
        matches!(result, Err(e) if e.downcast_ref::<ParsingError>().unwrap().to_string().contains("Unterminated string"))
    );
    Ok(())
}
