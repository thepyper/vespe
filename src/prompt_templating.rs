use anyhow::Result;
use handlebars::Handlebars;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::llm::markup_policy::MarkupPolicy;

#[derive(Clone)]
pub struct PromptTemplater {
    handlebars: Arc<Mutex<Handlebars<'static>>>,
    template_dir: PathBuf,
}

impl PromptTemplater {
    pub fn new(template_dir: PathBuf) -> Result<Self> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        handlebars.register_template_file("system_prompt", template_dir.join("system_prompt.hbs"))?;

        Ok(Self {
            handlebars: Arc::new(Mutex::new(handlebars)),
            template_dir,
        })
    }

    pub async fn render_system_prompt(&self, agent_name: &str, tool_prompt: &str, markup_policy: &dyn MarkupPolicy) -> Result<String> {
        let markup_instructions = markup_policy.get_markup_instructions();
        let data = json!({
            "agent_name": agent_name,
            "tool_prompt": tool_prompt,
            "markup_instructions": markup_instructions,
            // Add other context variables here if needed
        });
        let handlebars = self.handlebars.lock().await;
        let rendered = handlebars.render("system_prompt", &data)?;
        Ok(rendered)
    }
}
