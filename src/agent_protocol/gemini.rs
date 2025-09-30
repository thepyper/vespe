use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::ProjectError;
use super::{AgentProtocol, AgentResponse};

pub struct GeminiProtocol;

impl GeminiProtocol {
    pub fn new() -> Self {
        GeminiProtocol
    }
}

#[async_trait]
impl AgentProtocol for GeminiProtocol {
    fn format_query(&self, prompt: &str) -> String {
        let request = GeminiRequest {
            contents: vec![
                GeminiContent {
                    parts: vec![
                        GeminiPart {
                            text: prompt.to_string(),
                        },
                    ],
                },
            ],
        };
        serde_json::to_string(&request).unwrap_or_else(|_| "{}".to_string())
    }

    fn parse_response(&self, raw_response: &str) -> Result<AgentResponse, ProjectError> {
        let gemini_response: GeminiResponse = serde_json::from_str(raw_response)
            .map_err(|e| ProjectError::AgentProtocolError(format!("Failed to parse Gemini response: {}", e)))?;

        let mut thought = None;
        let mut tool_code = None;
        let mut response_text = String::new();

        if let Some(candidate) = gemini_response.candidates.into_iter().next() {
            if let Some(part) = candidate.content.parts.into_iter().next() {
                let text = part.text;

                // Regex to extract thought
                let thought_re = regex::Regex::new(r"<thought>(.*?)</thought>").unwrap();
                if let Some(captures) = thought_re.captures(&text) {
                    thought = captures.get(1).map(|m| m.as_str().to_string());
                }

                // Regex to extract tool_code
                let tool_code_re = regex::Regex::new(r"<tool_code>(.*?)</tool_code>").unwrap();
                if let Some(captures) = tool_code_re.captures(&text) {
                    tool_code = captures.get(1).map(|m| m.as_str().to_string());
                }

                // Remove thought and tool_code tags from the text to get the final response
                let cleaned_text = thought_re.replace_all(&text, "").to_string();
                let cleaned_text = tool_code_re.replace_all(&cleaned_text, "").to_string();
                response_text = cleaned_text.trim().to_string();
            }
        }

        Ok(AgentResponse {
            thought,
            tool_code,
            response: response_text,
        })
    }
}

// Define structs for Gemini API request/response serialization/deserialization
// based on doc/gemini_sample_01.txt

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContentResponse,
}

#[derive(Deserialize)]
struct GeminiContentResponse {
    parts: Vec<GeminiPartResponse>,
}

// This struct needs to handle potential 'thought' and 'tool_call' fields
#[derive(Deserialize)]
struct GeminiPartResponse {
    text: String,
}
