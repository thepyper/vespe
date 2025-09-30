use genai::{Client, ChatRequest, GenerationConfig, Content, Part, Role, Tool as GenaiTool, FunctionDeclaration, Schema, Type};
use crate::agent::{LLMProviderConfig, AIConfig};
use crate::memory::{Message, MessageContent};
use crate::tool::ToolConfig;
use crate::error::ProjectError;
use std::collections::HashMap;

// Helper function to create a genai client from LLMProviderConfig
pub fn create_genai_client(llm_config: &LLMProviderConfig) -> Result<Client, ProjectError> {
    match llm_config {
        LLMProviderConfig::Ollama { model: _, endpoint } => {
            Ok(Client::ollama().base_url(endpoint.clone()).build()?)
        },
        LLMProviderConfig::OpenAI { model: _, api_key_env } => {
            let api_key = std::env::var(api_key_env)
                .map_err(|_| ProjectError::MissingApiKey(api_key_env.clone()))?;
            Ok(Client::openai().api_key(api_key).build()?)
        },
        LLMProviderConfig::Gemini { model: _ } => {
            // Gemini client typically uses GOOGLE_API_KEY env var by default
            Ok(Client::gemini().build()?)
        },
    }
}

// Helper function to convert our Message to genai::Content
fn to_genai_content(messages: &[Message]) -> Vec<Content> {
    messages.iter().map(|msg| {
        let role = match msg.author_agent_uid.as_str() {
            "user" => Role::User,
            "assistant" => Role::Model,
            _ => Role::Model, // Default for other agents
        };
        let parts = match &msg.content {
            MessageContent::Text(text) => vec![Part::text(text.clone())],
            MessageContent::Thought(thought) => vec![Part::text(format!("Thought: {}", thought))],
            MessageContent::ToolCall { tool_name, call_uid: _, inputs } => {
                // Genai expects FunctionCall for tool calls
                vec![Part::function_call(tool_name.clone(), inputs.clone())]
            },
            MessageContent::ToolResult { tool_name: _, call_uid: _, inputs: _, outputs } => {
                // Genai expects FunctionResponse for tool results
                vec![Part::function_response(outputs.clone())]
            },
        };
        Content { role, parts }
    }).collect()
}

// Helper function to convert our ToolConfig to genai::Tool
fn to_genai_tools(tools: &[ToolConfig]) -> Vec<GenaiTool> {
    tools.iter().map(|tool_config| {
        let parameters_schema = serde_json::from_value(tool_config.schema.clone())
            .unwrap_or_else(|_| Schema { type_: Type::Object, properties: None, required: None });

        GenaiTool::Function(FunctionDeclaration {
            name: tool_config.name.clone(),
            description: Some(tool_config.description.clone()),
            parameters: Some(parameters_schema),
        })
    }).collect()
}

// Main function to send chat request using genai
pub async fn send_chat_request(
    llm_config: &LLMProviderConfig,
    system_instructions: Option<&str>,
    messages: &[Message],
    available_tools: &[ToolConfig],
) -> Result<Vec<Message>, ProjectError> {
    let client = create_genai_client(llm_config)?;

    let mut genai_messages = Vec::new();

    if let Some(instructions) = system_instructions {
        genai_messages.push(Content {
            role: Role::User, // System instructions are often treated as user messages in genai context
            parts: vec![Part::text(instructions.to_string())],
        });
    }

    genai_messages.extend(to_genai_content(messages));

    let genai_tools = to_genai_tools(available_tools);

    let request = ChatRequest {
        model: llm_config.get_model_name().to_string(), // Assuming LLMProviderConfig has a get_model_name method
        contents: genai_messages,
        generation_config: Some(GenerationConfig { ..Default::default() }),
        tools: if genai_tools.is_empty() { None } else { Some(genai_tools) },
        tool_config: None,
    };

    let response = client.chat(request).await?;

    // Convert genai::Content back to our Message format
    let mut parsed_messages = Vec::new();
    if let Some(candidates) = response.candidates {
        for candidate in candidates {
            for part in candidate.content.parts {
                match part {
                    Part::Text(text) => {
                        parsed_messages.push(Message {
                            uid: crate::utils::generate_uid("msg").map_err(|e| ProjectError::GenaiError(e.to_string()))?,
                            timestamp: chrono::Utc::now(),
                            author_agent_uid: "assistant".to_string(),
                            content: MessageContent::Text(text),
                            status: crate::memory::MessageStatus::Enabled,
                        });
                    },
                    Part::FunctionCall(call) => {
                        parsed_messages.push(Message {
                            uid: crate::utils::generate_uid("msg").map_err(|e| ProjectError::GenaiError(e.to_string()))?,
                            timestamp: chrono::Utc::now(),
                            author_agent_uid: "assistant".to_string(),
                            content: MessageContent::ToolCall {
                                tool_name: call.name,
                                call_uid: crate::utils::generate_uid("call").map_err(|e| ProjectError::GenaiError(e.to_string()))?,
                                inputs: call.arguments,
                            },
                            status: crate::memory::MessageStatus::Enabled,
                        });
                    },
                    _ => { /* Ignore other part types for now */ }
                }
            }
        }
    }

    Ok(parsed_messages)
}
