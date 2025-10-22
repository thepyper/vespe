
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemModelContent {
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserModelContent {
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentModelContent {
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
    fn system(text: &str) -> Self {
        ModelContentItem::System(SystemModelContent { text: text.into() })
    }
    fn agent(author: &str, text: &str) -> Self {
        ModelContentItem::Agent(AgentModelContent {
            author: author.into(),
            text: text.into(),
        })
    }
}

pub type ModelContent = Vec<ModelContentItem>;
