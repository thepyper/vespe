use crate::project::Project;
use crate::syntax::types::{self, Anchor, AnchorKind, AnchorTag, TagKind};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use thiserror::Error;
use crate::execute::inject::InlineState;

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
    #[error("Invalid anchor format: {0}")]
    InvalidAnchorFormat(String),
}

pub fn save_state_to_metadata<T>(
    project: &Project,
    anchor_kind: AnchorKind,
    uuid: &Uuid,
    state: &T,
) -> std::result::Result<(), SemanticError>
where
    T: serde::Serialize,
{
    let metadata_dir = project.resolve_metadata(anchor_kind.to_string().as_str(), uuid).map_err(SemanticError::AnyhowError)?;
    std::fs::create_dir_all(&metadata_dir)?;
    let state_path = metadata_dir.join("state.json");
    let serialized = serde_json::to_string_pretty(state)?;
    std::fs::write(&state_path, serialized)?;
    Ok(())
}

pub fn load_state_from_metadata<T>(
    project: &Project,
    anchor_kind: &AnchorKind,
    uid: &Uuid,
) -> std::result::Result<T, SemanticError>
where
    T: for<'de> serde::Deserialize<'de> + Default,
{
    let metadata_dir = project.resolve_metadata(anchor_kind.to_string().as_str(), uid)
        .map_err(SemanticError::AnyhowError)?;
    let state_path = metadata_dir.join("state.json");

    match std::fs::read_to_string(&state_path) {
        Ok(content) => Ok(serde_json::from_str(&content)?),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(T::default()),
        Err(e) => Err(SemanticError::IoError(e)),
    }
}

impl Line {
    pub fn save_state(&self, project: &Project) -> std::result::Result<(), SemanticError> {
        match self {
            Line::InlineBeginAnchor { uuid, state } => {
                save_state_to_metadata(project, AnchorKind::Inline, uuid, state)?;
                Ok(())
            }
            Line::SummaryBeginAnchor { uuid, state } => {
                save_state_to_metadata(project, AnchorKind::Summary, uuid, state)?;
                Ok(())
            }
            Line::AnswerBeginAnchor { uuid, state } => {
                save_state_to_metadata(project, AnchorKind::Answer, uuid, state)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
impl InlineState {
    pub fn load(project: &Project, uid: &Uuid) -> Self {
        load_state_from_metadata(project, &AnchorKind::Inline, uid).unwrap_or_default()
    }

    pub fn save(&self, project: &Project, uid: &Uuid) -> std::result::Result<(), SemanticError> {
        save_state_to_metadata(project, AnchorKind::Inline, uid, self)
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
        load_state_from_metadata(project, &AnchorKind::Summary, uid).unwrap_or_default()
    }

    pub fn save(&self, project: &Project, uid: &Uuid) -> std::result::Result<(), SemanticError> {
        save_state_to_metadata(project, AnchorKind::Summary, uid, self)
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
        load_state_from_metadata(project, &AnchorKind::Answer, uid).unwrap_or_default()
    }

    pub fn save(&self, project: &Project, uid: &Uuid) -> std::result::Result<(), SemanticError> {
        save_state_to_metadata(project, AnchorKind::Answer, uid, self)
    }
}

#[derive(Debug)]
pub enum Line {
    Text(String),
    InlineTag { snippet_name: String },
    SummaryTag { context_name: String },
    AnswerTag,
    IncludeTag { context_name: String },
    InlineBeginAnchor { uuid: Uuid, state: InlineState },
    InlineEndAnchor { uuid: Uuid },
    SummaryBeginAnchor { uuid: Uuid, state: SummaryState },
    SummaryEndAnchor { uuid: Uuid },
    AnswerBeginAnchor { uuid: Uuid, state: AnswerState },
    AnswerEndAnchor { uuid: Uuid },
}

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Line::Text(s) => write!(f, "{}", s),
            Line::InlineTag { snippet_name } => write!(f, "@inline{{{}}}", snippet_name),
            Line::SummaryTag { context_name } => write!(f, "@summary{{{}}}", context_name),
            Line::AnswerTag => write!(f, "@answer"),
            Line::IncludeTag { context_name } => write!(f, "@include{{{}}}", context_name),
            Line::InlineBeginAnchor { uuid, state } => write!(f, "[[inline_begin:{} (state: {:?})]]", uuid, state),
            Line::InlineEndAnchor { uuid } => write!(f, "[[inline_end:{}]]", uuid),
            Line::SummaryBeginAnchor { uuid, state } => write!(f, "[[summary_begin:{} (state: {:?})]]", uuid, state),
            Line::SummaryEndAnchor { uuid } => write!(f, "[[summary_end:{}]]", uuid),
            Line::AnswerBeginAnchor { uuid, state } => write!(f, "[[answer_begin:{} (state: {:?})]]", uuid, state),
            Line::AnswerEndAnchor { uuid } => write!(f, "[[answer_end:{}]]", uuid),
        }
    }
}

fn enrich_syntax_tagged_line(project: &Project, tag: &TagKind, parameters: &HashMap<String, String>, arguments: &Vec<String>) -> std::result::Result<Line, SemanticError> {
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

fn enrich_syntax_anchor_line(project: &Project, anchor: &Anchor) -> std::result::Result<Line, SemanticError> {
    match (anchor.kind.clone(), anchor.tag.clone()) {
        (AnchorKind::Inline, AnchorTag::Begin) => Ok(Line::InlineBeginAnchor { uuid: anchor.uid, state: InlineState::load(project, &anchor.uid) }),
        (AnchorKind::Inline, AnchorTag::End) => Ok(Line::InlineEndAnchor { uuid: anchor.uid }),
        (AnchorKind::Summary, AnchorTag::Begin) => Ok(Line::SummaryBeginAnchor { uuid: anchor.uid, state: SummaryState::load(project, &anchor.uid) }),
        (AnchorKind::Summary, AnchorTag::End) => Ok(Line::SummaryEndAnchor { uuid: anchor.uid }),
        (AnchorKind::Answer, AnchorTag::Begin) => Ok(Line::AnswerBeginAnchor { uuid: anchor.uid, state: AnswerState::load(project, &anchor.uid) }),
        (AnchorKind::Answer, AnchorTag::End) => Ok(Line::AnswerEndAnchor { uuid: anchor.uid }),
        _ => Err(SemanticError::InvalidAnchorFormat(anchor.to_string())),
    }
}

pub fn enrich_syntax_line(project: &Project, line: &types::Line) -> std::result::Result<Line, SemanticError> {
    match line {
       types::Line::Text(text) => Ok(Line::Text(text.clone())),
       types::Line::Tagged{ tag, parameters, arguments } => enrich_syntax_tagged_line(project, tag, parameters, arguments),
       types::Line::Anchor(anchor) => enrich_syntax_anchor_line(project, anchor),
    }
}

pub fn enrich_syntax_document(project: &Project, lines: &Vec<types::Line>) -> std::result::Result<Vec<Line>, SemanticError> {
    lines.iter().map(|line| enrich_syntax_line(project, line)).collect()
}