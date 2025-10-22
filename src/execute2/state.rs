use serde::{Deserialize, Serialize};

use crate::ast2::Content;

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

impl InlineState {
    pub fn new() -> Self {
        InlineState {
            status: AnchorStatus::JustCreated,
            context_name: String::new(),
            context: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnswerState {
    pub status: AnchorStatus,
    pub query: Content,
    pub reply: String,
}

impl AnswerState {
    pub fn new() -> Self {
        AnswerState {
            status: AnchorStatus::JustCreated,
            query: Content::Text(Text { content: String::new(), range: Range::null() }),
            reply: String::new(),
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
    pub output: String,
}

impl DeriveState {
    pub fn new() -> Self {
        DeriveState {
            status: AnchorStatus::JustCreated,
            instruction_context_name: String::new(),
            instruction_context: Content::Text(Text { content: String::new(), range: Range::null() }),
            input_context_name: String::new(),
            input_context: Content::Text(Text { content: String::new(), range: Range::null() }),
            output: String::new(),
        }
    }
}
