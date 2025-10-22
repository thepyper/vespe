
use serde::{Serialize, Deserialize};

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
    }}

pub type ModelContent = Vec<ModelContentItem>;
