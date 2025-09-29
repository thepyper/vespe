use async_trait::async_trait;
use crate::memory::Message;
use crate::tool::ToolConfig;

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
    /// Formatta una sequenza di messaggi in una stringa pronta per essere inviata all'LLM.
    ///
    /// # Argomenti
    /// * `messages` - Un vettore di `Message` che rappresenta la cronologia della conversazione.
    ///
    /// # Restituisce
    /// Una stringa formattata per l'input dell'LLM.
    async fn format_messages(
        &self,
        messages: Vec<Message>,
    ) -> Result<String, AgentProtocolError>;

    /// Formatta le definizioni degli strumenti disponibili in una stringa
    /// pronta per essere inclusa nel prompt dell'LLM.
    ///
    /// # Argomenti
    /// * `available_tools` - Un vettore di `ToolConfig` che descrive gli strumenti che l'LLM può chiamare.
    ///
    /// # Restituisce
    /// Una stringa formattata per le definizioni degli strumenti.
    async fn format_available_tools(
        &self,
        available_tools: Option<Vec<ToolConfig>>,
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
