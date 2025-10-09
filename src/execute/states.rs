use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum InlineStatus {
    NeedInjection,
    Completed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InlineState {
    pub status: InlineStatus,
    pub snippet_name: String,
}

impl InlineState {
    pub fn new(snippet_name: &str) -> Self {
        InlineState {
            status: InlineStatus::NeedInjection,
            snippet_name: snippet_name.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum SummaryStatus {
    NeedContext,
    NeedInjection,
    Completed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SummaryState {
    pub status: SummaryStatus,
    pub context_name: String,
    pub context: String,
    pub context_hash: String,
    pub summary: String,
}

impl SummaryState {
    pub fn new(context_name: &str) -> Self {
        SummaryState {
            status: SummaryStatus::NeedContext,
            context_name: context_name.into(),
            context: String::new(),
            context_hash: String::new(),
            summary: String::new(),
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
