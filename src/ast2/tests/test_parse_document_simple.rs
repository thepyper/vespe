use crate::ast2::*;
use crate::ast2::tests::utils::*;
use anyhow::Result;

#[test]
fn test_parse_document_simple() -> Result<()> {
    let document_str = "Hello world";
    let document = parse_document(document_str)?;
    assert_eq!(document.content.len(), 1);
    assert_eq!(document.range, create_range(0, 1, 1, 11, 1, 12));
    Ok(())
}
