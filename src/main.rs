// Import required modules from the LLM library for OpenAI integration
use llm::{
    builder::{FunctionBuilder, LLMBackend, LLMBuilder, ParamBuilder},
    chat::ChatMessage, // Chat-related structures
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get OpenAI API key from environment variable or use test key as fallback
    //let api_key = std::env::var("OPENAI_API_KEY").unwrap_or("sk-TESTKEY".into());
    let base_url = std::env::var("OLLAMA_URL").unwrap_or("http://127.0.0.1:11434".into());
    
    // Initialize and configure the LLM client
    let llm = LLMBuilder::new()
        .backend(LLMBackend::Ollama) // Use OpenAI as the LLM provider
        .base_url(base_url) // Set the Ollama server URL
        .model("llama3.1:8b")
        .max_tokens(512) // Limit response length
        .temperature(0.7) // Control response randomness (0.0-1.0)
        .function(
            FunctionBuilder::new("weather_function")
                .description("Use this tool to get the weather in a specific city")
                .param(
                    ParamBuilder::new("url")
                        .type_of("string")
                        .description("The url to get the weather from for the city"),
                )
                .required(vec!["url".to_string()]),
        )
        .build()
        .expect("Failed to build LLM");

    // Prepare conversation history with example messages
    let messages = vec![ChatMessage::user().content("You are a weather assistant. What is the weather in Tokyo? Use the tools that you have available").build()];

    // Send chat request and handle the response
    // this returns the response as a string. The tool call is also returned as a serialized string. We can deserialize if needed.
    match llm.chat_with_tools(&messages, llm.tools()).await {
        Ok(text) => println!("Chat response:\n{text}"),
        Err(e) => eprintln!("Chat error: {e}"),
    }

    Ok(())
}
