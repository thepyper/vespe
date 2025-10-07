use crate::project::Project;
use crate::syntax::types::{self, Anchor, AnchorKind, AnchorTag, TagKind};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

// Error type for semantic processing
#[derive(Debug, thiserror::Error)]
pub enum SemanticError {
    #[error("Missing argument: {0}")]
    MissingArgument(String),
    #[error("Invalid anchor format: {0}")]
    InvalidAnchorFormat(String),
    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

// Placeholder for InlineState
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum InlineState {
    #[default]
    Default,
    // Add other states as needed
}

impl InlineState {
    pub fn load(project: &Project, uid: &Uuid) -> Self {
        // TODO: Implement actual loading logic from project metadata
        Self::default()
    }
}

// Placeholder for SummaryState
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum SummaryState {
    #[default]
    Default,
    // Add other states as needed
}

impl SummaryState {
    pub fn load(project: &Project, uid: &Uuid) -> Self {
        // TODO: Implement actual loading logic from project metadata
        Self::default()
    }
}

// Placeholder for AnswerState
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum AnswerState {
    #[default]
    Default,
    // Add other states as needed
}

impl AnswerState {
    pub fn load(project: &Project, uid: &Uuid) -> Self {
        // TODO: Implement actual loading logic from project metadata
        Self::default()
    }
}

#[derive(Debug)]
pub enum Line {
    Text(String),
    InlineTag { snippet_name: String },
    SummaryTag { context_name: String },
    AnswerTag,
    IncludeTag { context_name: String },
    InlineBeginAnchor { uuid: String, state: InlineState },
    InlineEndAnchor { uuid: String },
    SummaryBeginAnchor { uuid: String, state: SummaryState },
    SummaryEndAnchor { uuid: String },
    AnswerBeginAnchor { uuid: String, state: AnswerState },
    AnswerEndAnchor { uuid: String },
}

fn enrich_syntax_tagged_line(project: &Project, tag: &TagKind, parameters: &HashMap<String, String>, arguments: &Vec<String>) -> Result<Line, SemanticError> {
    match tag {
        TagKind::Include => {
            let context_name = arguments.get(0).cloned().ok_or(SemanticError::MissingArgument("Context not specified in @include tag.".to_string()))?;
            Ok(Line::IncludeTag { context_name })
        },
        TagKind::Inline => {
            let snippet_name = arguments.get(0).cloned().ok_or(SemanticError::MissingArgument("Snippet not specified in @inline tag.".to_string()))?;
            Ok(Line::InlineTag { snippet_name })
        },
        TagKind::Answer => Ok(Line::AnswerTag),
        TagKind::Summary => {
            let context_name = arguments.get(0).cloned().ok_or(SemanticError::MissingArgument("Context not specified in @summary tag.".to_string()))?;
            Ok(Line::SummaryTag { context_name })
        },
    }
}

fn enrich_syntax_anchor_line(project: &Project, anchor: &Anchor) -> Result<Line, SemanticError> {
    match (anchor.kind.clone(), anchor.tag.clone()) {
        (AnchorKind::Inline, AnchorTag::Begin) => Ok(Line::InlineBeginAnchor { uuid: anchor.uid.to_string(), state: InlineState::load(project, &anchor.uid) }),
        (AnchorKind::Inline, AnchorTag::End) => Ok(Line::InlineEndAnchor { uuid: anchor.uid.to_string() }),
        (AnchorKind::Summary, AnchorTag::Begin) => Ok(Line::SummaryBeginAnchor { uuid: anchor.uid.to_string(), state: SummaryState::load(project, &anchor.uid) }),
        (AnchorKind::Summary, AnchorTag::End) => Ok(Line::SummaryEndAnchor { uuid: anchor.uid.to_string() }),
        (AnchorKind::Answer, AnchorTag::Begin) => Ok(Line::AnswerBeginAnchor { uuid: anchor.uid.to_string(), state: AnswerState::load(project, &anchor.uid) }),
        (AnchorKind::Answer, AnchorTag::End) => Ok(Line::AnswerEndAnchor { uuid: anchor.uid.to_string() }),
        _ => Err(SemanticError::InvalidAnchorFormat(anchor.to_string())),
    }
}

pub fn enrich_syntax_line(project: &Project, line: &types::Line) -> Result<Line, SemanticError> {
    match line {
       types::Line::Text(text) => Ok(Line::Text(text.clone())),
       types::Line::Tagged{ tag, parameters, arguments } => enrich_syntax_tagged_line(project, tag, parameters, arguments),
       types::Line::Anchor(anchor) => enrich_syntax_anchor_line(project, anchor),
    }
}

pub fn enrich_syntax_document(project: &Project, lines: &Vec<types::Line>) -> Result<Vec<Line>, SemanticError> {
    lines.iter().map(|line| enrich_syntax_line(project, line)).collect()
}