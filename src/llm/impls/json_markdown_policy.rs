use anyhow::Result;
use regex::Regex;
use serde_json;

use crate::llm::markdown_policy::MarkdownPolicy;
use crate::llm::messages::{Message, AssistantContent, ToolCall};
use llm::chat::{ChatMessage as LlmChatMessage, ChatRole, MessageType};

// =============================================================================
// 1. JSON MARKDOWN POLICY (Improved)
// =============================================================================
pub struct JsonMarkdownPolicy;

impl JsonMarkdownPolicy {
    pub fn new() -> Self {
        Self {}
    }

    fn format_tool_output(tool_output: &crate::llm::messages::ToolOutput) -> Result<String> {
        let formatted_result = Self::format_result_by_tool_type(&tool_output.tool_name, &tool_output.output)?;
        
        Ok(format!(
            "```tool_result\n{{\n  \"tool\": \"{}\",\n  \"call_id\": \"{}\",\n  \"result\": {}\n}}\n```",
            tool_output.tool_name,
            tool_output.call_id.as_deref().unwrap_or("unknown"),
            formatted_result
        ))
    }

    fn format_result_by_tool_type(tool_name: &str, result: &serde_json::Value) -> Result<String> {
        match tool_name {
            "web_search" => {
                // Estrai e formatta risultati di ricerca in modo leggibile
                if let Some(results) = result.get("results").and_then(|r| r.as_array()) {
                    let formatted_results: Vec<String> = results.iter()
                        .take(3) // Limita a primi 3 risultati
                        .filter_map(|r| {
                            let title = r.get("title")?.as_str()?;
                            let snippet = r.get("snippet")?.as_str()?;
                            Some(format!("- **{}**: {}", title, snippet))
                        })
                        .collect();
                    Ok(serde_json::to_value(formatted_results.join("\\n"))?)
                } else {
                    Ok(result.clone())
                }
            },
            "code_execution" => {
                // Formatta output di codice con stdout/stderr separati
                if let Some(stdout) = result.get("stdout") {
                    let stderr = result.get("stderr").unwrap_or(&serde_json::Value::Null);
                    let formatted = serde_json::json!({
                        "stdout": stdout,
                        "stderr": stderr,
                        "formatted": format!("Output: {}", stdout.as_str().unwrap_or(""))
                    });
                    Ok(formatted)
                } else {
                    Ok(result.clone())
                }
            },
            "file_read" => {
                // Tronca file lunghi
                if let Some(content) = result.get("content").and_then(|c| c.as_str()) {
                    let truncated = if content.len() > 1000 {
                        format!("{}... [truncated, {} chars total]", &content[..1000], content.len())
                    } else {
                        content.to_string()
                    };
                    Ok(serde_json::to_value(truncated)?)
                } else {
                    Ok(result.clone())
                }
            },
            _ => Ok(result.clone()) // Default: passa il JSON così com'è
        }
    }
}

impl MarkdownPolicy for JsonMarkdownPolicy {
    fn markdown_format_instructions(&self) -> String {
        r#"RESPONSE FORMAT REQUIREMENTS:

You MUST structure your response using these EXACT formats:

1. TOOL CALLS - Use this format when calling tools:
```tool_call
{
  "name": "tool_name",
  "arguments": {
    "param1": "value1",
    "param2": "value2"
  }
}
```

2. INTERNAL REASONING - Use this format for your thinking process:
```reasoning
{
  "thoughts": "Step-by-step analysis of what I need to do and why",
  "approach": "My strategy for solving this problem"
}
```

3. CODE BLOCKS - Use standard markdown for code:
```python
# Your code here
def example():
    return "hello"
```

4. PLAIN TEXT - Use normal text for responses to the user.

IMPORTANT RULES:
- Each JSON block must be valid JSON
- Do not nest code blocks
- Separate different types of content clearly
- Tool calls and reasoning blocks are processed specially

Example response structure:
```reasoning
{
  "thoughts": "I need to search for information about X",
  "approach": "I'll use the web search tool first"
}
```

```tool_call
{
  "name": "web_search", 
  "arguments": {"query": "example search"}
}
```

Based on the search results, here's what I found...

```python
def solution():
    return "implementation"
```

This code demonstrates the solution because..."#.to_string()
    }

    fn parse_response(&self, response: &str) -> Result<Vec<AssistantContent>> {
        let mut parsed_content = Vec::new();
        let mut last_index = 0;

        // Combined regex to find all structured blocks in order
        let combined_re = Regex::new(
            r"```(tool_call|reasoning)\n([\s\S]*?)\n```"
        )?;

        for mat in combined_re.find_iter(response) {
            let start = mat.start();
            let end = mat.end();

            // Add any preceding plain text
            if start > last_index {
                let text = response[last_index..start].trim();
                if !text.is_empty() {
                    parsed_content.push(AssistantContent::Text(text.to_string()));
                }
            }

            // Extract block type and content
            let full_match = mat.as_str();
            let captures = combined_re.captures(full_match).unwrap();
            let block_type = captures.get(1).unwrap().as_str();
            let json_content = captures.get(2).unwrap().as_str();

            match block_type {
                "tool_call" => {
                    let tool_call: ToolCall = serde_json::from_str(json_content)
                        .map_err(|e| anyhow::anyhow!("Invalid tool_call JSON: {} in: {}", e, json_content))?;
                    parsed_content.push(AssistantContent::ToolCall(tool_call));
                }
                "reasoning" => {
                    #[derive(serde::Deserialize)]
                    struct ReasoningBlock {
                        thoughts: String,
                        #[serde(default)]
                        approach: String,
                    }
                    
                    let reasoning: ReasoningBlock = serde_json::from_str(json_content)
                        .map_err(|e| anyhow::anyhow!("Invalid reasoning JSON: {} in: {}", e, json_content))?;
                    
                    let combined_thought = if reasoning.approach.is_empty() {
                        reasoning.thoughts
                    } else {
                        format!("{}\nApproach: {}", reasoning.thoughts, reasoning.approach)
                    };
                    parsed_content.push(AssistantContent::Thought(combined_thought));
                }
                _ => {}
            }

            last_index = end;
        }

        // Add any remaining plain text
        if last_index < response.len() {
            let text = response[last_index..].trim();
            if !text.is_empty() {
                parsed_content.push(AssistantContent::Text(text.to_string()));
            }
        }

        // Fallback: if no structured content found, treat as plain text
        if parsed_content.is_empty() && !response.trim().is_empty() {
            parsed_content.push(AssistantContent::Text(response.to_string()));
        }

        Ok(parsed_content)
    }

    fn format_query(&self, messages: &[Message]) -> Result<Vec<LlmChatMessage>> {
        let mut llm_chat_messages = Vec::new();

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
                                assistant_content.push('\n');
                            },
                            AssistantContent::Thought(thought) => {
                                assistant_content.push_str(&format!(
                                    "```reasoning\n{{\n  \"thoughts\": \"{}\"\n}}\n```\n",
                                    thought.replace('"', "\\\"")
                                ));
                            },
                            AssistantContent::ToolCall(tool_call) => {
                                assistant_content.push_str(&format!(
                                    "```tool_call\n{{\n  \"name\": \"{}\",\n  \"arguments\": {}\n}}\n```\n",
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
                    let formatted_output = Self::format_tool_output(tool_output)?;
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
}

// =============================================================================
// 2. XML MARKDOWN POLICY
// =============================================================================
pub struct XmlMarkdownPolicy;

impl XmlMarkdownPolicy {
    pub fn new() -> Self {
        Self {}
    }

    fn format_tool_output(tool_output: &crate::llm::messages::ToolOutput) -> Result<String> {
        let formatted_result = Self::format_result_by_tool_type(&tool_output.tool_name, &tool_output.output)?;
        
        Ok(format!(
            "<tool_result>\n<tool_name>{}</tool_name>\n<call_id>{}</call_id>\n<result>\n{}\n</result>\n</tool_result>",
            tool_output.tool_name,
            tool_output.call_id.as_deref().unwrap_or("unknown"),
            serde_json::to_string_pretty(&formatted_result)?
        ))
    }

    fn format_result_by_tool_type(tool_name: &str, result: &serde_json::Value) -> Result<serde_json::Value> {
        // Stessa logica del JSON policy ma ritorna Value per consistency
        match tool_name {
            "web_search" => {
                if let Some(results) = result.get("results").and_then(|r| r.as_array()) {
                    let formatted: Vec<serde_json::Value> = results.iter()
                        .take(3)
                        .filter_map(|r| {
                            let title = r.get("title")?.as_str()?;
                            let snippet = r.get("snippet")?.as_str()?;
                            Some(serde_json::json!({
                                "title": title,
                                "snippet": snippet,
                                "display": format!("{}: {}", title, snippet)
                            }))
                        })
                        .collect();
                    Ok(serde_json::json!({"search_results": formatted}))
                } else {
                    Ok(result.clone())
                }
            },
            "code_execution" => {
                if let Some(stdout) = result.get("stdout") {
                    Ok(serde_json::json!({
                        "execution_output": stdout,
                        "errors": result.get("stderr"),
                        "status": "completed"
                    }))
                } else {
                    Ok(result.clone())
                }
            },
            _ => Ok(result.clone())
        }
    }
}

impl MarkdownPolicy for XmlMarkdownPolicy {
    fn markdown_format_instructions(&self) -> String {
        r#"RESPONSE FORMAT REQUIREMENTS:

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

This function demonstrates recursive implementation..."#.to_string()
    }

    fn parse_response(&self, response: &str) -> Result<Vec<AssistantContent>> {
        let mut parsed_content = Vec::new();
        let mut current_text = response.to_string();

        // Parse <thinking> blocks
        let thinking_re = Regex::new(r"<thinking>\s*([\s\S]*?)\s*</thinking>")?;
        for cap in thinking_re.captures_iter(response) {
            let thought_content = cap.get(1).unwrap().as_str().trim();
            parsed_content.push(AssistantContent::Thought(thought_content.to_string()));
            
            // Remove from current_text for processing remaining content
            current_text = current_text.replace(cap.get(0).unwrap().as_str(), "THINKING_PLACEHOLDER");
        }

        // Parse <tool_call> blocks
        let tool_call_re = Regex::new(r"<tool_call>\s*<name>\s*(.*?)\s*</name>\s*<arguments>\s*([\s\S]*?)\s*</arguments>\s*</tool_call>")?;
        for cap in tool_call_re.captures_iter(response) {
            let tool_name = cap.get(1).unwrap().as_str().trim();
            let args_json = cap.get(2).unwrap().as_str().trim();
            
            let arguments: serde_json::Value = serde_json::from_str(args_json)
                .map_err(|e| anyhow::anyhow!("Invalid tool arguments JSON: {} in: {}", e, args_json))?;
            
            let tool_call = ToolCall {
                name: tool_name.to_string(),
                arguments,
            };
            
            parsed_content.push(AssistantContent::ToolCall(tool_call));
            
            // Remove from current_text
            current_text = current_text.replace(cap.get(0).unwrap().as_str(), "TOOL_CALL_PLACEHOLDER");
        }

        // Process remaining text (remove placeholders and extract actual text)
        current_text = current_text
            .replace("THINKING_PLACEHOLDER", "")
            .replace("TOOL_CALL_PLACEHOLDER", "");
        
        let remaining_text = current_text.trim();
        if !remaining_text.is_empty() {
            parsed_content.push(AssistantContent::Text(remaining_text.to_string()));
        }

        // If no structured content was found, treat entire response as text
        if parsed_content.is_empty() && !response.trim().is_empty() {
            parsed_content.push(AssistantContent::Text(response.to_string()));
        }

        Ok(parsed_content)
    }

    fn format_query(&self, messages: &[Message]) -> Result<Vec<LlmChatMessage>> {
        let mut llm_chat_messages = Vec::new();

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
                    llm_chat_messages.push(LlmChatMessage {
                        role: ChatRole::User,
                        content: format!("Tool output: {}", serde_json::to_string(&tool_output.output)?),
                        message_type: MessageType::Text,
                    });
                },
            }
        }
        Ok(llm_chat_messages)
    }
}

// =============================================================================
// 3. SECTIONS MARKDOWN POLICY
// =============================================================================
pub struct SectionsMarkdownPolicy;

impl SectionsMarkdownPolicy {
    pub fn new() -> Self {
        Self {}
    }

    fn format_tool_output(tool_output: &crate::llm::messages::ToolOutput) -> Result<String> {
        let formatted_result = Self::format_result_by_tool_type(&tool_output.tool_name, &tool_output.output)?;
        
        Ok(format!(
            "## 🔧 Tool Result: {}\n**Call ID:** {}\n**Output:**\n```json\n{}\n```",
            tool_output.tool_name,
            tool_output.call_id.as_deref().unwrap_or("unknown"),
            serde_json::to_string_pretty(&formatted_result)?
        ))
    }

    fn format_result_by_tool_type(tool_name: &str, result: &serde_json::Value) -> Result<serde_json::Value> {
        match tool_name {
            "web_search" => {
                if let Some(results) = result.get("results").and_then(|r| r.as_array()) {
                    let summary = format!("Found {} results", results.len());
                    let top_results: Vec<_> = results.iter()
                        .take(3)
                        .filter_map(|r| {
                            Some(format!("• {}: {}", 
                                r.get("title")?.as_str()?,
                                r.get("snippet")?.as_str()?.chars().take(100).collect::<String>()
                            ))
                        })
                        .collect();
                    
                    Ok(serde_json::json!({
                        "summary": summary,
                        "top_results": top_results,
                        "total_found": results.len()
                    }))
                } else {
                    Ok(result.clone())
                }
            },
            "code_execution" => {
                let status = if result.get("stderr").is_some() && 
                    !result.get("stderr").unwrap().as_str().unwrap_or("").is_empty() {
                    "⚠️ Completed with warnings"
                } else {
                    "✅ Completed successfully"
                };
                
                Ok(serde_json::json!({
                    "status": status,
                    "output": result.get("stdout"),
                    "errors": result.get("stderr")
                }))
            },
            _ => Ok(result.clone())
        }
    }
}

impl MarkdownPolicy for SectionsMarkdownPolicy {
    fn markdown_format_instructions(&self) -> String {
        r#"RESPONSE FORMAT REQUIREMENTS:

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
```"#.to_string()
    }

    fn parse_response(&self, response: &str) -> Result<Vec<AssistantContent>> {
        let mut parsed_content = Vec::new();
        
        // Define section patterns
        let reasoning_re = Regex::new(r"## 🤔 Reasoning\s*\n([\s\S]*?)(?=\n## |$)")?;
        let tool_calls_re = Regex::new(r"## 🔧 Tool Calls\s*\n```json\s*\n([\s\S]*?)\n```")?;
        let response_re = Regex::new(r"## 💬 Response\s*\n([\s\S]*?)(?=\n## |$)")?;
        let code_re = Regex::new(r"## 💻 Code\s*\n([\s\S]*?)(?=\n## |$)")?;

        let mut found_structured_content = false;

        // Parse reasoning sections
        for cap in reasoning_re.captures_iter(response) {
            let reasoning_content = cap.get(1).unwrap().as_str().trim();
            if !reasoning_content.is_empty() {
                parsed_content.push(AssistantContent::Thought(reasoning_content.to_string()));
                found_structured_content = true;
            }
        }

        // Parse tool calls
        for cap in tool_calls_re.captures_iter(response) {
            let json_content = cap.get(1).unwrap().as_str().trim();
            let tool_call: ToolCall = serde_json::from_str(json_content)
                .map_err(|e| anyhow::anyhow!("Invalid tool call JSON: {} in: {}", e, json_content))?;
            
            parsed_content.push(AssistantContent::ToolCall(tool_call));
            found_structured_content = true;
        }

        // Parse response sections
        for cap in response_re.captures_iter(response) {
            let response_content = cap.get(1).unwrap().as_str().trim();
            if !response_content.is_empty() {
                parsed_content.push(AssistantContent::Text(response_content.to_string()));
                found_structured_content = true;
            }
        }

        // Parse code sections (treat as regular text since it contains markdown)
        for cap in code_re.captures_iter(response) {
            let code_content = cap.get(1).unwrap().as_str().trim();
            if !code_content.is_empty() {
                // Prepend a label to distinguish code sections
                let formatted_code = format!("Code:\n{}", code_content);
                parsed_content.push(AssistantContent::Text(formatted_code));
                found_structured_content = true;
            }
        }

        // If no structured sections found, check for any content outside sections
        if !found_structured_content {
            // Look for content that's not part of any section
            let section_header_re = Regex::new(r"## [🤔🔧💬💻] ")?;
            let parts: Vec<&str> = section_header_re.split(response).collect();
            
            if let Some(intro_text) = parts.first() {
                let intro = intro_text.trim();
                if !intro.is_empty() {
                    parsed_content.push(AssistantContent::Text(intro.to_string()));
                }
            }
            
            // If still empty, treat entire response as text
            if parsed_content.is_empty() && !response.trim().is_empty() {
                parsed_content.push(AssistantContent::Text(response.to_string()));
            }
        }

        Ok(parsed_content)
    }

    fn format_query(&self, messages: &[Message]) -> Result<Vec<LlmChatMessage>> {
        let mut llm_chat_messages = Vec::new();

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
                    let mut has_reasoning = false;
                    let mut has_tools = false;
                    let mut has_response = false;
                    let mut has_code = false;

                    // First pass: check what content types we have
                    for part in content_parts {
                        match part {
                            AssistantContent::Thought(_) => has_reasoning = true,
                            AssistantContent::ToolCall(_) => has_tools = true,
                            AssistantContent::Text(text) => {
                                if text.starts_with("Code:") {
                                    has_code = true;
                                } else {
                                    has_response = true;
                                }
                            },
                        }
                    }

                    // Second pass: format in logical order
                    // 1. Reasoning first
                    if has_reasoning {
                        assistant_content.push_str("## 🤔 Reasoning\n");
                        for part in content_parts {
                            if let AssistantContent::Thought(thought) = part {
                                assistant_content.push_str(thought);
                                assistant_content.push_str("\n\n");
                            }
                        }
                    }

                    // 2. Tool calls second
                    if has_tools {
                        assistant_content.push_str("## 🔧 Tool Calls\n");
                        for part in content_parts {
                            if let AssistantContent::ToolCall(tool_call) = part {
                                assistant_content.push_str("```json\n");
                                assistant_content.push_str(&serde_json::to_string_pretty(&serde_json::json!({
                                    "name": tool_call.name,
                                    "arguments": tool_call.arguments
                                }))?);
                                assistant_content.push_str("\n```\n\n");
                            }
                        }
                    }

                    // 3. Response content
                    if has_response {
                        assistant_content.push_str("## 💬 Response\n");
                        for part in content_parts {
                            if let AssistantContent::Text(text) = part {
                                if !text.starts_with("Code:") {
                                    assistant_content.push_str(text);
                                    assistant_content.push_str("\n\n");
                                }
                            }
                        }
                    }

                    // 4. Code sections last
                    if has_code {
                        assistant_content.push_str("## 💻 Code\n");
                        for part in content_parts {
                            if let AssistantContent::Text(text) = part {
                                if text.starts_with("Code:") {
                                    // Remove the "Code:" prefix we added during parsing
                                    let code_content = text.strip_prefix("Code:\n").unwrap_or(text);
                                    assistant_content.push_str(code_content);
                                    assistant_content.push_str("\n\n");
                                }
                            }
                        }
                    }

                    llm_chat_messages.push(LlmChatMessage {
                        role: ChatRole::Assistant,
                        content: assistant_content.trim().to_string(),
                        message_type: MessageType::Text,
                    });
                },
                Message::Tool(tool_output) => {
                    let formatted_output = Self::format_tool_output(tool_output)?;
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
}