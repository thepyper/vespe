use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;
use uuid::Uuid;

#[test]
fn test_try_parse_uuid() -> Result<()> {
    let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
    let mut parser = Parser::new(uuid_str);
    let uuid = _try_parse_uuid(&mut parser)?.unwrap();
    assert_eq!(uuid.to_string(), uuid_str);
    assert_eq!(parser.get_position(), create_pos(36, 1, 37));

    let mut parser = Parser::new("invalid-uuid");
    let result = _try_parse_uuid(&mut parser);
    assert!(
        matches!(result, Err(e) if e.downcast_ref::<ParsingError>().unwrap().to_string().contains("Invalid UUID format"))
    );
    Ok(())
}
