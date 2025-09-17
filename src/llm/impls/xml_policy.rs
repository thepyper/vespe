use anyhow::Result;
use regex::Regex;
use serde_json;

use crate::llm::markdown_policy::{MarkdownPolicy};
use crate::llm::messages::{Message, AssistantContent, ToolCall};
use crate::llm::policy_types::PolicyType;
use crate::llm::tool_types::ToolType;
use crate::llm::tool_output_formatters;
use llm::chat::{ChatMessage as LlmChatMessage, ChatRole, MessageType};

pub struct XmlPolicy;

impl XmlPolicy {
    pub fn new() -> Self { Self {} }

    fn format_tool_output(tool_output: &crate::llm::messages::ToolOutput, policy_type: PolicyType) -> Result<String> {
        let tool_type = ToolType::from(tool_output.tool_name.as_str());
        let formatted_result = tool_output_formatters::format_tool_output(tool_type, &tool_output.output, policy_type)?;

        Ok(format!(
            "<tool_result>\n<tool_name>{}</tool_name>\n<call_id>{}</call_id>\n<result>\n{}\n</result>\n</tool_result>",
            tool_output.tool_name,
            tool_output.call_id.as_deref().unwrap_or("unknown"),
            formatted_result
        ))
    }
}

impl MarkdownPolicy for XmlPolicy {
    fn markdown_format_instructions(&self) -> String {
        r###"RESPONSE FORMAT REQUIREMENTS:

You MUST structure your response using these EXACT XML-style tags:

1. INTERNAL REASONING - Wrap your thinking in <thinking> tags:
<thinking>
Your step-by-step reasoning, analysis, and approach goes here.
You can use multiple lines and explain your thought process clearly.
</thinking>

2. TOOL CALLS - Use <tool_call> tags with specific structure:
<tool_call>
<name>tool_name</name>
<arguments>
{
  "param1": "value1",
  "param2": "value2"
}
</arguments>
</tool_call>

3. CODE BLOCKS - Use standard markdown for code:
```python
def example():
    return "code here"
```

4. REGULAR RESPONSE - Use normal text for user-facing responses.

IMPORTANT RULES:
- Always close your XML tags properly
- JSON in <arguments> must be valid
- <thinking> content is for your internal reasoning
- Regular text is what the user sees
- Keep XML tags on separate lines for clarity

Example response:
<thinking>
I need to search for information about Python functions, then provide an example implementation.
</thinking>

<tool_call>
<name>web_search</name>
<arguments>
{
  "query": "Python function best practices"
}
</arguments>
</tool_call>

Based on the search results, here's a great example of Python functions:

```python
def calculate_fibonacci(n):
    if n <= 1:
        return n
    return calculate_fibonacci(n-1) + calculate_fibonacci(n-2)
```

This function demonstrates recursive implementation..."###.to_string()
    }

    fn parse_response(&self, response: &str) -> Result<Vec<AssistantContent>> {
        let mut parsed_content = Vec::new();
        let mut last_index = 0;

        // Regex per trovare tutti i blocchi XML strutturati in ordine
        let combined_re = Regex::new(
            r"(<thinking>[\s\S]*?<\/thinking>)|(<tool_call>[\s\S]*?<\/tool_call>)"
        )?;

        for mat in combined_re.find_iter(response) {
            let start = mat.start();
            let end = mat.end();

            // Aggiungi qualsiasi testo libero precedente
            if start > last_index {
                let text = response[last_index..start].trim();
                if !text.is_empty() {
                    parsed_content.push(AssistantContent::Text(text.to_string()));
                }
            }

            let full_match = mat.as_str();

            if full_match.starts_with("<thinking>") {
                let thinking_re = Regex::new(r"<thinking>([\s\S]*?)<\/thinking>")?;
                if let Some(captures) = thinking_re.captures(full_match) {
                    let thought_content = captures.get(1).unwrap().as_str().trim();
                    parsed_content.push(AssistantContent::Thought(thought_content.to_string()));
                }
            } else if full_match.starts_with("<tool_call>") {
                let tool_call_re = Regex::new(r"<tool_call>\s*<name>\s*(.*?)\s*<\/name>\s*<arguments>\s*([\s\S]*?)\s*<\/arguments>\s*<\/tool_call>")?;
                if let Some(captures) = tool_call_re.captures(full_match) {
                    let tool_name = captures.get(1).unwrap().as_str().trim();
                    let args_json = captures.get(2).unwrap().as_str().trim();

                    let arguments: serde_json::Value = serde_json::from_str(args_json)
                        .map_err(|e| anyhow::anyhow!("Invalid tool arguments JSON: {} in: {}", e, args_json))?;

                    let tool_call = ToolCall {
                        name: tool_name.to_string(),
                        arguments,
                    };
                    parsed_content.push(AssistantContent::ToolCall(tool_call));
                }
            }

            last_index = end;
        }

        // Aggiungi qualsiasi testo libero rimanente
        if last_index < response.len() {
            let text = response[last_index..].trim();
            if !text.is_empty() {
                parsed_content.push(AssistantContent::Text(text.to_string()));
            }
        }

        // Fallback: se nessun contenuto strutturato trovato, tratta l'intera risposta come testo semplice
        if parsed_content.is_empty() && !response.trim().is_empty() {
            parsed_content.push(AssistantContent::Text(response.to_string()));
        }

        Ok(parsed_content)
    }

    fn format_query(&self, messages: &[Message]) -> Result<Vec<LlmChatMessage>> {
        let mut llm_chat_messages = Vec::new();
        let policy_type = self.get_policy_type();

        for msg in messages {
            match msg {
                Message::System(content) => {
                    llm_chat_messages.push(LlmChatMessage {
                        role: ChatRole::User,
                        content: content.clone(),
                        message_type: MessageType::Text,
                    });
                },
                Message::User(content) => {
                    llm_chat_messages.push(LlmChatMessage {
                        role: ChatRole::User,
                        content: content.clone(),
                        message_type: MessageType::Text,
                    });
                },
                Message::Assistant(content_parts) => {
                    let mut assistant_content = String::new();

                    for part in content_parts {
                        match part {
                            AssistantContent::Text(text) => {
                                assistant_content.push_str(text);
                                assistant_content.push_str("\n\n");
                            },
                            AssistantContent::Thought(thought) => {
                                assistant_content.push_str(&format!(
                                    "<thinking>\n{}\n</thinking>\n\n",
                                    thought
                                ));
                            },
                            AssistantContent::ToolCall(tool_call) => {
                                assistant_content.push_str(&format!(
                                    "<tool_call>\n<name>{}</name>\n<arguments>\n{}\n</arguments>\n</tool_call>\n\n",
                                    tool_call.name,
                                    serde_json::to_string_pretty(&tool_call.arguments)?
                                ));
                            },
                        }
                    }

                    llm_chat_messages.push(LlmChatMessage {
                        role: ChatRole::Assistant,
                        content: assistant_content.trim().to_string(),
                        message_type: MessageType::Text,
                    });
                },
                Message::Tool(tool_output) => {
                    let formatted_output = Self::format_tool_output(tool_output, policy_type)?;
                    llm_chat_messages.push(LlmChatMessage {
                        role: ChatRole::User,
                        content: formatted_output,
                        message_type: MessageType::Text,
                    });
                },
            }
        }
        Ok(llm_chat_messages)
    }

    fn get_policy_type(&self) -> PolicyType { PolicyType::Xml }
}
