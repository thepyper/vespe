/*
pub mod agent;
pub mod task;
pub mod error;
pub mod utils;
pub mod tool;
pub mod memory;
pub mod registry;

pub mod project;

pub use tool::*;
pub use project::*;
pub use task::*;
pub use agent::*;
pub use error::*;
pub use utils::*;
pub use memory::*;
*/

use std::path::Path;
use anyhow::Result;

pub enum TaskStatePlanSectionItem{
    LocalTask(String),
    ReferencedTask(String),
}

pub enum TaskStateSection {
    Intent{ title: String, text: String },
    Plan{ title: String, items: Vec<TaskStatePlanSectionItem> },
    Text{ title: String, items: String }, 
}

pub struct TaskState {
    /// Original markdown file 
    md: String,
    /// Original markdown file parsed ast
    //mdast: markdown::mdast::Node,
    /// State structure parsed
    sections: Vec<TaskStateSection>,
}

pub struct TaskMetadata {
    name: String,
}

pub struct Task {
    uid: String,
    meta: TaskMetadata,
    state: TaskState,
}

/* TODO decidere come fare
pub enum ReconcileQueryKind {
    DeleteConfirm,
}

pub struct ReconcileQuery {
    uid: String,    
    kind: ReconcileQueryKind,
}

pub enum ReconcileResponseKind {
    Accept,
    Reject,
}

pub enum ReconcileResponse {
    uid: String,
    query_kind: ReconcileQueryKind,
    response_kind: ReconcileResponseKind,
}
*/

impl TaskState {

    pub fn new() -> TaskState {
        TaskState {
            // TODO vuoto 
        }
    }
    /// Ricarica da files interni al task
    pub fn load(task_root_path: &Path) -> Result<TaskState> {
        // TODO se file non esiste, usa new()
        // TODO se file esiste, carica ed esegui parsing in mdast, e poi in sections
        // TODO se file malformato, errore
    }
    /// Riconcilia file modificato da utente con file interno
    pub fn reconcile(md_file_path: &Path) -> Result<()> { // TODO return type? result ok, result errore con problemi (potenzialmente da sistemare con utente)
        // TODO carica md
        // TODO parsing md -> mdast
        // TODO parsing mdast -> new_sections
        // TODO reconcile new_sections con sections esistenti; in questa fase marking nuove sezioni, invenzione uid, genera modifiche per marking
        // TODO se reconcile ok, salva nuovo file sia internamente che sovrascrivi quello passato; altrimenti non salvare nulla

        // TODO decidere come dare a utente domande / risposte su reconcile... 1 - passare per file scrivendo dei tag; 2 - passare struttura dati runtime;
        // forse 1 piu' coerente con il design?
        unimplemented!();
    }        
}