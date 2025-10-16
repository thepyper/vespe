use crate::ast2::*;
use crate::ast2::tests::utils::*;
use anyhow::Result;
use serde_json::json;

#[test]
fn test_try_parse_enclosed_value_single_quote() -> Result<()> {
    let mut parser = Parser::new("'hello world'" );
    let value = _try_parse_enclosed_value(&mut parser, '\'')?;
    assert_eq!(value, Some("hello world".to_string()));
    assert_eq!(parser.get_position(), create_pos(13, 1, 14));
    Ok(())
}
