use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use tracing::info;

use llm::{
    InferenceRequest,
    InferenceResponse,
    InferenceSessionConfig,
    KnownModel,
    Model,
    load_progress_callback,
};

// Re-define ChatMessage and LlmResponse for internal use within the adapter,
// or adapt to llm crate's internal types if they are exposed.
// For now, let's keep them similar to the old vespe ones for easier transition.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LlmResponse {
    pub content: String,
}

// This trait will be implemented by our LlmAdapter
#[async_trait]
pub trait LlmClient {
    async fn generate_response(&self, messages: Vec<ChatMessage>) -> Result<LlmResponse>;
}

pub struct LlmAdapter {
    // The actual llm crate model instance
    model: Box<dyn Model>,
    // Configuration for the LLM
    config: crate::config::LlmConfig,
}

impl LlmAdapter {
    pub fn new(config: crate::config::LlmConfig) -> Result<Self> {
        // This is a placeholder for loading a real model based on config.
        // In a real scenario, you'd use config.model_id to determine which model to load.
        // For now, we'll just load a dummy Llama model.
        let model_path = PathBuf::from("./path/to/your/model/llama-7b-q4_0.bin"); // Placeholder

        let model = llm::load_dynamic(
            Some(llm::ModelArchitecture::Llama), // This should be dynamic based on config
            &model_path,
            llm::TokenizerSource::Embedded,
            Default::default(),
            load_progress_callback,
        )?;

        Ok(Self { model, config })
    }
}

#[async_trait]
impl LlmClient for LlmAdapter {
    async fn generate_response(&self, messages: Vec<ChatMessage>) -> Result<LlmResponse> {
        let mut session = self.model.start_session(InferenceSessionConfig::default());

        // Convert ChatMessage to a single prompt string for the llm crate
        let prompt_string = messages.iter()
            .map(|msg| format!( "{}: {}", msg.role, msg.content))
            .collect::<Vec<String>>()
            .join("\n");

        let mut response_text = String::new();

        session.infer(
            self.model.as_ref(),
            &mut rand::thread_rng(),
            &InferenceRequest {
                prompt: prompt_string.into(),
                parameters: &llm::InferenceParameters::default(),
                play_back_previous_tokens: false,
                repetition_penalty_last_n: 64,
                repetition_penalty_sustain_n: 64,
                repetition_penalty: 1.3,
                token_bias: None,
                n_batch: 8,
                n_threads: 4,
                n_predict: Some(self.config.max_tokens as usize),
                top_k: 40,
                top_p: 0.95,
                temperature: self.config.temperature,
                bias_tokens: None,
                mirostat: llm::Mirostat::V0,
                mirostat_tau: 5.0,
                mirostat_eta: 0.1,
                log_softmax: false,
                grammar: None,
                path_session_cache: None,
                token_callback: None,
            },
            &mut |r| match r {
                InferenceResponse::PromptToken(t) | InferenceResponse::InferredToken(t) => {
                    response_text.push_str(&t);
                    Ok(llm::InferenceFeedback::Continue)
                }
                _ => Ok(llm::InferenceFeedback::Continue), // Handle other response types
            },
        )?;

        info!("LLM Adapter Response: {}", response_text);

        Ok(LlmResponse { content: response_text })
    }
}
