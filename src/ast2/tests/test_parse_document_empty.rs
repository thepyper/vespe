use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;

#[test]
fn test_parse_document_empty() -> Result<()> {
    let document_str = "";
    let document = parse_document(document_str)?;
    assert!(document.content.is_empty());
    assert_eq!(document.range, create_range(0, 1, 1, 0, 1, 1));
    Ok(())
}
