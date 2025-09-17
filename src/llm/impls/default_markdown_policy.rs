use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::llm::markdown_policy::MarkdownPolicy;
use crate::llm::messages::{Message, AssistantContent, ToolCall, ToolOutput};
use crate::config::MalformedJsonHandling;
use crate::agent::core::text_utils::trim_markdown_code_blocks;
use crate::llm::models::ChatMessage as LlmChatMessage;
use llm::chat::{ChatRole, MessageType};

pub struct DefaultMarkdownPolicy;

impl DefaultMarkdownPolicy {
    pub fn new() -> Self {
        Self {}
    }
}

impl MarkdownPolicy for DefaultMarkdownPolicy {
    fn markdown_format_instructions(&self) -> String {
        r#"Your responses should be formatted using markdown. 
When you need to call a tool, use a JSON code block with the language identifier `tool_code`.
Example:
```tool_code
{
  "name": "tool_name",
  "arguments": {
    "arg1": "value1"
  }
}
```
If you have an internal thought, use a JSON code block with the language identifier `thought`.
Example:
```thought
{
  "reasoning": "I need to do X because Y"
}
```
All other content should be plain text.
"#.to_string()
    }

    fn parse_response(&self, response: &str) -> Result<Vec<AssistantContent>> {
        // This will be implemented in a later step, replicating ResponseParser logic
        Err(anyhow!("Not yet implemented"))
    }

    fn format_query(&self, messages: &[Message]) -> Result<Vec<LlmChatMessage>> {
        // This will be implemented in a later step, replicating ChatMessage mapping logic
        Err(anyhow!("Not yet implemented"))
    }
}
