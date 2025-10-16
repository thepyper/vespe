use crate::ast2::*;
use crate::ast2::tests::utils::*;
use anyhow::Result;

#[test]
fn test_try_parse_parameters0_missing_value() -> Result<()> {
    let mut parser = Parser::new("{key: }");
    let result = _try_parse_parameters0(&mut parser);
    assert!(matches!(result, Err(e) if e.downcast_ref::<ParsingError>().unwrap().to_string().contains("Expected parameter value")));
    Ok(())
}
