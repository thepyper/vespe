// vespe-project/src/agent_api.rs

use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::{Tool, ProjectConfig}; // Assumendo che Task, Tool e ProjectConfig siano già definite e pubbliche

/// Rappresenta il contesto fornito a un agente per l'esecuzione di un'azione.
/// Contiene tutte le informazioni rilevanti che l'agente potrebbe necessitare.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Context {
    /// Il prompt principale o la richiesta dell'utente per l'agente.
    pub prompt: String,
    /// L'UID del task corrente, se l'agente opera nel contesto di un task specifico.
    pub current_task_uid: Option<String>,
    /// Il percorso della root del progetto Vespe.
    pub project_root: PathBuf,
    /// La configurazione del progetto Vespe.
    pub project_config: ProjectConfig,
    /// Una lista delle definizioni degli strumenti disponibili per l'agente.
    /// Questi strumenti possono essere richiamati dall'agente.
    pub available_tools: Vec<Tool>,
    // TODO: Aggiungere altri campi man mano che le esigenze si evolvono, ad esempio:
    // pub chat_history: Vec<Message>, // Per agenti conversazionali
    // pub working_directory: PathBuf, // Per operazioni su file
    // pub relevant_files: Vec<PathBuf>, // File rilevanti per il contesto
}

/// Rappresenta la risposta strutturata di un agente.
/// Contiene il risultato dell'elaborazione dell'agente e le eventuali azioni suggerite.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response {
    /// La risposta testuale principale dell'agente.
    pub text_response: String,
    // TODO: Aggiungere altri campi man mano che le esigenze si evolvono, ad esempio:
    // pub tool_calls: Vec<ToolCall>, // Azioni suggerite dall'agente (es. chiamate a strumenti)
    // pub pub thoughts: Option<String>, // Pensieri interni dell'agente per debugging/trasparenza
    // pub new_task_state: Option<TaskState>, // Suggerimento per un cambio di stato del task
    // pub files_to_create: Vec<FileContent>, // Contenuto di file da creare
    // pub files_to_modify: Vec<FileContent>, // Contenuto di file da modificare
}
