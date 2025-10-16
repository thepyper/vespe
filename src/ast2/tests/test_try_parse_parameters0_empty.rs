use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;

#[test]
fn test_try_parse_parameters0_empty() -> Result<()> {
    let mut parser = Parser::new("{}");
    let params = _try_parse_parameters0(&mut parser)?.unwrap();
    assert!(params.parameters.as_object().unwrap().is_empty());
    assert_eq!(params.range, create_range(0, 1, 1, 2, 1, 3));
    Ok(())
}
