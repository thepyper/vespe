use std::collections::HashMap;
use std::path::PathBuf;
use serde_json::Value;
use uuid::Uuid;

pub type Parameters = HashMap<String, Value>;

#[derive(Debug, PartialEq)]
pub enum AnchorKind {
    Inline,
    Answer,
}

#[derive(Debug, PartialEq)]
pub struct AnchorData {
    pub kind: AnchorKind,
    pub uid: Uuid,
    pub data: String,
}

#[derive(Debug, PartialEq)]
pub enum LineKind {
    Text,
    Include { context: Context, parameters: Parameters },
    Inline { snippet: Snippet, parameters: Parameters },
    Answer { parameters: Parameters },
    Summary { context: Context, parameters: Parameters },
}

#[derive(Debug, PartialEq)]
pub struct Line {
    pub kind: LineKind,
    pub text: String,
    pub anchor: Option<AnchorData>,
}

#[derive(Debug, PartialEq)]
pub struct Context {
    pub path: PathBuf,
    pub lines: Vec<Line>,
}

#[derive(Debug, PartialEq)]
pub struct Snippet {
    pub path: PathBuf,
    pub lines: Vec<Line>,
}
