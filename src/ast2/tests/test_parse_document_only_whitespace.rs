use crate::ast2::tests::utils::*;
use crate::ast2::*;
use anyhow::Result;

#[test]
fn test_parse_document_only_whitespace() -> Result<()> {
    let document_str = "   \n\t ";
    let document = parse_document(document_str)?;
    assert!(document.content.is_empty());
    assert_eq!(document.range, create_range(0, 1, 1, 6, 2, 3));
    Ok(())
}
