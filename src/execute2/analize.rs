
use super::{tag_answer::AnswerState, tag_inline::InlineState, tag_task::TaskState, Result};
use crate::ast2::{parse_document, Anchor, CommandKind, Content};
use crate::file::FileAccessor;
use crate::path::PathResolver;
use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;
use uuid::Uuid;

/// Enum that aggregates the different types of state specific for each anchor.
#[derive(Debug)]
pub enum AnchorState {
    Answer(AnswerState),
    Inline(InlineState),
    Task(TaskState),
}

/// Contains the complete analysis of a single anchor.
#[derive(Debug)]
pub struct AnchorAnalysis {
    pub anchor: Anchor,
    pub state: AnchorState,
}

/// Final result of the analysis of a context.
#[derive(Debug)]
pub struct ContextAnalysis {
    pub anchors: HashMap<Uuid, AnchorAnalysis>,
}

pub fn analyze_context(
    file_access: Arc<dyn FileAccessor>,
    path_res: Arc<dyn PathResolver>,
    context_name: &str,
) -> Result<ContextAnalysis> {
    let analyzer = Analyzer {
        file_access,
        path_res,
    };
    analyzer.run(context_name)
}

struct Analyzer {
    file_access: Arc<dyn FileAccessor>,
    path_res: Arc<dyn PathResolver>,
}

impl Analyzer {
    fn run(&self, context_name: &str) -> Result<ContextAnalysis> {
        let path = self.path_res.resolve_input_file(context_name)?;
        let content = self.file_access.read_file(&path)?;
        let doc = parse_document(&content)?;

        let mut analysis = ContextAnalysis {
            anchors: HashMap::new(),
        };

        for node in doc.content {
            if let Content::Anchor(anchor) = node {
                if let Some(state) = self.extract_anchor_state(&anchor)? {
                    let anchor_analysis = AnchorAnalysis {
                        anchor: anchor.clone(),
                        state,
                    };
                    analysis.anchors.insert(anchor.uuid, anchor_analysis);
                }
            }
        }

        Ok(analysis)
    }

    fn extract_anchor_state(&self, anchor: &Anchor) -> Result<Option<AnchorState>> {
        match anchor.command {
            CommandKind::Answer => {
                let state = self.load_state::<AnswerState>(anchor.command, &anchor.uuid)?;
                Ok(Some(AnchorState::Answer(state)))
            }
            CommandKind::Inline => {
                let state = self.load_state::<InlineState>(anchor.command, &anchor.uuid)?;
                Ok(Some(AnchorState::Inline(state)))
            }
            CommandKind::Task => {
                let state = self.load_state::<TaskState>(anchor.command, &anchor.uuid)?;
                Ok(Some(AnchorState::Task(state)))
            }
            _ => Ok(None),
        }
    }

    /// TODO execute e analize hanno le seguenti in comune, fare trait con default impl? o fare un oggetto che li contenga? magari un oggetto in root crate, visto che e' funzionalita' abbastanza comune

    /// Constructs the file system path for storing the state of a dynamic command.
    ///
    /// Dynamic commands (like `@answer` or `@repeat`) persist their state in JSON files
    /// within a metadata directory associated with their UUID.
    ///
    /// # Arguments
    ///
    /// * `command` - The [`CommandKind`] of the dynamic command.
    /// * `uuid` - The unique identifier of the command instance.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `PathBuf` to the state file.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::PathResolutionError`] if the metadata path cannot be resolved.
    fn get_state_path(&self, command: CommandKind, uuid: &Uuid) -> Result<PathBuf> {
        let meta_path = self
            .path_res
            .resolve_metadata(&command.to_string(), &uuid)?;
        let state_path = meta_path.join("state.json");
        Ok(state_path)
    }

    /// Loads the state of a dynamic command from its associated JSON file.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type into which the JSON state should be deserialized. Must implement `serde::de::DeserializeOwned`.
    ///
    /// # Arguments
    ///
    /// * `command` - The [`CommandKind`] of the dynamic command.
    /// * `uuid` - The unique identifier of the command instance.
    ///
    /// # Returns
    ///
    /// A `Result` containing the deserialized state object of type `T`.
    ///
    /// # Errors
    ///
    /// Returns [`ExecuteError::PathResolutionError`] if the state path cannot be resolved.
    /// Returns [`ExecuteError::IoError`] if the file cannot be read.
    /// Returns [`ExecuteError::JsonError`] if the file content is not valid JSON or cannot be deserialized into `T`.
    pub fn load_state<T: serde::de::DeserializeOwned>(
        &self,
        command: CommandKind,
        uuid: &Uuid,
    ) -> Result<T> {
        let state_path = self.get_state_path(command, uuid)?;
        let state = self.file_access.read_file(&state_path)?;
        let state: T = serde_json::from_str(&state)?;
        Ok(state)
    }
}

