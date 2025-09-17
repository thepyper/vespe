use anyhow::Result;
use regex::Regex;
use serde_json;

use crate::llm::markdown_policy::{MarkdownPolicy};
use crate::llm::messages::{Message, AssistantContent, ToolCall};
use crate::llm::policy_types::PolicyType;
use crate::llm::tool_types::ToolType;
use crate::llm::tool_output_formatters;
use llm::chat::{ChatMessage as LlmChatMessage, ChatRole, MessageType};

pub struct SectionsPolicy;

impl SectionsPolicy {
    pub fn new() -> Self { Self {} }

    fn format_tool_output(tool_output: &crate::llm::messages::ToolOutput, policy_type: PolicyType) -> Result<String> {
        let tool_type = ToolType::from(tool_output.tool_name.as_str());
        let formatted_result = tool_output_formatters::format_tool_output(tool_type, &tool_output.output, policy_type)?;

        Ok(format!(
            "## 🔧 Tool Result: {}\n**Call ID:** {}\n**Output:**\n```json\n{}\n```",
            tool_output.tool_name,
            tool_output.call_id.as_deref().unwrap_or("unknown"),
            formatted_result
        ))
    }
}

impl MarkdownPolicy for SectionsPolicy {
    fn markdown_format_instructions(&self) -> String {
        r###"RESPONSE FORMAT REQUIREMENTS:

        Structure your response using these EXACT section headers:

        1. REASONING SECTION (for internal thoughts):
        ## 🤔 Reasoning
        Your step-by-step analysis and thinking process goes here.
        Explain what you need to do and your approach.

        2. TOOL CALLS SECTION (when you need to call tools):
        ## 🔧 Tool Calls
        ```json
        {
          "name": "tool_name",
          "arguments": {
            "param1": "value1",
            "param2": "value2"
          }
        }
        ```

        3. RESPONSE SECTION (main content for user):
        ## 💬 Response
        Your main response to the user goes here.
        This is what they'll primarily read.

        4. CODE SECTION (when providing code):
        ## 💻 Code
        ```python
        def example_function():
            return "Your code implementation"
        ```

        IMPORTANT RULES:
        - Use exactly these section headers (including emojis)
        - Sections can appear in any order based on what you need
        - Not every section is required for every response
        - JSON in Tool Calls must be valid
        - Regular markdown formatting allowed within sections

        Example structure:
        ## 🤔 Reasoning
        I need to search for Python best practices and then show an example.

        ## 🔧 Tool Calls
        ```json
        {
          "name": "web_search",
          "arguments": {
            "query": "Python function best practices 2024"
          }
        }
        ```

        ## 💬 Response
        Based on current best practices, here's what you should know about Python functions...

        ## 💻 Code
        ```python
        def well_designed_function(param: str) -> str:
            """Clear docstring explaining the function."""
            return f"Processed: {param}"
        ```"###.to_string()
    }

    fn parse_response(&self, response: &str) -> Result<Vec<AssistantContent>> {
        let mut parsed_content = Vec::new();
        let mut last_index = 0;

        // Regex per trovare tutti i blocchi di sezione strutturati in ordine
        let combined_re = Regex::new(
            r"(## 🤔 Reasoning[\s\S]*?(?=\n## |\z))|(
            r"## 🔧 Tool Calls[\s\S]*?\n```json[\s\S]*?\n```)|(
            r"## 💬 Response[\s\S]*?(?=\n## |\z))|(
            r"## 💻 Code[\s\S]*?(?=\n## |\z))"
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

            if full_match.contains("## 🤔 Reasoning") {
                let reasoning_re = Regex::new(r"## 🤔 Reasoning[\s\S]*?\n([\s\S]*)")?;
                if let Some(captures) = reasoning_re.captures(full_match) {
                    let content = captures.get(1).unwrap().as_str().trim();
                    parsed_content.push(AssistantContent::Thought(content.to_string()));
                }
            } else if full_match.contains("## 🔧 Tool Calls") {
                let tool_calls_re = Regex::new(r"## 🔧 Tool Calls[\s\S]*?\n```json[\s\S]*?\n([\s\S]*?)\n```")?;
                if let Some(captures) = tool_calls_re.captures(full_match) {
                    let json_content = captures.get(1).unwrap().as_str().trim();
                    let tool_call: ToolCall = serde_json::from_str(json_content)
                        .map_err(|e| anyhow::anyhow!("Invalid tool call JSON: {} in: {}", e, json_content))?;
                    parsed_content.push(AssistantContent::ToolCall(tool_call));
                }
            } else if full_match.contains("## 💬 Response") {
                let response_re = Regex::new(r"## 💬 Response[\s\S]*?\n([\s\S]*)")?;
                if let Some(captures) = response_re.captures(full_match) {
                    let content = captures.get(1).unwrap().as_str().trim();
                    parsed_content.push(AssistantContent::Text(content.to_string()));
                }
            } else if full_match.contains("## 💻 Code") {
                let code_re = Regex::new(r"## 💻 Code[\s\S]*?\n```[a-zA-Z]*[\s\S]*?\n([\s\S]*?)\n```")?;
                if let Some(captures) = code_re.captures(full_match) {
                    let content = captures.get(1).unwrap().as_str().trim();
                    parsed_content.push(AssistantContent::Text(format!("Code:\n{}", content))); // Mantiene il prefisso per format_query
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

        // Fallback
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

                    // Costruisci il contenuto in un singolo passaggio, mantenendo l'ordine logico
                    let mut reasoning_parts = Vec::new();
                    let mut tool_call_parts = Vec::new();
                    let mut response_parts = Vec::new();
                    let mut code_parts = Vec::new();

                    for part in content_parts {
                        match part {
                            AssistantContent::Thought(thought) => reasoning_parts.push(thought),
                            AssistantContent::ToolCall(tool_call) => tool_call_parts.push(tool_call),
                            AssistantContent::Text(text) => {
                                if text.starts_with("Code:\n") {
                                    code_parts.push(text);
                                } else {
                                    response_parts.push(text);
                                }
                            },
                        }
                    }

                    if !reasoning_parts.is_empty() {
                        assistant_content.push_str("## 🤔 Reasoning\n");
                        for thought in reasoning_parts {
                            assistant_content.push_str(thought);
                            assistant_content.push_str("\n\n");
                        }
                    }

                    if !tool_call_parts.is_empty() {
                        assistant_content.push_str("## 🔧 Tool Calls\n");
                        for tool_call in tool_call_parts {
                            assistant_content.push_str("```json\n");
                            assistant_content.push_str(&serde_json::to_string_pretty(&serde_json::json!({
                                "name": &tool_call.name,
                                "arguments": &tool_call.arguments
                            }))?);
                            assistant_content.push_str("\n```\n\n");
                        }
                    }

                    if !response_parts.is_empty() {
                        assistant_content.push_str("## 💬 Response\n");
                        for text in response_parts {
                            assistant_content.push_str(text);
                            assistant_content.push_str("\n\n");
                        }
                    }

                    if !code_parts.is_empty() {
                        assistant_content.push_str("## 💻 Code\n");
                        for text in code_parts {
                            let code_content = text.strip_prefix("Code:\n").unwrap_or(text);
                            assistant_content.push_str(code_content);
                            assistant_content.push_str("\n\n");
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

    fn get_policy_type(&self) -> PolicyType { PolicyType::Sections }
}
