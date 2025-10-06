use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AnchorKind {
    Inline,
    Answer,
    Summary,
    // Add other specific kinds as they are defined
}

impl std::str::FromStr for AnchorKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "inline" => Ok(AnchorKind::Inline),
            "answer" => Ok(AnchorKind::Answer),
            "summary" => Ok(AnchorKind::Summary),
            _ => Err(format!("Unknown AnchorKind: {}", s)),
        }
    }
}

impl std::fmt::Display for AnchorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnchorKind::Inline => write!(f, "inline"),
            AnchorKind::Answer => write!(f, "answer"),
            AnchorKind::Summary => write!(f, "summary"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AnchorTag {
    None,
    Begin,
    End,
    // Add other specific tags as they are defined
}

impl std::str::FromStr for AnchorTag {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" => Ok(AnchorTag::None),
            "begin" => Ok(AnchorTag::Begin),
            "end" => Ok(AnchorTag::End),
            _ => Err(format!("Unknown AnchorTag: {}", s)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Anchor {
    pub kind: AnchorKind,
    pub uid: Uuid,
    pub tag: AnchorTag,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TagKind {
    Include,
    Inline,
    Answer,
    Summary,
    // Add other specific kinds as they are defined
}

impl std::str::FromStr for TagKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "include" => Ok(TagKind::Include),
            "inline" => Ok(TagKind::Inline),
            "answer" => Ok(TagKind::Answer),
            "summary" => Ok(TagKind::Summary),
            _ => Err(format!("Unknown TagKind: {}", s)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LineKind {
    Text(String),
    Tagged {
        tag: TagKind,
        parameters: HashMap<String, String>,
        arguments: Vec<String>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Line {
    pub kind: LineKind,
    pub anchor: Option<Anchor>,
}
