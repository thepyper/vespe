use anyhow::Result;
use llm::{InferenceRequest, InferenceResponse, InferenceSessionConfig, Model, KnownModel, load_progress_callback};
use std::convert::Infallible;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // This example assumes a Llama model is available locally.
    // You would replace "llama-7b-q4_0.bin" with your actual model file.
    let model_path = PathBuf::from("./path/to/your/model/llama-7b-q4_0.bin");

    // Load the model
    let model = llm::load_dynamic(
        Some(llm::ModelArchitecture::Llama),
        &model_path,
        llm::TokenizerSource::Embedded,
        Default::default(),
        load_progress_callback,
    )?;

    let mut session = model.start_session(InferenceSessionConfig::default());

    let prompt = "Hello, what is the capital of France?";
    let mut response_text = String::new();

    session.infer(
        model.as_ref(),
        &mut rand::thread_rng(),
        &InferenceRequest {
            prompt: prompt.into(),
            parameters: &llm::InferenceParameters::default(),
            play_back_previous_tokens: false,
            repetition_penalty_last_n: 64,
            repetition_penalty_sustain_n: 64,
            repetition_penalty: 1.3,
            token_bias: None,
            n_batch: 8,
            n_threads: 4,
            n_predict: Some(128),
            top_k: 40,
            top_p: 0.95,
            temperature: 0.8,
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
                Ok(InferenceResponse::Continue)
            }
            _ => Ok(Infallible),
        },
    )?;

    println!("LLM Response: {}", response_text);

    Ok(())
}
