use serde::{Deserialize, Serialize};

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
    /// A command to repeat a section (not fully implemented).
    Repeat,
    /// Set variables from there on
    Set,
    /// Forget previous context
    Forget,
    /// Allows writing things ignored by LLM
    Comment,
    /// Used to segment long tasks into steps
    Task,
    /// Used in tandem with task
    Done,
}


impl ToString for CommandKind {
    fn to_string(&self) -> String {
        match self {
            CommandKind::Tag => "tag",
            CommandKind::Include => "include",
            CommandKind::Inline => "inline",
            CommandKind::Answer => "answer",
            CommandKind::Repeat => "repeat",
            CommandKind::Set => "set",
            CommandKind::Forget => "forget",
            CommandKind::Comment => "comment",
            CommandKind::Task => "task",
            CommandKind::Done => "done",
        }
        .to_string()
    }
}