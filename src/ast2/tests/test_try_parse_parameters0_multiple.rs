use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;
use serde_json::json;

#[test]
fn test_try_parse_parameters0_multiple() -> Result<()> {
    let mut parser = Parser::new("{key1: \"value1\", key2: 123, key3: true}");
    let params = _try_parse_parameters0(&mut parser)?.unwrap();
    assert_eq!(params.parameters["key1"], json!("value1"));
    assert_eq!(params.parameters["key2"], json!(123));
    assert_eq!(params.parameters["key3"], json!(true));
    assert_eq!(params.range, create_range(0, 1, 1, 39, 1, 40));
    Ok(())
}
