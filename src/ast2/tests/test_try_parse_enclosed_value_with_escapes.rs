use crate::ast2::*;
use crate::ast2::tests::utils::*;
use anyhow::Result;
use serde_json::json;

#[test]
fn test_try_parse_enclosed_value_with_escapes() -> Result<()> {
    let mut parser = Parser::new("\"hello\nworld\t\"\"");
    let value = _try_parse_enclosed_value(&mut parser, "\"")?;
    assert_eq!(value, Some(json!("hello\nworld\t\"")));
    assert_eq!(parser.get_position(), create_pos(18, 1, 19));
    Ok(())
}

