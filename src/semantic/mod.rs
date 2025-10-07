use crate::project::Project;
use crate::syntax::types;

use anyhow::{Result};
use serde_json;
use serde::{Deserialize, Serialize};

// Placeholder for InlineState, will need to be defined properly later
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InlineState {
    Default,
    // Add other states as needed
}

impl InlineState {
    pub fn load(project: &Project, uid: &Uuid) -> Self {        
        let state_path = project.resolve_metadata(AnchorKind::Inline, uid);
        if state_path.exists() {
            // TODO try parse with json
        }
        // If the state file does not exist, return the default state
        Self::default()
    }
    fn default() -> Self {
        InlineState::Default
    }
}

// TODO impl SummaryState and AnswerState similarly

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

fn enrich_syntax_tagged_line(project: &Project, tag: &syntax::TagKind, parameters: &HashMap<String, String>, arguments: &Vec<String>) -> Result<Line, SemanticError> {
    match tag {
        syntax::TagKind::Include => { Ok(Line::IncludeTag { context_name: arguments.get(0).ok_or(anyhow!("Context not specified in @include tag.")) }) },
        syntax::TagKind::Inline => { Ok(Line::InlineTag { snippet_name: arguments.get(0).ok_or(anyhow!("Snippet not specified in @inline tag.")) }) },
        syntax::TagKind::Answer => { Ok(Line::AnswerTag) },
        syntax::TagKind::Summary => { Ok(Line::SummaryTag { context_name: arguments.get(0).ok_or(anyhow!("Context not specified in @summary tag.")) }) },
        _ => Ok(Line::Text(format!("Unrecognized tag: @{}", tag))), // Fallback for unrecognized tags, or copy as text?? TODO
    }
}

fn enrich_syntax_anchor_line(project: &Project, anchor: &Anchor) -> Result<Line, SemanticError> {
    match (anchor.kind, anchor.tag) {
        (AnchorKind::Inline, AnchorTag::Begin) => Ok(Line::InlineBeginAnchor { uuid: anchor.uid.clone(), state: InlineState::load(project, anchor.uid) }), // TODO: parse state from tag parameters
        (AnchorKind::Inline, AnchorTag::End) => Ok(Line::InlineEndAnchor { uuid: anchor.uid.clone() }),
        (AnchorKind::Summary, AnchorTag::Begin) => Ok(Line::SummaryBeginAnchor { uuid: anchor.uid.clone(), state: SummaryState::load(project, anchor.uid) }), // TODO: parse state from tag parameters
        (AnchorKind::Summary, AnchorTag::End) => Ok(Line::SummaryEndAnchor { uuid: anchor.uid.clone() }),
        (AnchorKind::Answer, AnchorTag::Begin) => Ok(Line::AnswerBeginAnchor { uuid: anchor.uid.clone(), state: AnswerState::load(project, anchor.uid) }), // TODO: parse state from tag parameters
        (AnchorKind::Answer, AnchorTag::End) => Ok(Line::AnswerEndAnchor { uuid: anchor.uid.clone() }),
        _ => Err(SemanticError::InvalidAnchorFormat(anchor.to_string())),
    }   
}

pub fn enrich_syntax_line(project: &Project, line: &syntax::Line) -> Result<Line, SemanticError> {
    match line {
       syntax::Line::Text(text) => Ok(Line::Text(text.clone())),
       syntax::Line::Tagged{ tag, parameters, arguments } => enrich_syntax_tagged_line(project, tag, parameters, arguments),
       syntax::Line::Anchor(anchor) => enrich_syntax_anchor_line(project, anchor),
    }
}

pub fn enrich_syntax_document(project: &Project, lines: &Vec<types::Line>) -> Result<Vec<Line>, SemanticError> {
    lines.iter().map(|line| enrich_syntax_line(project, line)).collect()
}
