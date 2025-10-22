use uuid::Uuid;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
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

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        self.offset.cmp(other.offset)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct Range {
    pub begin: Position,
    pub end: Position,
}

impl Ord for Range {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.begin.cmp(other.begin) {
            Ordering::Equal => self.end.cmp(other.end),
            x => x                
        }
    }
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Text {
    pub content: String,
    pub range: Range,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum CommandKind {
    Tag, // for debug purpose
    Include,
    Inline,
    Answer,
    Derive,
    Repeat,
}

impl ToString for CommandKind {
    fn to_string(&self) -> String {
        match self {
            CommandKind::Tag => "tag",
            CommandKind::Include => "include",
            CommandKind::Inline => "inline",
            CommandKind::Answer => "answer",
            CommandKind::Derive => "derive",
            CommandKind::Repeat => "repeat",            
        }.to_string()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Argument {
    pub value: String,
    pub range: Range,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arguments {
    pub arguments: Vec<Argument>,
    pub range: Range,
}

impl Arguments {
    pub fn new() -> Self {
        Arguments {
            arguments: Vec::new(),
            range: Range::null(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub command: CommandKind,
    pub parameters: Parameters,
    pub arguments: Arguments,
    pub range: Range,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum AnchorKind {
    Begin,
    End,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Anchor {
    pub command: CommandKind,
    pub uuid: Uuid,
    pub kind: AnchorKind,
    pub parameters: Parameters,
    pub arguments: Arguments,
    pub range: Range,
}

impl Anchor {
    pub fn new_couple(command: CommandKind, parameters: &Parameters, arguments: &Arguments) -> (Anchor, Anchor) {
        let uuid = Uuid::new_v4();
        let begin = Anchor {
            command,
            uuid,
            kind: AnchorKind::Begin,
            parameters: parameters.clone(),
            arguments: arguments.clone(),
            range: Range::null(),
        };
        let end = Anchor {
            command,
            uuid,
            kind: AnchorKind::End,
            parameters: Parameters::new(),
            arguments: Arguments::new(),
            range: Range::null(),
        };
        (begin, end)
    }
    
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Content {
    Text(Text),
    Tag(Tag),
    Anchor(Anchor),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub content: Vec<Content>,
    pub range: Range,
}
