use serde_json;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    /// 0-based character offset
    pub offset: usize,
    /// 1-based line
    pub line: usize,
    /// 1-based column
    pub column: usize,
}

impl Position {
    pub fn null() -> Self {
        Position {
            offset: 0,
            line: 0,
            column: 0,
        }
    }
    pub fn is_valid(&self) -> bool {
        (self.line > 0) && (self.column > 0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Range {
    pub begin: Position,
    pub end: Position,
}

impl Range {
    pub fn null() -> Self {
        Range {
            begin: Position::null(),
            end: Position::null(),
        }
    }
    pub fn is_valid(&self) -> bool {
        if !self.begin.is_valid() {
            false
        } else if !self.end.is_valid() {
            false
        } else {
            self.begin.offset <= self.end.offset
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CommandKind {
    Tag, // for debug purpose
    Include,
    Inline,
    Answer,
    Summarize,
    Derive,
    Repeat,
}

pub struct Parameters {
    pub parameters: serde_json::Map<String, serde_json::Value>,
    pub range: Range,
}

impl Parameters {
    pub fn new() -> Self {
        Parameters {
            parameters: serde_json::Map::new(),
            range: Range::null(),
        }
    }
}

pub struct Argument {
    pub value: String,
    pub range: Range,
}

pub struct Arguments {
    pub arguments: Vec<Argument>,
    pub range: Range,
}

pub struct Tag {
    pub command: CommandKind,
    pub parameters: Parameters,
    pub arguments: Arguments,
    pub range: Range,
}

#[derive(Debug, PartialEq)]
pub enum AnchorKind {
    Begin,
    End,
}

pub struct Anchor {
    pub command: CommandKind,
    pub uuid: Uuid,
    pub kind: AnchorKind,
    pub parameters: Parameters,
    pub arguments: Arguments,
    pub range: Range,
}

pub struct Text {
    pub range: Range,
}

pub enum Content {
    Text(Text),
    Tag(Tag),
    Anchor(Anchor),
}

pub struct Document {
    pub content: Vec<Content>,
    pub range: Range,
}
