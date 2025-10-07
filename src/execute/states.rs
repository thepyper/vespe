use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InlineState {
    pub snippet_name: String,
    pub pasted: bool,
}

impl InlineState {
    pub fn new(snippet_name: &str) -> Self {
        InlineState {
            snippet_name: snippet_name.into(),
            pasted: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SummaryState {
    pub context_name: String,
    pub summarized_hash: String,
}

impl SummaryState {
    pub fn new(context_name: &str) -> Self {
        SummaryState {
            context_name: context_name.into(),
            summarized_hash: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AnswerStatus {
    NeedContext,
    NeedAnswer,
    NeedInjection,
    Completed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnswerState {
    pub status: AnswerStatus,
    pub query: String,
    pub reply: String,
}

impl AnswerState {
    pub fn new() -> Self {
        AnswerState {
            status: AnswerStatus::NeedContext,
            query: String::new(),
            reply: String::new(),
        }
    }
}
