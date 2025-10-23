use serde::{Deserialize, Serialize};

use super::ModelContent;

pub trait State {
    fn new() -> Self;
    fn output(&self) -> String;
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AnchorStatus {
    /// Just created, empty without any content nor state gathered
    JustCreated,
    /// Gathered info, need to process them
    NeedProcessing,
    /// Information has been processed, need to be injected in document
    NeedInjection,
    /// Completed, no further processing needed
    Completed,
}

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
}

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
}

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
}
