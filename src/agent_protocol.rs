use async_trait::async_trait;
use serde_json::Value;
use crate::error::ProjectError;
use crate::memory::{Message, MessageContent};

/// Rappresenta la definizione di uno strumento che può essere offerto all'LLM.
/// Questa struttura verrà utilizzata per generare la parte del prompt che descrive gli strumenti disponibili.
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value, // JSON Schema per i parametri dello strumento
}

/// Errore specifico per le operazioni di AgentProtocol.
#[derive(Debug, thiserror::Error)]
pub enum AgentProtocolError {
    #[error("Failed to parse LLM output: {0}")]
    ParseError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Project error: {0}")]
    Project(#[from] ProjectError),
    // Aggiungi altri tipi di errore se necessario
}

/// Il trait `AgentProtocol` definisce l'interfaccia per la comunicazione tra un agente e un LLM.
/// Gestisce la formattazione dei messaggi in un formato comprensibile dall'LLM e il parsing
/// della risposta dell'LLM in una sequenza di messaggi strutturati.
#[async_trait]
pub trait AgentProtocol: Send + Sync {
    /// Formatta una sequenza di messaggi e le definizioni degli strumenti in una stringa
    /// pronta per essere inviata all'LLM.
    ///
    /// # Argomenti
    /// * `messages` - Un vettore di `Message` che rappresenta la cronologia della conversazione.
    /// * `available_tools` - Un vettore di `ToolDefinition` che descrive gli strumenti che l'LLM può chiamare.
    ///
    /// # Restituisce
    /// Una stringa formattata per l'input dell'LLM.
    async fn format_messages(
        &self,
        messages: Vec<Message>,
        available_tools: Option<Vec<ToolDefinition>>,
    ) -> Result<String, AgentProtocolError>;

    /// Parsa la stringa di output grezza dell'LLM in una sequenza di `Message` strutturati.
    ///
    /// # Argomenti
    /// * `llm_output` - La stringa di risposta grezza ricevuta dall'LLM.
    ///
    /// # Restituisce
    /// Un `Result` contenente un vettore di `Message` parsati o un `AgentProtocolError` in caso di fallimento.
    async fn parse_llm_output(
        &self,
        llm_output: String,
    ) -> Result<Vec<Message>, AgentProtocolError>;
}
