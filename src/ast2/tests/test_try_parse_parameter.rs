use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;
use serde_json::json;

#[test]
fn test_try_parse_parameter() -> Result<()> {
    let mut parser = Parser::new("key: \"value\"");
    let (key, value) = _try_parse_parameter(&mut parser)?.unwrap();
    assert_eq!(key, "key");
    assert_eq!(value, json!("value"));
    assert_eq!(parser.get_position(), create_pos(12, 1, 13));

    let mut parser = Parser::new("number: 123");
    let (key, value) = _try_parse_parameter(&mut parser)?.unwrap();
    assert_eq!(key, "number");
    assert_eq!(value, json!(123));
    assert_eq!(parser.get_position(), create_pos(11, 1, 12));

    let mut parser = Parser::new("boolean: true");
    let (key, value) = _try_parse_parameter(&mut parser)?.unwrap();
    assert_eq!(key, "boolean");
    assert_eq!(value, json!(true));
    assert_eq!(parser.get_position(), create_pos(13, 1, 14));

    let mut parser = Parser::new("float: 1.23");
    let (key, value) = _try_parse_parameter(&mut parser)?.unwrap();
    assert_eq!(key, "float");
    assert_eq!(value, json!(1.23));
    assert_eq!(parser.get_position(), create_pos(11, 1, 12));
    Ok(())
}
