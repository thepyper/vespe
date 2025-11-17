use super::{ExecuteError, tag_answer::AnswerState, tag_inline::InlineState, tag_task::TaskState, Result};
use crate::ast2::{parse_document, Anchor, CommandKind, Content};
use crate::file::FileAccessor;
use crate::path::PathResolver;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

/// Represents the specific state associated with different types of dynamic anchors.
///
/// Each variant corresponds to a particular command kind (e.g., `@answer`, `@inline`, `@task`)
/// and holds the state object relevant to that command's execution lifecycle.
#[derive(Debug)]
pub enum AnchorState {
    Answer(AnswerState),
    Inline(InlineState),
    Task(TaskState),
}

/// Encapsulates the complete analysis of a single dynamic anchor found within a document.
///
/// This struct combines the parsed `Anchor` definition with its current execution state,
/// providing a comprehensive view of a dynamic directive's status.
#[derive(Debug)]
pub struct AnchorAnalysis {
    pub anchor: Anchor,
    pub state: AnchorState,
}

/// Represents the aggregated analysis result for all dynamic anchors within a given context.
///
/// It stores a map of `Uuid` to `AnchorAnalysis`, allowing for quick lookup and
/// management of individual anchor states during the execution process.
#[derive(Debug)]
pub struct ContextAnalysis {
    pub anchors: HashMap<Uuid, AnchorAnalysis>,
}

/// Analyzes a given context (document) to extract all dynamic anchors and their current states.
///
/// This function serves as the public entry point for initiating the analysis of a document.
/// It parses the document content, identifies all `Anchor` nodes, and attempts to load
/// the persisted state for each dynamic anchor (e.g., `@answer`, `@inline`, `@task`).
///
/// # Arguments
///
/// * `file_access` - An `Arc` to an object implementing `FileAccessor` for file system operations.
/// * `path_res` - An `Arc` to an object implementing `PathResolver` for resolving file paths.
/// * `context_name` - The name of the context (document) to analyze.
///
/// # Returns
///
/// A `Result` containing a `ContextAnalysis` struct, which holds a map of all
/// found dynamic anchors and their associated states.
///
/// # Errors
///
/// Returns an `ExecuteError` if:
/// - The `context_name` cannot be resolved to a valid file path.
/// - The document file cannot be read.
/// - The document content cannot be parsed.
/// - The state file for any dynamic anchor cannot be read or deserialized.
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

/// Internal struct responsible for performing the document analysis.
///
/// It holds references to `FileAccessor` and `PathResolver` to interact with the
/// file system and resolve paths during the analysis process.
struct Analyzer {
    file_access: Arc<dyn FileAccessor>,
    path_res: Arc<dyn PathResolver>,
}

impl Analyzer {
    /// Executes the analysis process for a given context name.
    ///
    /// This method resolves the input file, reads its content, parses the document
    /// to find all anchors, and then extracts the state for each dynamic anchor.
    ///
    /// # Arguments
    ///
    /// * `context_name` - The name of the context (document) to analyze.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `ContextAnalysis` struct.
    ///
    /// # Errors
    ///
    /// Returns an `ExecuteError` if:
    /// - The input file path cannot be resolved.
    /// - The file cannot be read.
    /// - The document content cannot be parsed.
    /// - The state for any dynamic anchor cannot be loaded.
    fn run(&self, context_name: &str) -> Result<ContextAnalysis> {
        let path = self
            .path_res
            .resolve_input_file(context_name)
            .map_err(|e| ExecuteError::Anyhow(e.into()))?;
        let content = self
            .file_access
            .read_file(&path)
            .map_err(|e| ExecuteError::Anyhow(e.into()))?;
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

    /// Extracts the specific state for a given anchor based on its command kind.
    ///
    /// This method dispatches to the appropriate state loading mechanism for `Answer`, `Inline`,
    /// and `Task` commands. If the anchor's command is not a dynamic command that
    /// maintains state, it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `anchor` - A reference to the `Anchor` for which to extract the state.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option<AnchorState>`. `Some(AnchorState)` if the anchor
    /// is dynamic and has a state, `None` otherwise.
    ///
    /// # Errors
    ///
    /// Returns an `ExecuteError` if the state file for the dynamic anchor cannot be
    /// read or deserialized.
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
            .resolve_metadata(&command.to_string(), &uuid)
            .map_err(|e| ExecuteError::Anyhow(e.into()))?;
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
        let state = self
            .file_access
            .read_file(&state_path)
            .map_err(|e| ExecuteError::Anyhow(e.into()))?;
        let state: T = serde_json::from_str(&state)?;
        Ok(state)
    }
}
