use std::collections::HashMap;
use uuid::Uuid;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AnchorKind {
    Inline,
    Answer,
}

impl fmt::Display for AnchorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnchorKind::Inline => write!(f, "inline"),
            AnchorKind::Answer => write!(f, "answer"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AnchorTag {
    None,
    Begin,
    End,
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

#[derive(Debug, PartialEq, Eq, Clone)]
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TaggedLine {
    pub tag: TagKind,
    pub parameters: HashMap<String, String>,
    pub arguments: Vec<String>,
}

impl fmt::Display for TaggedLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}", self.tag)?;
        if !self.parameters.is_empty() {
            write!(f, "[")?;
            let mut first = true;
            for (key, value) in &self.parameters {
                if !first {
                    write!(f, "; ")?;
                }
                write!(f, "{}={}", key, value)?;
                first = false;
            }
            write!(f, "]")?;
        }
        if !self.arguments.is_empty() {
            write!(f, " ")?;
            let mut first = true;
            for arg in &self.arguments {
                if !first {
                    write!(f, " ")?;
                }
                if arg.contains(' ') || arg.contains('"') {
                    // Simple quoting for now, proper escaping will be handled in parser
                    write!(f, "\"{}\"", arg.replace('"', "\""))?;
                } else {
                    write!(f, "{}", arg)?;
                }
                first = false;
            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LineKind {
    Text(String),
    Tagged(TaggedLine),
}

impl fmt::Display for LineKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LineKind::Text(s) => write!(f, "{}", s),
            LineKind::Tagged(tagged_line) => write!(f, "{}", tagged_line),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Line {
    pub kind: LineKind,
    pub anchor: Option<Anchor>,
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)?;
        if let Some(anchor) = &self.anchor {
            // Ensure there's a space before the anchor if the line kind isn't empty
            if !self.kind.to_string().is_empty() {
                write!(f, " ")?;
            }
            write!(f, "{}", anchor)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AstError {
    InvalidAnchorFormat,
    InvalidUuid(uuid::Error),
    InvalidParameterFormat,
    InvalidTagFormat,
    MissingTagName,
    UnclosedQuote,
    EmptyParameterKey,
    InvalidParameterKey(String),
    EmptyParameterValue,
    InvalidAnchorKind(String),
    InvalidAnchorTag(String),
    InvalidTagKind(String),
}

impl fmt::Display for AstError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AstError::InvalidAnchorFormat => write!(f, "Invalid anchor format"),
            AstError::InvalidUuid(e) => write!(f, "Invalid UUID: {}", e),
            AstError::InvalidParameterFormat => write!(f, "Invalid parameter format"),
            AstError::InvalidTagFormat => write!(f, "Invalid tag format"),
            AstError::MissingTagName => write!(f, "Missing tag name after '@'"),
            AstError::UnclosedQuote => write!(f, "Unclosed quote in arguments or parameters"),
            AstError::EmptyParameterKey => write!(f, "Empty parameter key"),
            AstError::InvalidParameterKey(key) => write!(f, "Invalid parameter key: '{}'", key),
            AstError::EmptyParameterValue => write!(f, "Empty parameter value"),
            AstError::InvalidAnchorKind(kind) => write!(f, "Invalid anchor kind: '{}'", kind),
            AstError::InvalidAnchorTag(tag) => write!(f, "Invalid anchor tag: '{}'", tag),
            AstError::InvalidTagKind(kind) => write!(f, "Invalid tag kind: '{}'", kind),
        }
    }
}

impl std::error::Error for AstError {}