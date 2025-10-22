struct SystemContent {
    text: String,
}

struct UserContent {
    text: String,
}

struct AgentContent {
    author: String,
    text: String,
}

enum ContentItem {
    System(SystemContent),
    User(UserContent),
    Agent(AgentContent),
}

impl ContentItem {
    fn user(text: &str) -> Self {
        ContentItem::User(UserContent { text: text.into() })
    }
    fn system(text: &str) -> Self {
        ContentItem::System(SystemContent { text: text.into() })
    }
    fn agent(author: &str, text: &str) -> Self {
        ContentItem::Agent(AgentContent {
            author: author.into(),
            text: text.into(),
        })
    }
}

type Vec<ContentItem> = Content;
