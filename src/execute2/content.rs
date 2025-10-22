struct SystemModelContent {
    text: String,
}

struct UserModelContent {
    text: String,
}

struct AgentModelContent {
    author: String,
    text: String,
}

pub enum ModelContentItem {
    System(SystemModelContent),
    User(UserModelContent),
    Agent(AgentModelContent),
}

impl ModelContentItem {
    pub fn user(text: &str) -> Self {
        ContentItem::User(UserModelContent { text: text.into() })
    }
    fn system(text: &str) -> Self {
        ContentItem::System(SystemModelContent { text: text.into() })
    }
    fn agent(author: &str, text: &str) -> Self {
        ContentItem::Agent(AgentModelContent {
            author: author.into(),
            text: text.into(),
        })
    }
}

type Vec<ModelContentItem> = ModelContent;
