use handlebars::Handlebars;
use anyhow::Result;

pub fn register_all_templates(handlebars: &mut Handlebars) -> Result<()> {
    handlebars.register_template_string("meta_prompt", include_str!("../tool-format-tester-templates/meta_prompt.hbs"))?;
    handlebars.register_template_string("system_prompt", include_str!("../tool-format-tester-templates/system_prompt.hbs"))?;
    // Note: Policy-specific templates are now handled by the policies themselves.
    Ok(())
}