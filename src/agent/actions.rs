use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub args: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AgentAction {
    #[serde(rename = "tool_call")]
    ToolCall(ToolCall),
    #[serde(rename = "text_response")]
    TextResponse { content: String },
    #[serde(rename = "thought")]
    Thought { content: String },
}
