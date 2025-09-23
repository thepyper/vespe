use crate::llm::markup_policy::MarkupPolicy;

pub struct JsonMarkupPolicy;

impl MarkupPolicy for JsonMarkupPolicy {
    fn get_markup_instructions(&self) -> String {
        "Always respond with JSON, enclosed in ```json ... ``` blocks.".to_string()
    }

    fn name(&self) -> &str {
        "json"
    }

    fn get_tool_invocation_preamble(&self) -> String {
        "You have access to the following tools. To call a tool, respond with a JSON object enclosed in a ```json block, like this:\n```json\n{\n  \"tool_code\": {\n    \"name\": \"tool_name\",\n    \"arguments\": { /* JSON arguments */ }\n  }\n}\n```\nEach tool is described below with its name, description, input schema (JSON), and output schema (JSON):".to_string()
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

    fn get_tool_invocation_preamble(&self) -> String {
        "You have access to the following tools. To call a tool, respond with an XML block enclosed in a ```xml block, like this:\n```xml\n<tool_code>\n  <tool_call>\n    <name>tool_name</name>\n    <arguments> <!-- XML arguments --> </arguments>\n  </tool_call>\n</tool_code>\n```\nEach tool is described below with its name, description, input schema (JSON), and output schema (JSON):".to_string()
    }
}
