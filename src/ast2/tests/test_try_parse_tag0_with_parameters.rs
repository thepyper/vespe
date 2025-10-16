use crate::ast2::*;
use crate::ast2::tests::utils::*;
use anyhow::Result;
use serde_json::json;

#[test]
fn test_try_parse_tag0_with_parameters() -> Result<()> {
    let mut parser = Parser::new("@inline {key: \"value\"} arg1\n");
    let tag = _try_parse_tag0(&mut parser)?.unwrap();
    assert!(matches!(tag.command, CommandKind::Inline));
    assert_eq!(tag.parameters.parameters["key"], json!("value"));
    assert_eq!(tag.arguments.arguments.len(), 1);
    assert_eq!(tag.arguments.arguments[0].value, "arg1");
    assert_eq!(tag.range, create_range(0, 1, 1, 29, 1, 30));
    Ok(())
}

