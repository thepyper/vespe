use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs;

pub mod models;

pub use models::{GlobalConfig, LlmConfig};

pub async fn load_global_config() -> Result<GlobalConfig> {
    // For now, provide a hardcoded default. In a real scenario, this would
    // load from a system-wide path (e.g., ~/.config/vespe/config.toml)
    Ok(GlobalConfig {
        default_llm_config: LlmConfig {
            provider: "openai".to_string(),
            model_id: "gpt-4o-mini".to_string(),
            api_key: None, // Should be loaded from env or secret manager
            temperature: 0.7,
            max_tokens: 512,
        },
    })
}

pub async fn load_project_config(global_config: GlobalConfig) -> Result<GlobalConfig> {
    let project_root = PathBuf::from("."); // Assuming current dir is project root
    let config_path = project_root.join(".vespe").join("config.toml");

    if config_path.exists() {
        tracing::info!("Loading project-specific configuration from {:?}", config_path);
        let content = fs::read_to_string(&config_path)
            .await
            .context("Failed to read project config file")?;
        let project_config: GlobalConfig = toml::from_str(&content)
            .context("Failed to parse project config TOML")?;

        // Apply overrides from project_config to global_config
        // For simplicity, we'll just return the project_config if it exists
        // A more robust implementation would merge fields selectively.
        Ok(project_config)
    } else {
        tracing::info!("No project-specific configuration found at {:?}", config_path);
        Ok(global_config)
    }
}