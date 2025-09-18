use crate::llm::markup_policy::MarkupPolicy;

pub struct JsonMarkupPolicy;

impl MarkupPolicy for JsonMarkupPolicy {
    fn get_markup_instructions(&self) -> String {
        "Always respond with JSON, enclosed in ```json ... ``` blocks.".to_string()
    }

    fn name(&self) -> &str {
        "json"
    }
}

pub struct XmlMarkupPolicy;

impl MarkupPolicy for XmlMarkupPolicy {
    fn get_markup_instructions(&self) -> String {
        "Always respond with XML, enclosed in ```xml ... ``` blocks, using <tool_code> tags for tool calls.".to_string()
    }

    fn name(&self) -> &str {
        "xml"
    }
}
