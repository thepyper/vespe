use crate::ast2::*;
use crate::ast2::tests::utils::*;
use anyhow::Result;

#[test]
fn test_parse_document_error() -> Result<()> {
    let document_str = "@invalid-tag";
    let result = parse_document(document_str);
    assert!(matches!(result, Err(e) if e.downcast_ref::<ParsingError>().unwrap().to_string().contains("Expected command kind after @")));
    Ok(())
}
