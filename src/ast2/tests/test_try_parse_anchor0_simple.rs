use crate::ast2::*;
use crate::ast2::tests::utils::*;
use anyhow::Result;
use uuid::Uuid;

#[test]
fn test_try_parse_anchor0_simple() -> Result<()> {
    let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
    let input = format!("<!-- answer-{}:begin -->\n", uuid_str);
    let mut parser = Parser::new(&input);
    let anchor = _try_parse_anchor0(&mut parser)?.unwrap();
    assert!(matches!(anchor.command, CommandKind::Answer));
    assert_eq!(anchor.uuid.to_string(), uuid_str);
    assert!(matches!(anchor.kind, AnchorKind::Begin));
    assert!(anchor.parameters.parameters.as_object().unwrap().is_empty());
    assert!(anchor.arguments.arguments.is_empty());
    assert_eq!(anchor.range, create_range(0, 1, 1, 49, 1, 50));
    Ok(())
}

