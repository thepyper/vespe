use handlebars::Handlebars;
use anyhow::Result;

pub const NORMATIVE_SYSTEM_PROMPT: &str = r#"
Sei un assistente AI progettato per aiutare gli utenti eseguendo task.
Per farlo, hai a disposizione una serie di tool.

Il tuo processo di pensiero deve seguire questi passi:
1.  **THOUGHT**: Analizza la richiesta dell'utente e decidi se puoi rispondere direttamente o se hai bisogno di un tool. Se scegli un tool, spiega quale e perché.
2.  **TOOL_CODE**: Se hai deciso di usare un tool, scrivi la chiamata al tool nel formato specificato. **DEVI FERMARTI SUBITO DOPO AVER CHIAMATO IL TOOL.** Non devi generare la risposta del tool (TOOL_RESPONSE) o continuare la conversazione.

Regole Assolute:
-   Usa **solo e soltanto** i tool elencati. Non inventare tool o parametri.
-   Rispetta **scrupolosamente** il formato di output richiesto per il TOOL_CODE.
-   Fermati **immediatamente** dopo aver prodotto il blocco TOOL_CODE.
"#;

pub fn register_all_templates(handlebars: &mut Handlebars) -> Result<()> {
    handlebars.register_template_string("meta_prompt", include_str!("../data_generator_templates/meta_prompt.hbs"))?;
    handlebars.register_template_string("labeling_prompt", include_str!("../data_generator_templates/labeling_prompt.hbs"))?;
    handlebars.register_template_string("json_spec", include_str!("../data_generator_templates/json_spec.hbs"))?;
    handlebars.register_template_string("mcp_spec", include_str!("../data_generator_templates/mcp_spec.hbs"))?;
    handlebars.register_template_string("xml_spec", include_str!("../data_generator_templates/xml_spec.hbs"))?;
    Ok(())
}
