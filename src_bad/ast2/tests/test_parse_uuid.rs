use crate::ast2::error::Ast2Error;
use crate::ast2::parser::Parser;
use crate::ast2::parser::values;
use uuid::Uuid;

#[test]
fn test_try_parse_uuid_valid() {
    let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
    let doc = format!("{} rest", uuid_str);
    let parser = Parser::new(&doc);
    let (uuid, p_next) = values::_try_parse_uuid(&parser).unwrap().unwrap();
    assert_eq!(uuid, Uuid::parse_str(uuid_str).unwrap());
    assert_eq!(p_next.remain(), " rest");
}

#[test]
fn test_try_parse_uuid_invalid_format() {
    let parser = Parser::new("invalid-uuid rest");
    let result = values::_try_parse_uuid(&parser);
    assert!(matches!(result, Err(Ast2Error::InvalidUuid { .. })));
}

#[test]
fn test_try_parse_uuid_partial() {
    let parser = Parser::new("123e4567-e89b-12d3-a456-42661417400 rest");
    let result = values::_try_parse_uuid(&parser);
    assert!(matches!(result, Err(Ast2Error::InvalidUuid { .. })));
}

#[test]
fn test_try_parse_uuid_empty() {
    let parser = Parser::new(" rest");
    let result = values::_try_parse_uuid(&parser);
    assert!(matches!(result, Err(Ast2Error::InvalidUuid { .. })));
}
