use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use uuid::Uuid;

/// Represents a specific point in the source document.
///
/// All locations are 1-based for user-facing error reporting, while the offset
/// is 0-based for internal use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct Position {
    /// 0-based character offset from the beginning of the file.
    pub offset: usize,
    /// 1-based line number.
    pub line: usize,
    /// 1-based column number.
    pub column: usize,
}

impl Position {
    /// Creates a "null" or invalid position. Useful for initializing ranges
    /// before a valid position is known.
    pub fn null() -> Self {
        Position {
            offset: 0,
            line: 0,
            column: 0,
        }
    }
    /// Checks if the position is valid (line and column are not zero).
    pub fn is_valid(&self) -> bool {
        (self.line > 0) && (self.column > 0)
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        self.offset.cmp(&other.offset)
    }
}

/// Represents a span of text in the source document, from a `begin` to an `end`
/// `Position`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct Range {
    pub begin: Position,
    pub end: Position,
}

impl Ord for Range {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.begin.cmp(&other.begin) {
            Ordering::Equal => self.end.cmp(&other.end),
            x => x,
        }
    }
}

impl Range {
    /// Creates a "null" or invalid range.
    pub fn null() -> Self {
        Range {
            begin: Position::null(),
            end: Position::null(),
        }
    }
    /// Checks if the range is valid.
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

/// A block of plain text content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Text {
    /// The raw string content of the text block.
    pub content: String,
    /// The location of the text block in the source document.
    pub range: Range,
}

/// Enumerates the different types of commands that can be invoked with tags or anchors.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CommandKind {
    /// A debug-only command.
    Tag,
    /// Includes content from another context file.
    Include,
    /// Inlines content from another file directly.
    Inline,
    /// Triggers a call to an external model to get an "answer".
    Answer,
    /// Derives new content by processing other contexts through a model.
    Derive,
    /// A command to repeat a section (not fully implemented).
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
        }
        .to_string()
    }
}

/// A collection of key-value parameters associated with a `Tag` or `Anchor`.
///
/// Parameters are parsed from a `[key=value, ...]` syntax.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameters {
    /// The map of parameter keys to their JSON values.
    pub parameters: serde_json::Map<String, serde_json::Value>,
    /// The location of the parameter block in the source document.
    pub range: Range,
}

impl Parameters {
    /// Creates a new, empty set of parameters.
    pub fn new() -> Self {
        Parameters {
            parameters: serde_json::Map::new(),
            range: Range::null(),
        }
    }
}

impl ToString for Parameters {
    fn to_string(&self) -> String {
        if self.parameters.is_empty() {
            String::new()
        } else {
            format!(
                "[{}]",
                self.parameters
                    .iter()
                    .map(|(x, y)| format!("{}={}", x, y.to_string()))
                    .collect::<Vec::<String>>()
                    .join(",")
            )
        }
    }
}

/// A single positional argument, typically a string literal or a "nude" string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Argument {
    /// The parsed value of the argument.
    pub value: String,
    /// The location of the argument in the source document.
    pub range: Range,
}

/// A collection of positional arguments associated with a `Tag` or `Anchor`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arguments {
    /// The vector of parsed arguments.
    pub arguments: Vec<Argument>,
    /// The location of the arguments block in the source document.
    pub range: Range,
}

impl Arguments {
    /// Creates a new, empty set of arguments.
    pub fn new() -> Self {
        Arguments {
            arguments: Vec::new(),
            range: Range::null(),
        }
    }
}

impl ToString for Arguments {
    fn to_string(&self) -> String {
        self.arguments
            .iter()
            .map(|x| x.value.clone())
            .collect::<Vec<String>>()
            .join(",")
    }
}

/// Represents a command tag, starting with `@`.
///
/// Example: `@include 'path/to/file.ctx'`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    /// The command to be executed.
    pub command: CommandKind,
    /// Key-value parameters for the command.
    pub parameters: Parameters,
    /// Positional arguments for the command.
    pub arguments: Arguments,
    /// The location of the tag in the source document.
    pub range: Range,
}

impl ToString for Tag {
    fn to_string(&self) -> String {
        format!(
            "@{} {} {}",
            self.command.to_string(),
            self.parameters.to_string(),
            self.arguments.to_string(),
        )
    }
}

/// The kind of an `Anchor`, indicating if it's the start or end of a block.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AnchorKind {
    Begin,
    End,
}

impl ToString for AnchorKind {
    fn to_string(&self) -> String {
        match self {
            AnchorKind::Begin => "begin".to_string(),
            AnchorKind::End => "end".to_string(),
        }
    }
}

/// Represents a processing anchor, which is an HTML-style comment `<!-- ... -->`.
///
/// Anchors are used for commands that have a body or represent a block of content
/// that can be dynamically modified. They are always paired (Begin/End) via a UUID.
///
/// Example: `<!-- answer-uuid:begin -->...<!-- answer-uuid:end -->`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Anchor {
    /// The command associated with the anchor.
    pub command: CommandKind,
    /// A unique identifier that links a `Begin` anchor with its `End` anchor.
    pub uuid: Uuid,
    /// Whether this is the `Begin` or `End` of the anchor pair.
    pub kind: AnchorKind,
    /// Key-value parameters, typically only present on the `Begin` anchor.
    pub parameters: Parameters,
    /// Positional arguments, typically only present on the `Begin` anchor.
    pub arguments: Arguments,
    /// The location of the anchor in the source document.
    pub range: Range,
}

impl Anchor {
    /// Creates a new pair of `Begin` and `End` anchors with a shared, new UUID.
    pub fn new_couple(
        command: CommandKind,
        parameters: &Parameters,
        arguments: &Arguments,
    ) -> (Anchor, Anchor) {
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
    fn to_string(&self) -> String {
        format!(
            "<!-- {}-{}:{} {} {} -->",
            self.command.to_string(),
            self.uuid.to_string(),
            self.kind.to_string(),
            self.parameters.to_string(),
            self.arguments.to_string(),
        )
    }
}

/// An enum representing any of the top-level content types in a document.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Content {
    /// A plain text block.
    Text(Text),
    /// A command tag (`@...`).
    Tag(Tag),
    /// A processing anchor (`<!-- ... -->`).
    Anchor(Anchor),
}

/// The root of the Abstract Syntax Tree, representing a fully parsed document.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Document {
    /// A vector of the top-level content items that make up the document.
    pub content: Vec<Content>,
    /// The range spanning the entire document.
    pub range: Range,
}

