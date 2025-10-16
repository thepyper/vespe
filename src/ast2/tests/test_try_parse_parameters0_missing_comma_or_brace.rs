use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;

#[test]
fn test_try_parse_parameters0_missing_comma_or_brace() -> Result<()> {
    let mut parser = Parser::new("{key1: \"value1\" key2: 123}");
    let result = _try_parse_parameters0(&mut parser);
    assert!(
        matches!(result, Err(e) if e.downcast_ref::<ParsingError>().unwrap().to_string().contains("Expected , or } "))
    );
    Ok(())
}
