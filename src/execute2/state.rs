use serde::{Deserialize, Serialize};
use uuid::{uuid, Uuid};

use crate::{ast2::Anchor, execute2::variables};

use super::content::ModelContent;
use super::variables::Variables;

/// A trait for objects that represent the persistent state of an anchor command.
///
/// This trait provides a common interface for creating, serializing/deserializing,
/// and managing the status of different state machines used by commands like
/// `@answer`, `@derive`, etc.
pub trait State: serde::Serialize + serde::de::DeserializeOwned {
    /// Creates a new instance of the state, typically in its initial status.
    fn new(variables: &Variables) -> Self;
    /// Generates the final string output to be injected into the document.
    fn output(&self) -> String {
        String::new()
    }
    /// Gets the current status of the anchor's state machine.
    fn get_status(&self) -> &AnchorStatus;
    /// Sets the status of the anchor's state machine.
    fn set_status(&mut self, status: AnchorStatus);

    fn get_variables(&self) -> Variables;
}

/// Defines the lifecycle stages of an anchor-based command.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AnchorStatus {
    /// The anchor has just been created. The state is empty and needs to be initialized.
    JustCreated,
    /// The anchor need to clean current contents and repeat its action.
    NeedRepeat,
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

impl Default for AnchorStatus {
    fn default() -> Self {
        AnchorStatus::JustCreated
    }
}

/// The persistent state for an `@inline` command.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InlineState {
    #[serde(default)]
    pub status: AnchorStatus,
    #[serde(default)]
    pub context_name: String,
    #[serde(default)]
    pub context: String,
    #[serde(default)]
    pub variables: Variables,
}

impl State for InlineState {
    fn new(variables: &Variables) -> Self {
        InlineState {
            status: AnchorStatus::JustCreated,
            context_name: String::new(),
            context: String::new(),
            variables: variables.clone(),
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
    fn get_variables(&self) -> Variables {
        self.variables.clone()
    }
}

/// The persistent state for an `@answer` command.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnswerState {
    #[serde(default)]
    pub status: AnchorStatus,
    #[serde(default)]
    pub query: ModelContent,
    #[serde(default)]
    pub reply: String,
    #[serde(default)]
    pub variables: Variables,
}

impl State for AnswerState {
    fn new(variables: &Variables) -> Self {
        AnswerState {
            status: AnchorStatus::JustCreated,
            query: ModelContent::new(),
            reply: String::new(),
            variables: variables.clone(),
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
    fn get_variables(&self) -> Variables {
        self.variables.clone()
    }
}

/// The persistent state for an `@decide` command.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DecideState {
    #[serde(default)]
    pub status: AnchorStatus,
    #[serde(default)]
    pub query: ModelContent,
    #[serde(default)]
    pub reply: String,
    #[serde(default)]
    pub variables: Variables,
}

impl State for DecideState {
    fn new(variables: &Variables) -> Self {
        DecideState {
            status: AnchorStatus::JustCreated,
            query: ModelContent::new(),
            reply: String::new(),
            variables: variables.clone(),
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
    fn get_variables(&self) -> Variables {
        self.variables.clone()
    }
}

/// The persistent state for an `@choose` command.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChooseState {
    #[serde(default)]
    pub status: AnchorStatus,
    #[serde(default)]
    pub query: ModelContent,
    #[serde(default)]
    pub reply: String,
    #[serde(default)]
    pub variables: Variables,
}

impl State for ChooseState {
    fn new(variables: &Variables) -> Self {
        ChooseState {
            status: AnchorStatus::JustCreated,
            query: ModelContent::new(),
            reply: String::new(),
            variables: variables.clone(),
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
    fn get_variables(&self) -> Variables {
        self.variables.clone()
    }
}

/// The persistent state for a `@derive` command.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeriveState {
    #[serde(default)]
    pub status: AnchorStatus,
    #[serde(default)]
    pub instruction_context_name: String,
    #[serde(default)]
    pub instruction_context: ModelContent,
    #[serde(default)]
    pub input_context_name: String,
    #[serde(default)]
    pub input_context: ModelContent,
    #[serde(default)]
    pub derived: String,
    #[serde(default)]
    pub variables: Variables,
}

impl State for DeriveState {
    fn new(variables: &Variables) -> Self {
        DeriveState {
            status: AnchorStatus::JustCreated,
            instruction_context_name: String::new(),
            instruction_context: ModelContent::new(),
            input_context_name: String::new(),
            input_context: ModelContent::new(),
            derived: String::new(),
            variables: variables.clone(),
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
    fn get_variables(&self) -> Variables {
        self.variables.clone()
    }
}

/// The persistent state for a `@repeat` command.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepeatState {
    #[serde(default)]
    pub status: AnchorStatus,
    #[serde(default)]
    pub wrapper: Uuid,
    #[serde(default)]
    pub variables: Variables,
}

impl State for RepeatState {
    fn new(variables: &Variables) -> Self {
        RepeatState {
            status: AnchorStatus::JustCreated,
            wrapper: uuid::uuid!("00000000-0000-0000-0000-000000000000"),
            variables: variables.clone(),
        }
    }
    fn get_status(&self) -> &AnchorStatus {
        &self.status
    }
    fn set_status(&mut self, status: AnchorStatus) {
        self.status = status;
    }
    fn get_variables(&self) -> Variables {
        self.variables.clone()
    }
}
