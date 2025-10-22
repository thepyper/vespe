use serde::{Deserialize, Serialize};

use super::Content;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
enum AnchorStatus {
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
    pub snippet_name: String,
}

impl InlineState {
    pub fn new(snippet_name: &str) -> Self {
        InlineState {
            status: AnchorStatus::JustCreated,
            snippet_name: snippet_name.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SummaryState {
    pub status: AnchorStatus,
    pub context_name: String,
    pub context: String,
    pub context_hash: String,
    pub summary: String,
}

impl SummaryState {
    pub fn new(context_name: &str) -> Self {
        SummaryState {
            status: AnchorStatus::JustCreated,
            context_name: context_name.into(),
            context: String::new(),
            context_hash: String::new(),
            summary: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnswerState {
    pub status: AnchorStatus,
    pub query: Content,
    pub reply: Content,
}

impl AnswerState {
    pub fn new() -> Self {
        AnswerState {
            status: AnchorStatus::JustCreated,
            query: Content::new(),
            reply: Content::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeriveState {
    pub status: AnchorStatus,
    pub instruction_context_name: String,
    pub instruction_context: Content,
    pub input_context_name: String,
    pub input_context: Content,
    pub output: Content,
}

impl DeriveState {
    pub fn new() -> Self {
        DeriveState {
            status: AnchorStatus::JustCreated,
            instruction_context_name: String::new(),
            instruction_context: Content::new(),
            input_context_name: String::new(),
            input_context: Content::new(),
            output: Content::new(),
        }
    }
}
