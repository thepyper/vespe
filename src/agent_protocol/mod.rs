use async_trait::async_trait;
use crate::memory::Message;
use crate::tool::ToolConfig;

pub struct QueryContext<'a> {
    pub task_context: &'a [Message],
    pub agent_context: &'a [Message],
    pub available_tools: &'a [ToolConfig],
    pub system_instructions: Option<&'a str>,
}

/// Errore specifico per le operazioni di AgentProtocol.
#[derive(Debug, thiserror::Error)]
pub enum AgentProtocolError {
    #[error("Failed to parse LLM output: {0}")]
    ParseError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    // Aggiungi altri tipi di errore se necessario
}

/// Il trait `AgentProtocol` definisce l'interfaccia per la comunicazione tra un agente e un LLM.
/// Gestisce la formattazione dei messaggi in un formato comprensibile dall'LLM e il parsing
/// della risposta dell'LLM in una sequenza di messaggi strutturati.
#[async_trait]
pub trait AgentProtocol: Send + Sync {
    /// Formatta l'intera query per l'LLM, includendo contesto del task, contesto dell'agente,
    /// strumenti disponibili e istruzioni di sistema.
    ///
    /// # Argomenti
    /// * `context` - Una struct `QueryContext` contenente tutti i dati necessari per costruire il prompt.
    ///
    /// # Restituisce
    /// Una stringa formattata per l'input dell'LLM.
    async fn format_query(
        &self,
        context: QueryContext<'_>,
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
pub mod mcp;
