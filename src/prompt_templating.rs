use anyhow::{Result, anyhow};
use handlebars::Handlebars;
use serde_json::Value;
use std::path::PathBuf;
use std::fs;

use crate::llm::markup_policy::MarkupPolicy;

pub struct PromptTemplater {
    handlebars: Handlebars<'static>, // Use 'static' lifetime for owned Handlebars instance
    template_dir: PathBuf,
}

impl PromptTemplater {
    pub fn new(template_dir: PathBuf) -> Result<Self> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        handlebars.register_template_file("system_prompt", template_dir.join("system_prompt.hbs"))?;

        Ok(Self {
            handlebars,
            template_dir,
        })
    }

    pub fn render_system_prompt(&self, agent_name: &str, markup_policy: &dyn MarkupPolicy) -> Result<String> {
        let markup_instructions = markup_policy.get_markup_instructions();
        let data = json!({
            "agent_name": agent_name,
            "markup_instructions": markup_instructions,
            // Add other context variables here if needed
        });
        let rendered = self.handlebars.render("system_prompt", &data)?;
        Ok(rendered)
    }
}
