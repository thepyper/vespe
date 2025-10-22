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
        self.offset.cmp(&other.offset)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct Range {
    pub begin: Position,
    pub end: Position,
}

impl Ord for Range {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.begin.cmp(&other.begin) {
            Ordering::Equal => self.end.cmp(&other.end),
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

impl ToString for Parameters {
    pub fn to_string(&self) -> String {
        if self.parameters.is_empty() {
            String::new()
        } else {
            format!("[{}]", self.parameters.iter().map(|(x,y)| format!("{}={}", x, y.to_string())).collect::<Vec::<String>>().join(","))
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

impl ToString for Arguments {
    pub fn to_string(&self) -> String {
        self.arguments.iter().map(|x| x.value.clone()).collect::<Vec::<String>>().join(",")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub command: CommandKind,
    pub parameters: Parameters,
    pub arguments: Arguments,
    pub range: Range,
}

impl ToString for Tag {
    pub to_string(&self) -> String {
        format!("@{} {} {}",
            tag.command.to_string(),
            tag.parameters.to_string(),
            tag.arguments.to_string(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AnchorKind {
    Begin,
    End,
}

impl ToString for AnchorKind {
    pub to_string(&self) -> String {
        match self {
            AnchorKind::Begin => "begin",
            AnchorKind::End => "end",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

impl ToString for Anchor {
    pub fn to_string(&self) -> String {
        format!("<!-- {}-{}:{} {} {} -->",
            self.command.to_string(),
            self.uuid.to_string(),
            self.kind.to_string(),
            self.parameters.to_string(),
            self.arguments.to_string(),            
        )
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
