use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemModelContent {
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserModelContent {
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentModelContent {
    author: String,
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelContentItem {
    System(SystemModelContent),
    User(UserModelContent),
    Agent(AgentModelContent),
}

impl ModelContentItem {
    pub fn user(text: &str) -> Self {
        ModelContentItem::User(UserModelContent { text: text.into() })
    }
}

impl ToString for ModelContentItem {
    fn to_string(&self) -> String {
        match self {
            /*
            ModelContentItem::System(content) => format!("System: {}", content.text),
            ModelContentItem::User(content) => format!("User: {}", content.text),
            ModelContentItem::Agent(content) => {
                format!("Agent ({}): {}", content.author, content.text)
            }
            */
            ModelContentItem::System(content) => format!("{}", content.text),
            ModelContentItem::User(content) => format!("{}", content.text),
            ModelContentItem::Agent(content) => {
                format!("{}", content.text)
            }
        }
    }
}

pub type ModelContent = Vec<ModelContentItem>;
