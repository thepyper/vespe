use serde::{Deserialize, Serialize};
use uuid::{uuid, Uuid};

use super::ModelContent;


/// A trait for objects that represent the persistent state of an anchor command.
///
/// This trait provides a common interface for creating, serializing/deserializing,
/// and managing the status of different state machines used by commands like
/// `@answer`, `@derive`, etc.
pub trait State: serde::Serialize + serde::de::DeserializeOwned {
    /// Creates a new instance of the state, typically in its initial status.
    fn new() -> Self;
    /// Generates the final string output to be injected into the document.
    fn output(&self) -> String
    {
        String::new()
    }
    /// Gets the current status of the anchor's state machine.
    fn get_status(&self) -> &AnchorStatus;
    /// Sets the status of the anchor's state machine.
    fn set_status(&mut self, status: AnchorStatus);
}

/// Defines the lifecycle stages of an anchor-based command.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AnchorStatus {
    /// The anchor has just been created. The state is empty and needs to be initialized.
    JustCreated,
    /// The necessary information has been gathered (e.g., context collected).
    /// The state is ready for the main processing step (e.g., calling a model).
    NeedProcessing,
    /// The main processing is complete (e.g., a reply was received from a model).
    /// The resulting output is ready to be injected into the document.
    NeedInjection,
    /// The command has been fully processed, and its output has been injected.
    /// No further action is needed.
    Completed,
}

/// The persistent state for an `@inline` command.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InlineState {
    pub status: AnchorStatus,
    pub context_name: String,
    pub context: String,
}

impl State for InlineState {
    fn new() -> Self {
        InlineState {
            status: AnchorStatus::JustCreated,
            context_name: String::new(),
            context: String::new(),
        }
    }
    fn output(&self) -> String {
        self.context.clone()
    }
    fn get_status(&self) -> &AnchorStatus {
        &self.status
    }
    fn set_status(&mut self, status: AnchorStatus) {
        self.status = status;
    }
}

/// The persistent state for an `@answer` command.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnswerState {
    pub status: AnchorStatus,
    pub query: ModelContent,
    pub reply: String,
}

impl State for AnswerState {
    fn new() -> Self {
        AnswerState {
            status: AnchorStatus::JustCreated,
            query: ModelContent::new(),
            reply: String::new(),
        }
    }
    fn output(&self) -> String {
        self.reply.clone()
    }
    fn get_status(&self) -> &AnchorStatus {
        &self.status
    }
    fn set_status(&mut self, status: AnchorStatus) {
        self.status = status;
    }
}

/// The persistent state for a `@derive` command.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeriveState {
    pub status: AnchorStatus,
    pub instruction_context_name: String,
    pub instruction_context: ModelContent,
    pub input_context_name: String,
    pub input_context: ModelContent,
    pub derived: String,
}

impl State for DeriveState {
    fn new() -> Self {
        DeriveState {
            status: AnchorStatus::JustCreated,
            instruction_context_name: String::new(),
            instruction_context: ModelContent::new(),
            input_context_name: String::new(),
            input_context: ModelContent::new(),
            derived: String::new(),
        }
    }
    fn output(&self) -> String {
        self.derived.clone()
    }
    fn get_status(&self) -> &AnchorStatus {
        &self.status
    }
    fn set_status(&mut self, status: AnchorStatus) {
        self.status = status;
    }
}

/// The persistent state for a `@repeat` command.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepeatState {
    pub status: AnchorStatus,
    pub wrapper_uuid: Uuid,
}

impl State for RepeatState {
    fn new() -> Self {
        RepeatState {
            status: AnchorStatus::JustCreated,
            wrapper_uuid: uuid!("00000000-0000-0000-0000-000000000000"),
        }
    }
    fn get_status(&self) -> &AnchorStatus {
        &self.status
    }
    fn set_status(&mut self, status: AnchorStatus) {
        self.status = status;
    }
}
