use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};

// 1. MESSAGE: L'unità atomica di memoria.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub uid: String, // "msg-..."
    pub timestamp: DateTime<Utc>,
    pub author_agent_uid: String, // L'autore è SEMPRE un agente
    pub content: MessageContent,  // Il contenuto è strutturato
    #[serde(default)]
    pub status: MessageStatus,
}

// 2. MESSAGECONTENT: Enum per descrivere il TIPO di contenuto.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MessageContent {
    Text(String),    // Input utente, output finale dell'agente, messaggi di sistema
    Thought(String), // Ragionamento interno dell'agente (non mostrato di default)
    ToolResult {
        tool_uid: String,
        result: serde_json::Value,
    },
}

// 3. MESSAGESTATUS: Lo stato di un messaggio nel contesto.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
pub enum MessageStatus {
    #[default]
    Enabled,
    Disabled, // Ignorato dal contesto di default, ma conservato per la cronologia
}

// 4. MEMORY: Il gestore di una collezione di messaggi.
#[derive(Debug)]
pub struct Memory {
    pub root_path: PathBuf, // Path alla directory che contiene i file msg-*.json
    messages: Vec<Message>, // Cache in-memory
}

// Custom Error for Memory operations
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Message not found: {0}")]
    MessageNotFound(String),
}

impl Memory {
    pub fn load(path: &Path) -> Result<Self, MemoryError> {
        unimplemented!();
    }

    pub fn new(path: &Path) -> Result<Self, MemoryError> {
        unimplemented!();
    }

    pub fn add_message(&mut self, author_agent_uid: String, content: MessageContent) -> Result<&Message, MemoryError> {
        unimplemented!();
    }

    pub fn delete_message(&mut self, message_uid: &str) -> Result<(), MemoryError> {
        unimplemented!();
    }

    pub fn enable_message(&mut self, message_uid: &str) -> Result<(), MemoryError> {
        unimplemented!();
    }

    pub fn disable_message(&mut self, message_uid: &str) -> Result<(), MemoryError> {
        unimplemented!();
    }

    pub fn get_all_messages(&self) -> &Vec<Message> { &self.messages }

    pub fn get_context(&self) -> Vec<&Message> {
        unimplemented!();
    }
}
