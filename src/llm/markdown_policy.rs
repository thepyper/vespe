use super::messages::{Message, AssistantContent};
use anyhow::Result;

pub trait MarkdownPolicy: Send + Sync {
    /// Returns a string containing instructions for the LLM on the expected markdown format
    /// for its responses (e.g., how to format tool calls, thoughts, and text).
    fn markdown_format_instructions(&self) -> String;

    /// Parses the raw LLM response string into a vector of internal `AssistantContent` enums.
    /// This method is responsible for interpreting the LLM's markdown output.
    /// Returns an error if parsing fails or the response does not conform to the policy.
    fn parse_response(&self, response: &str) -> Result<Vec<AssistantContent>>;

    /// Formats a vector of internal `Message` enums into a vector of `llm::chat::ChatMessage`
    /// suitable for the underlying LLM client.
    fn format_query(&self, messages: &[Message]) -> Result<Vec<llm::chat::ChatMessage>>;
}
