use std::fmt::{self, Display, Formatter};

use super::types::{Anchor, AnchorTag, Line, LineKind, TagKind};

impl Display for Line {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)?;
        if let Some(anchor) = &self.anchor {
            write!(f, " {}", anchor)?;
        }
        Ok(())
    }
}

impl Display for LineKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LineKind::Text(s) => write!(f, "{}", s),
            LineKind::Tagged {
                tag,
                parameters,
                arguments,
            } => {
                write!(f, " @{}", tag)?;
                if !parameters.is_empty() {
                    write!(f, "[")?;
                    let mut first = true;
                    for (key, value) in parameters {
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
                    for arg in arguments {
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
        }
    }
}

impl Display for Anchor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.tag == AnchorTag::None {
            write!(f, "<!-- {}-{} -->", self.kind, self.uid)
        } else {
            write!(f, "<!-- {}-{}:{} -->", self.kind, self.uid, self.tag)
        }
    }
}

impl Display for AnchorTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AnchorTag::None => write!(f, ""),
            AnchorTag::Begin => write!(f, "begin"),
            AnchorTag::End => write!(f, "end"),
        }
    }
}

impl Display for TagKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TagKind::Include => write!(f, "include"),
            TagKind::Inline => write!(f, "inline"),
            TagKind::Answer => write!(f, "answer"),
            TagKind::Summary => write!(f, "summary"),
        }
    }
}

pub fn format_document(lines: Vec<Line>) -> String {
    lines
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<String>>()
        .join("\n")
}