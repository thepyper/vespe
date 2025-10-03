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

use std::fmt;

#[derive(Debug, PartialEq)]
pub struct AnchorData {
    pub kind: AnchorKind,
    pub uid: Uuid,
    pub data: String,
}

impl fmt::Display for AnchorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AnchorKind::Inline => write!(f, "inline"),
            AnchorKind::Answer => write!(f, "answer"),
        }
    }
}

impl fmt::Display for AnchorData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<!-- {}-{}:{} -->", self.kind, self.uid, self.data)
    }
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
