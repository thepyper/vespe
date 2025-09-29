use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};
use crate::utils::{read_json_file, write_json_file, generate_uid};
use std::fs;
use crate::error::ProjectError;

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
#[derive(Debug, Clone)]
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
    #[error("UID generation error: {0}")]
    UidGenerationError(String),
}

impl Memory {
    /// Carica una memoria esistente da una directory.
    pub fn load(path: &Path) -> Result<Self, MemoryError> {
        if !path.exists() {
            return Err(MemoryError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Memory path not found: {}", path.display()))));
        }
        let mut messages = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.is_file() && entry_path.extension().map_or(false, |ext| ext == "json") {
                let message: Message = read_json_file(&entry_path).map_err(|e| match e {
                    ProjectError::Io(io_err) => MemoryError::Io(io_err),
                    ProjectError::Json(json_err) => MemoryError::Json(json_err),
                    _ => MemoryError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Unexpected project error: {}", e))),
                })?;
                messages.push(message);
            }
        }
        messages.sort_by_key(|m| m.timestamp);
        Ok(Self { root_path: path.to_path_buf(), messages })
    }

    /// Crea una nuova memoria (e la sua directory).
    pub fn new(path: &Path) -> Result<Self, MemoryError> {
        fs::create_dir_all(path)?;
        Ok(Self { root_path: path.to_path_buf(), messages: Vec::new() })
    }

    /// Aggiunge un nuovo messaggio e lo persiste su file.
    pub fn add_message(&mut self, author_agent_uid: String, content: MessageContent) -> Result<&Message, MemoryError> {
        let uid = generate_uid("msg").map_err(|e| MemoryError::UidGenerationError(e.to_string()))?;
        let now = Utc::now();
        let message = Message {
            uid: uid.clone(),
            timestamp: now,
            author_agent_uid,
            content,
            status: MessageStatus::Enabled,
        };
        let message_path = self.root_path.join(format!("{}.json", uid));
        write_json_file(&message_path, &message).map_err(|e| match e {
            ProjectError::Io(io_err) => MemoryError::Io(io_err),
            ProjectError::Json(json_err) => MemoryError::Json(json_err),
            _ => MemoryError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Unexpected project error: {}", e))),
        })?;
        self.messages.push(message);
        self.messages.sort_by_key(|m| m.timestamp);
        Ok(self.messages.last().unwrap())
    }

    /// Elimina un messaggio in modo permanente.
    pub fn delete_message(&mut self, message_uid: &str) -> Result<(), MemoryError> {
        let initial_len = self.messages.len();
        self.messages.retain(|m| m.uid != message_uid);
        if self.messages.len() == initial_len {
            return Err(MemoryError::MessageNotFound(message_uid.to_string()));
        }
        let message_path = self.root_path.join(format!("{}.json", message_uid));
        fs::remove_file(&message_path)?;
        Ok(())
    }

    /// Abilita un messaggio.
    pub fn enable_message(&mut self, message_uid: &str) -> Result<(), MemoryError> {
        if let Some(message) = self.messages.iter_mut().find(|m| m.uid == message_uid) {
            message.status = MessageStatus::Enabled;
            let message_path = self.root_path.join(format!("{}.json", message_uid));
            write_json_file(&message_path, message).map_err(|e| match e {
                ProjectError::Io(io_err) => MemoryError::Io(io_err),
                ProjectError::Json(json_err) => MemoryError::Json(json_err),
                _ => MemoryError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Unexpected project error: {}", e))),
            })?;
            Ok(())
        } else {
            Err(MemoryError::MessageNotFound(message_uid.to_string()))
        }
    }

    /// Disabilita un messaggio.
    pub fn disable_message(&mut self, message_uid: &str) -> Result<(), MemoryError> {
        if let Some(message) = self.messages.iter_mut().find(|m| m.uid == message_uid) {
            message.status = MessageStatus::Disabled;
            let message_path = self.root_path.join(format!("{}.json", message_uid));
            write_json_file(&message_path, message).map_err(|e| match e {
                ProjectError::Io(io_err) => MemoryError::Io(io_err),
                ProjectError::Json(json_err) => MemoryError::Json(json_err),
                _ => MemoryError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Unexpected project error: {}", e))),
            })?;
            Ok(())
        } else {
            Err(MemoryError::MessageNotFound(message_uid.to_string()))
        }
    }

    /// Restituisce tutti i messaggi (abilitati e non).
    pub fn get_all_messages(&self) -> &Vec<Message> { &self.messages }

    /// Restituisce solo i messaggi abilitati, pronti per essere usati come contesto.
    pub fn get_enabled_messages(&self) -> Vec<&Message> {
        self.messages.iter().filter(|m| m.status == MessageStatus::Enabled).collect()
    }
}
