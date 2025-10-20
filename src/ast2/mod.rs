pub mod types;
pub(crate) mod parser;
pub(crate) mod parse_utils;
pub(crate) mod parse_elements;

use types::Document;
use parser::Parser;
use parse_elements::parse_content;

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

#[cfg(test)]
mod tests {
    #[path = "../tests/utils.rs"]
    mod utils;
    #[path = "../tests/test_position_range.rs"]
    mod test_position_range;
    #[path = "../tests/test_parser_advance.rs"]
    mod test_parser_advance;
    #[path = "../tests/test_parser_consume.rs"]
    mod test_parser_consume;
    #[path = "../tests/test_parse_identifier.rs"]
    mod test_parse_identifier;
    #[path = "../tests/test_parse_nude_values.rs"]
    mod test_parse_nude_values;
    #[path = "../tests/test_parse_enclosed_values.rs"]
    mod test_parse_enclosed_values;
    #[path = "../tests/test_parse_argument.rs"]
    mod test_parse_argument;
    #[path = "../tests/test_parse_arguments.rs"]
    mod test_parse_arguments;
    #[path = "../tests/test_parse_parameters.rs"]
    mod test_parse_parameters;
    #[path = "../tests/test_parse_kinds.rs"]
    mod test_parse_kinds;
    #[path = "../tests/test_parse_uuid.rs"]
    mod test_parse_uuid;
    #[path = "../tests/test_parse_tag.rs"]
    mod test_parse_tag;
    #[path = "../tests/test_parse_anchor.rs"]
    mod test_parse_anchor;
    #[path = "../tests/test_parse_text.rs"]
    mod test_parse_text;
    #[path = "../tests/test_parse_document.rs"]
    mod test_parse_document;
}