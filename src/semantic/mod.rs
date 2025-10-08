mod context;
pub use context::*;

use crate::project::Project;

use crate::syntax::types::{Anchor, AnchorKind, AnchorTag, Line as SyntaxLine, TagKind};
use std::collections::HashMap;

use thiserror::Error;
use tracing::error;
use uuid::Uuid;

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

impl Line {
    pub fn new_inline_anchors(uuid: Uuid) -> Vec<Self> {
        vec![
            Line::InlineBeginAnchor { uuid: uuid.clone() },
            Line::InlineEndAnchor { uuid },
        ]
    }
    pub fn new_summary_anchors(uuid: Uuid) -> Vec<Self> {
        vec![
            Line::SummaryBeginAnchor { uuid: uuid.clone() },
            Line::SummaryEndAnchor { uuid },
        ]
    }
    pub fn new_answer_anchors(uuid: Uuid) -> Vec<Self> {
        vec![
            Line::AnswerBeginAnchor { uuid: uuid.clone() },
            Line::AnswerEndAnchor { uuid },
        ]
    }
    pub fn is_begin_anchor(&self) -> bool {
        matches!(
            self,
            Line::InlineBeginAnchor { .. }
                | Line::SummaryBeginAnchor { .. }
                | Line::AnswerBeginAnchor { .. }
        )
    }
    pub fn is_end_anchor(&self) -> bool {
        matches!(
            self,
            Line::InlineEndAnchor { .. }
                | Line::SummaryEndAnchor { .. }
                | Line::AnswerEndAnchor { .. }
        )
    }

    pub fn get_uid(&self) -> Uuid {
        match self {
            Line::InlineBeginAnchor { uuid } => *uuid,
            Line::InlineEndAnchor { uuid } => *uuid,
            Line::SummaryBeginAnchor { uuid } => *uuid,
            Line::SummaryEndAnchor { uuid } => *uuid,
            Line::AnswerBeginAnchor { uuid } => *uuid,
            Line::AnswerEndAnchor { uuid } => *uuid,
            _ => panic!("get_uid called on a non-anchor line"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Line {
    Text(String),
    InlineTag { snippet_name: String },
    SummaryTag { context_name: String },
    AnswerTag,
    IncludeTag { context_name: String },
    InlineBeginAnchor { uuid: Uuid },
    InlineEndAnchor { uuid: Uuid },
    SummaryBeginAnchor { uuid: Uuid },
    SummaryEndAnchor { uuid: Uuid },
    AnswerBeginAnchor { uuid: Uuid },
    AnswerEndAnchor { uuid: Uuid },
}

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Line::Text(s) => write!(f, "{}", s),
            Line::InlineTag { snippet_name } => {
                let syntax_line = SyntaxLine::Tagged {
                    tag: TagKind::Inline,
                    parameters: HashMap::new(),
                    arguments: vec![snippet_name.clone()],
                };
                write!(f, "{}", syntax_line)
            }
            Line::SummaryTag { context_name } => {
                let syntax_line = SyntaxLine::Tagged {
                    tag: TagKind::Summary,
                    parameters: HashMap::new(),
                    arguments: vec![context_name.clone()],
                };
                write!(f, "{}", syntax_line)
            }
            Line::AnswerTag => {
                let syntax_line = SyntaxLine::Tagged {
                    tag: TagKind::Answer,
                    parameters: HashMap::new(),
                    arguments: Vec::new(),
                };
                write!(f, "{}", syntax_line)
            }
            Line::IncludeTag { context_name } => {
                let syntax_line = SyntaxLine::Tagged {
                    tag: TagKind::Include,
                    parameters: HashMap::new(),
                    arguments: vec![context_name.clone()],
                };
                write!(f, "{}", syntax_line)
            }
            Line::InlineBeginAnchor { uuid } => {
                let anchor = Anchor {
                    kind: AnchorKind::Inline,
                    uid: *uuid,
                    tag: AnchorTag::Begin,
                };
                write!(f, "{}", SyntaxLine::Anchor(anchor))
            }
            Line::InlineEndAnchor { uuid } => {
                let anchor = Anchor {
                    kind: AnchorKind::Inline,
                    uid: *uuid,
                    tag: AnchorTag::End,
                };
                write!(f, "{}", SyntaxLine::Anchor(anchor))
            }
            Line::SummaryBeginAnchor { uuid } => {
                let anchor = Anchor {
                    kind: AnchorKind::Summary,
                    uid: *uuid,
                    tag: AnchorTag::Begin,
                };
                write!(f, "{}", SyntaxLine::Anchor(anchor))
            }
            Line::SummaryEndAnchor { uuid } => {
                let anchor = Anchor {
                    kind: AnchorKind::Summary,
                    uid: *uuid,
                    tag: AnchorTag::End,
                };
                write!(f, "{}", SyntaxLine::Anchor(anchor))
            }
            Line::AnswerBeginAnchor { uuid } => {
                let anchor = Anchor {
                    kind: AnchorKind::Answer,
                    uid: *uuid,
                    tag: AnchorTag::Begin,
                };
                write!(f, "{}", SyntaxLine::Anchor(anchor))
            }
            Line::AnswerEndAnchor { uuid } => {
                let anchor = Anchor {
                    kind: AnchorKind::Answer,
                    uid: *uuid,
                    tag: AnchorTag::End,
                };
                write!(f, "{}", SyntaxLine::Anchor(anchor))
            }
        }
    }
}

fn enrich_syntax_tagged_line(
    _project: &Project,
    tag: &TagKind,
    _parameters: &HashMap<String, String>,
    arguments: &Vec<String>,
) -> std::result::Result<Line, SemanticError> {
    match tag {
        TagKind::Include => {
            let context_name = arguments
                .get(0)
                .cloned()
                .ok_or(SemanticError::MissingArgument(
                    "Context not specified in @include tag.".to_string(),
                ))?;
            Ok(Line::IncludeTag { context_name })
        }
        TagKind::Inline => {
            let snippet_name = arguments
                .get(0)
                .cloned()
                .ok_or(SemanticError::MissingArgument(
                    "Snippet not specified in @inline tag.".to_string(),
                ))?;
            Ok(Line::InlineTag { snippet_name })
        }
        TagKind::Answer => Ok(Line::AnswerTag),
        TagKind::Summary => {
            let context_name = arguments
                .get(0)
                .cloned()
                .ok_or(SemanticError::MissingArgument(
                    "Context not specified in @summary tag.".to_string(),
                ))?;
            Ok(Line::SummaryTag { context_name })
        }
    }
}

fn enrich_syntax_anchor_line(
    _project: &Project,
    anchor: &Anchor,
) -> std::result::Result<Line, SemanticError> {
    match (anchor.kind.clone(), anchor.tag.clone()) {
        (AnchorKind::Inline, AnchorTag::Begin) => Ok(Line::InlineBeginAnchor { uuid: anchor.uid }),
        (AnchorKind::Inline, AnchorTag::End) => Ok(Line::InlineEndAnchor { uuid: anchor.uid }),
        (AnchorKind::Summary, AnchorTag::Begin) => {
            Ok(Line::SummaryBeginAnchor { uuid: anchor.uid })
        }
        (AnchorKind::Summary, AnchorTag::End) => Ok(Line::SummaryEndAnchor { uuid: anchor.uid }),
        (AnchorKind::Answer, AnchorTag::Begin) => Ok(Line::AnswerBeginAnchor { uuid: anchor.uid }),
        (AnchorKind::Answer, AnchorTag::End) => Ok(Line::AnswerEndAnchor { uuid: anchor.uid }),
        _ => Err(SemanticError::InvalidAnchorFormat(anchor.to_string())),
    }
}
pub fn enrich_syntax_line(
    project: &Project,
    line: &SyntaxLine,
) -> std::result::Result<Line, SemanticError> {
    match line {
        SyntaxLine::Text(text) => Ok(Line::Text(text.clone())),
        SyntaxLine::Tagged {
            tag,
            parameters,
            arguments,
        } => enrich_syntax_tagged_line(project, tag, parameters, arguments),
        SyntaxLine::Anchor(anchor) => enrich_syntax_anchor_line(project, anchor),
    }
}

pub fn enrich_syntax_document(
    project: &Project,
    lines: &Vec<SyntaxLine>,
) -> std::result::Result<Vec<Line>, SemanticError> {
    lines
        .iter()
        .map(|line| enrich_syntax_line(project, line))
        .collect()
}

pub fn parse_document(
    project: &Project,
    content: &str,
) -> std::result::Result<Vec<Line>, SemanticError> {
    let syntax_lines = crate::syntax::parser::parse_document(content)
        .map_err(|e| SemanticError::Generic(e.to_string()))?;
    enrich_syntax_document(project, &syntax_lines)
}

pub fn format_document(lines: &[Line]) -> String {
    lines
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<String>>()
        .join("\n")
}
