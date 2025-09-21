use handlebars::Handlebars;
use anyhow::Result;

pub fn register_all_templates(handlebars: &mut Handlebars) -> Result<()> {
    handlebars.register_template_string("meta_prompt", include_str!("../data_generator_templates/meta_prompt.hbs"))?;
    handlebars.register_template_string("labeling_prompt", include_str!("../data_generator_templates/labeling_prompt.hbs"))?;
    handlebars.register_template_string("system_prompt", include_str!("../data_generator_templates/system_prompt.hbs"))?;
    handlebars.register_template_string("json_spec", include_str!("../data_generator_templates/json_spec.hbs"))?;
    handlebars.register_template_string("mcp_spec", include_str!("../data_generator_templates/mcp_spec.hbs"))?;
    handlebars.register_template_string("xml_spec", include_str!("../data_generator_templates/xml_spec.hbs"))?;
    Ok(())
}
