use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AnchorKind {
    Inline,
    Answer,
    Summary,
    // Add other well-defined anchor kinds here
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

impl fmt::Display for AnchorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    // Add other well-defined anchor tags here
}

impl std::str::FromStr for AnchorTag {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "begin" => Ok(AnchorTag::Begin),
            "end" => Ok(AnchorTag::End),
            "" => Ok(AnchorTag::None), // Handle empty string for None
            _ => Err(format!("Unknown AnchorTag: {}", s)),
        }
    }
}

impl fmt::Display for AnchorTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnchorTag::None => write!(f, ""),
            AnchorTag::Begin => write!(f, "begin"),
            AnchorTag::End => write!(f, "end"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Anchor {
    pub kind: AnchorKind,
    pub uid: Uuid,
    pub tag: AnchorTag,
}

impl fmt::Display for Anchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.tag == AnchorTag::None {
            write!(f, "<!-- {}-{} -->", self.kind, self.uid)
        } else {
            write!(f, "<!-- {}-{}:{} -->", self.kind, self.uid, self.tag)
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TagKind {
    Include,
    Inline,
    Answer,
    Summary,
    // Add other well-defined tag kinds here
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

impl fmt::Display for TagKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TagKind::Include => write!(f, "include"),
            TagKind::Inline => write!(f, "inline"),
            TagKind::Answer => write!(f, "answer"),
            TagKind::Summary => write!(f, "summary"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum LineKind {
    Text(String),
    Tagged {
        tag: TagKind,
        parameters: HashMap<String, String>,
        arguments: Vec<String>,
    },
}

impl fmt::Display for LineKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LineKind::Text(s) => write!(f, "{}", s),
            LineKind::Tagged {
                tag,
                parameters,
                arguments,
            } => {
                write!(f, "@{}", tag)?;
                if !parameters.is_empty() {
                    write!(f, "[")?;
                    let mut first = true;
                    for (key, value) in parameters.iter() {
                        if !first {
                            write!(f, "; ")?;
                        }
                        write!(f, "{}={}", key, value)?;
                        first = false;
                    }
                    write!(f, "]")?;
                }
                if !arguments.is_empty() {
                    write!(f, " ")?;
                    let mut first = true;
                    for arg in arguments.iter() {
                        if !first {
                            write!(f, " ")?;
                        }
                        if arg.contains(' ') || arg.contains('"') {
                            write!(f, "\"{}\"", arg.replace('"', "\\\""))?;
                        } else {
                            write!(f, "{}", arg)?;
                        }
                        first = false;
                    }
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Line {
    Text(String),
    Tagged {
        tag: TagKind,
        parameters: HashMap<String, String>,
        arguments: Vec<String>,
    },
    Anchor(Anchor),
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line::Text(s) => write!(f, "{}", s),
            Line::Tagged {
                tag,
                parameters,
                arguments,
            } => {
                write!(f, "@{}", tag)?;
                if !parameters.is_empty() {
                    write!(f, "[")?;
                    let mut first = true;
                    for (key, value) in parameters.iter() {
                        if !first {
                            write!(f, "; ")?;
                        }
                        write!(f, "{}={}", key, value)?;
                        first = false;
                    }
                    write!(f, "]")?;
                }
                if !arguments.is_empty() {
                    write!(f, " ")?;
                    let mut first = true;
                    for arg in arguments.iter() {
                        if !first {
                            write!(f, " ")?;
                        }
                        if arg.contains(' ') || arg.contains('"') {
                            write!(f, "\"{}\"", arg.replace('"', "\""))?;
                        } else {
                            write!(f, "{}", arg)?;
                        }
                        first = false;
                    }
                }
                Ok(())
            }
            Line::Anchor(anchor) => write!(f, "{}", anchor),
        }
    }
}

impl Line {
    pub fn get_text_content(&self) -> String {
        match self {
            Line::Text(s) => s.clone(),
            Line::Tagged { .. } => self.to_string(),
            Line::Anchor(anchor) => anchor.to_string(),
        }
    }

    pub fn get_anchor(&self) -> Option<&Anchor> {
        if let Line::Anchor(anchor) = self {
            Some(anchor)
        } else {
            None
        }
    }
}
