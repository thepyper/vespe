use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;
use serde_json::json;

#[test]
fn test_try_parse_parameters0_single() -> Result<()> {
    let mut parser = Parser::new("{key: \"value\"}");
    let params = _try_parse_parameters0(&mut parser)?.unwrap();
    assert_eq!(params.parameters["key"], json!("value"));
    assert_eq!(params.range, create_range(0, 1, 1, 14, 1, 15));
    Ok(())
}
