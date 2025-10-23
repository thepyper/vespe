struct Variables {
    provider: String,
}

impl Variables {
    pub fn new() -> Self {
        Variables {
            provider: "gemini -p -y -m gemini-2.5-flash".to_string(),
        }
    }
}
