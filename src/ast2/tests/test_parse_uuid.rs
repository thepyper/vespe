use super::{Ast2Error, Parser};
use uuid::Uuid;

#[test]
fn test_try_parse_uuid_valid() {
    let uuid_str = "123e4567-e89b-12d3-a456-426614174000";
    let doc = format!("{} rest", uuid_str);
    let parser = Parser::new(&doc);
    let (uuid, p_next) = super::_try_parse_uuid(&parser).unwrap().unwrap();
    assert_eq!(uuid, Uuid::parse_str(uuid_str).unwrap());
    assert_eq!(p_next.remain(), " rest");
}

#[test]
fn test_try_parse_uuid_invalid_format() {
    let doc = "invalid-uuid rest";
    let parser = Parser::new(doc);
    let result = super::_try_parse_uuid(&parser);
    assert!(matches!(result, Err(Ast2Error::InvalidUuid { .. })));
}

#[test]
fn test_try_parse_uuid_partial() {
    let doc = "123e4567-e89b rest";
    let parser = Parser::new(doc);
    let result = super::_try_parse_uuid(&parser);
    assert!(matches!(result, Err(Ast2Error::InvalidUuid { .. })));
}

#[test]
fn test_try_parse_uuid_empty() {
    let doc = " rest";
    let parser = Parser::new(doc);
    let result = super::_try_parse_uuid(&parser);
    assert!(matches!(result, Err(Ast2Error::InvalidUuid { .. })));
}
