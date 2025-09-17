use anyhow::{Result, anyhow};
use handlebars::Handlebars;
use serde_json::Value;
use std::path::PathBuf;
use std::fs;

pub struct PromptTemplater<'a> {
    handlebars: Handlebars<'a>,
    template_dir: PathBuf,
}

impl<'a> PromptTemplater<'a> {
    pub fn new(template_dir: PathBuf) -> Result<Self> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true); // Strict mode for better error detection

        // Register all templates in the template_dir
        for entry in fs::read_dir(&template_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "hbs") {
                let template_name = path.file_stem().and_then(|s| s.to_str()).ok_or_else(|| {
                    anyhow!("Invalid template file name: {}", path.display())
                })?.to_string();
                let template_content = fs::read_to_string(&path)?;
                handlebars.register_template_string(&template_name, template_content)?;
            }
        }

        Ok(Self {
            handlebars,
            template_dir,
        })
    }

    pub fn render_prompt(&self, template_name: &str, data: &Value) -> Result<String> {
        self.handlebars.render(template_name, data).map_err(|e| anyhow!("Failed to render template: {}", e))
    }
}
