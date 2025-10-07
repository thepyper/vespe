use crate::project::Project;
use crate::syntax::types::{self, Anchor, AnchorKind, AnchorTag, TagKind};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;

// Error type for semantic processing
#[derive(Error, Debug)]
pub enum SemanticError {
    #[error("Semantic error: {0}")]
    Generic(String),
    #[error("Missing argument: {0}")]
    MissingArgument(String),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("UUID parsing error: {0}")]
    UuidParsingError(#[from] uuid::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON serialization/deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
}

pub fn save_state_to_metadata<T>(
    project: &Project,
    anchor_kind: AnchorKind,
    uuid: &Uuid,
    state: &T,
) -> Result<(), SemanticError>
where
    T: serde::Serialize,
{
    let metadata_dir = project.resolve_metadata(anchor_kind.to_string().as_str(), uuid)?;
    std::fs::create_dir_all(&metadata_dir)?;
    let state_path = metadata_dir.join("state.json");
    let serialized = serde_json::to_string_pretty(state)?;
    std::fs::write(&state_path, serialized)?;
    Ok(())
}

impl Line {
    pub fn save_state(&self, project: &Project) -> Result<(), SemanticError> {
        match self {
            Line::InlineBeginAnchor { uuid, state } => {
                let parsed_uuid = Uuid::parse_str(uuid)?;
                save_state_to_metadata(project, AnchorKind::Inline, &parsed_uuid, state)?;
                Ok(())
            }
            Line::SummaryBeginAnchor { uuid, state } => {
                let parsed_uuid = Uuid::parse_str(uuid)?;
                save_state_to_metadata(project, AnchorKind::Summary, &parsed_uuid, state)?;
                Ok(())
            }
            Line::AnswerBeginAnchor { uuid, state } => {
                let parsed_uuid = Uuid::parse_str(uuid)?;
                save_state_to_metadata(project, AnchorKind::Answer, &parsed_uuid, state)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
impl InlineState {
    pub fn load(project: &Project, uid: &Uuid) -> Self {
        load_state_from_metadata(project, &AnchorKind::Inline, uid)
    }

    pub fn save(&self, project: &Project, uid: &Uuid) -> Result<(), SemanticError> {
        save_state_to_metadata(project, &AnchorKind::Inline, uid, self)
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
        load_state_from_metadata(project, &AnchorKind::Summary, uid)
    }

    pub fn save(&self, project: &Project, uid: &Uuid) -> Result<(), SemanticError> {
        save_state_to_metadata(project, &AnchorKind::Summary, uid, self)
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
        load_state_from_metadata(project, &AnchorKind::Answer, uid)
    }

    pub fn save(&self, project: &Project, uid: &Uuid) -> Result<(), SemanticError> {
        save_state_to_metadata(project, &AnchorKind::Answer, uid, self)
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