use crate::ast2::*;
use crate::ast2::tests::utils::*;
use anyhow::Result;
use serde_json::json;
use uuid::Uuid;

#[test]
fn test_try_parse_anchor0_with_parameters_and_args() -> Result<()> {
    let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
    let input = format!("<!-- derive-{}:end {{key: \"value\"}} arg1 arg2 -->\n", uuid_str);
    let mut parser = Parser::new(&input);
    let anchor = _try_parse_anchor0(&mut parser)?.unwrap();
    assert!(matches!(anchor.command, CommandKind::Derive));
    assert_eq!(anchor.uuid.to_string(), uuid_str);
    assert!(matches!(anchor.kind, AnchorKind::End));
    assert_eq!(anchor.parameters.parameters["key"], json!("value"));
    assert_eq!(anchor.arguments.arguments.len(), 2);
    assert_eq!(anchor.arguments.arguments[0].value, "arg1");
    assert_eq!(anchor.arguments.arguments[1].value, "arg2");
    assert_eq!(anchor.range, create_range(0, 1, 1, 74, 1, 75));
    Ok(())
}

