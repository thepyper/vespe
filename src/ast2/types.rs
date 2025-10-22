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

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        self.offset.cmp(other.offset)
    }
}

#[derive(Debug, Clone, Copy)]
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

pub struct Text {
    pub content: String,
    pub range: Range,
}

#[derive(Debug, PartialEq)]
pub enum CommandKind {
    Tag, // for debug purpose
    Include,
    Inline,
    Answer,
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

impl Anchor {
    pub fn new_couple<T>(parameters: &Parameters, arguments: &Arguments) -> (Anchor, Anchor) {
        let uuid = Uuid::new();
        let begin = Anchor {
            command: T,
            uuid: uuid.clone(),
            kind: AnchorKind::Begin,
            parameters: parameters.clone(),
            arguments: arguments.clone(),
            range: Range::null(),
        };
        let end = Anchor {
            command: T,
            uuid: uuid,
            kind: AnchorKind::End,
            parameters: Parameters::new(),
            arguments: Arguments::new(),
            range: Range::null(),
        };
        (begin, end)
    }
    
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
