use serde::{Deserialize, Serialize};
use uuid::{uuid, Uuid};

use super::arguments::Arguments;
use super::command_kind::CommandKind;
use super::parameters::Parameters;
use super::range::Range;

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
    /// A status tag useful to visualize things
    pub status: Option<String>,
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
        status: Option<String>,
        parameters: &Parameters,
        arguments: &Arguments,
    ) -> (Anchor, Anchor) {
        let uuid = Uuid::new_v4();
        let begin = Anchor {
            command,
            uuid,
            kind: AnchorKind::Begin,
            status: status,
            parameters: parameters.clone(),
            arguments: arguments.clone(),
            range: Range::null(),
        };
        let end = Anchor {
            command,
            uuid,
            kind: AnchorKind::End,
            status: None,
            parameters: Parameters::new(),
            arguments: Arguments::new(),
            range: Range::null(),
        };
        (begin, end)
    }
    /// Create a new invalid anchor
    pub fn invalid() -> Self {
        Anchor {
            command: CommandKind::Tag,
            uuid: uuid!("00000000-0000-0000-0000-000000000000"),
            kind: AnchorKind::Begin,
            status: None,
            parameters: Parameters::new(),
            arguments: Arguments::new(),
            range: Range::null(),
        }
    }
    /// Create a new anchor from an existing one taking values from new Parameters and Arguments
    pub fn update(&self, parameters: &Parameters, arguments: &Arguments) -> Self {
        let mut anchor = self.clone();
        anchor.parameters = anchor.parameters.update(parameters);
        anchor.arguments = anchor.arguments.update(arguments);
        anchor
    }
    /// Mutate an anchor into another with different status
    pub fn set_status(mut self, new_status: String) -> Self {
        self.status = Some(new_status);
        self
    }
}

impl ToString for Anchor {
    fn to_string(&self) -> String {
        format!(
            "<!-- {}-{}:{} {} {} {} -->",
            self.command.to_string(),
            self.uuid.to_string(),
            self.kind.to_string(),
            match &self.status {
                None => format!(""),
                Some(x) => format!("+{}+", x),
            },
            self.parameters.to_string(),
            self.arguments.to_string(),
        )
    }
}
