use serde::{Deserialize, Serialize};

use super::ModelContent;

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


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnswerState {
    pub status: AnchorStatus,
    pub query: ModelContent,
    pub reply: String,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeriveState {
    pub status: AnchorStatus,
    pub instruction_context_name: String,
    pub instruction_context: ModelContent,
    pub input_context_name: String,
    pub input_context: ModelContent,
    pub output: String,
}

