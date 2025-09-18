use crate::llm::messages::Message;

/// A simple formatter to convert a series of messages into a single string for the LLM.
/// This is a basic implementation and can be expanded to support different formats.
pub fn format_messages(messages: &[Message]) -> String {
    messages
        .iter()
        .map(|m| match m {
            Message::System(s) => format!("System: {{}}\n", s),
            Message::User(s) => format!("User: {{}}\n", s),
            Message::Assistant(contents) => {
                let content_str = contents
                    .iter()
                    .map(|c| format!("{:?}", c))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("Assistant: {{}}\n", content_str)
            }
            Message::Tool(output) => format!("Tool Output: {{:?}}\n", output),
        })
        .collect::<Vec<_>>()
        .join("\n")
}