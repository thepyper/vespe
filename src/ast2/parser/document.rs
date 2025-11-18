
use super::parser::Parser;
use super::content::parse_content;
use super::super::{Result, Document, Range};

/// Parses a string slice into a `Document` AST.
///
/// This is the main entry point for the parser.
///
/// # Arguments
///
/// * `document` - A string slice representing the source document to be parsed.
///
/// # Returns
///
/// A `Result` containing the parsed `Document` or an `Ast2Error` on failure.
pub fn parse_document(document: &str) -> Result<Document> {
    let parser = Parser::new(document);
    let begin = parser.get_position();
    let (content, parser_after_content) = parse_content(parser)?;
    let end = parser_after_content.get_position();

    Ok(Document {
        content: content,
        range: Range { begin, end },
    })
}

