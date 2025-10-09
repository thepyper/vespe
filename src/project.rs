use crate::execute::states::{AnswerState, InlineState, SummaryState};
use crate::semantic::Line;
use crate::semantic::SemanticError;
use crate::syntax::types::AnchorKind;
use crate::git::{Commit};
use crate::execute;
use crate::agent::ShellAgentCall;

use anyhow::anyhow;
use anyhow::Context as AnyhowContext;
use anyhow::Result;

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use tracing::debug;
use thiserror::Error;

use crate::config::{EditorInterface, ProjectConfig};
use crate::editor::{
    lockfile::FileBasedEditorCommunicator, DummyEditorCommunicator, EditorCommunicator,
};

#[derive(Error, Debug)]
pub enum ProjectError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON serialization/deserialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Editor(#[from] crate::editor::EditorError),
    #[error(transparent)]
    Lockfile(#[from] crate::editor::lockfile::LockfileError),
    #[error(transparent)]
    Semantic(#[from] crate::semantic::SemanticError),
    #[error("Project already initialized in this directory.")]
    AlreadyInitialized,
    #[error("No .ctx project found in the current directory or any parent directory.")]
    NotFound,
    #[error("Context file already exists: {0}")]
    ContextFileAlreadyExists(PathBuf),
    #[error("Snippet file already exists: {0}")]
    SnippetFileAlreadyExists(PathBuf),
    #[error("Failed to get parent directory")]
    ParentDirNotFound,
    #[error("Failed to parse snippet document: {0}")]
    SnippetParseError(String),
    #[error("Failed to read snippet file: {0}")]
    SnippetReadError(PathBuf, #[source] std::io::Error),
    #[error("Failed to write project config file: {0}")]
    ProjectConfigWriteError(#[source] std::io::Error),
    #[error("Failed to read project config file: {0}")]
    ProjectConfigReadError(#[source] std::io::Error),
    #[error("Failed to create metadata directory for anchor {0}-{1}: {2}")]
    MetadataDirCreationError(String, Uuid, PathBuf, #[source] std::io::Error),
    #[error("Failed to create parent directories for context file: {0}")]
    ContextParentDirCreationError(#[source] std::io::Error),
    #[error("Failed to create context file: {0}")]
    ContextFileCreationError(#[source] std::io::Error),
    #[error("Failed to create parent directories for snippet file: {0}")]
    SnippetParentDirCreationError(#[source] std::io::Error),
    #[error("Failed to create snippet file: {0}")]
    SnippetFileCreationError(#[source] std::io::Error),
    #[error("Failed to create .ctx directory: {0}")]
    CtxDirCreationError(#[source] std::io::Error),
    #[error("Failed to write .ctx_root file: {0}")]
    CtxRootFileWriteError(#[source] std::io::Error),
    #[error("Failed to canonicalize path: {0}")]
    CanonicalizePathError(#[source] std::io::Error),
    #[error("Failed to strip prefix: {0}")]
    StripPrefixError(#[from] std::path::StripPrefixError),
    #[error("Failed to read directory: {0}")]
    ReadDirError(#[source] std::io::Error),
    #[error("Failed to get entry from directory: {0}")]
    DirEntryError(#[source] std::io::Error),
    #[error("Git error: {0}")]
    Git(#[from] crate::git::GitError),
}

/*
#[derive(Debug)]
pub struct Context {
    pub name: String,
    pub content: Vec<Line>,
    pub includes: BTreeMap<usize, Context>, // line index to Context
    pub inlines: BTreeMap<usize, Snippet>,  // line index to Snippet
    pub summaries: BTreeMap<usize, Context>, // line index to Context
    pub answers: BTreeSet<usize>,           // line index
}
*/

#[derive(Debug)]
pub struct Snippet {
    pub name: String,
    pub content: Vec<Line>,
}

// ... (rest of the file)

const CTX_DIR_NAME: &str = ".ctx";
const CTX_ROOT_FILE_NAME: &str = ".ctx_root";
const METADATA_DIR_NAME: &str = ".meta";
const CONTEXTS_DIR_NAME: &str = "contexts";
const SNIPPETS_DIR_NAME: &str = "snippets";

#[derive(Debug)] // Add Debug trait for easy printing
pub struct ContextInfo {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug)] // Add Debug trait for easy printing
pub struct SnippetInfo {
    pub name: String,
    pub path: PathBuf,
}

pub struct Project {
    root_path: PathBuf,
    editor_communicator: Box<dyn EditorCommunicator>,
    project_config: ProjectConfig,
}

#[allow(dead_code)]
impl Project {
    pub fn init(path: &Path) -> Result<Project, ProjectError> {
        let ctx_dir = path.join(CTX_DIR_NAME);
        if ctx_dir.is_dir() && ctx_dir.join(CTX_ROOT_FILE_NAME).is_file() {
            return Err(ProjectError::AlreadyInitialized);
        }

        std::fs::create_dir_all(&ctx_dir).map_err(ProjectError::CtxDirCreationError)?;

        let ctx_root_file = ctx_dir.join(CTX_ROOT_FILE_NAME);
        std::fs::write(&ctx_root_file, "Feel The BuZZ!!")
            .map_err(ProjectError::CtxRootFileWriteError)?;

        let project = Project {
            root_path: path.canonicalize().map_err(ProjectError::CanonicalizePathError)?,
            editor_communicator: Box::new(DummyEditorCommunicator),
            project_config: ProjectConfig::default(),
        };

        if project.project_config.git_integration_enabled {
            let mut commit = Commit::new();
            commit.files.insert(ctx_dir);
            commit.files.insert(ctx_root_file);
            commit.commit("feat: Initialize .ctx project\nInitial commit of the .ctx project structure, including the .ctx directory and .ctx_root file.")?;
        }

        project.save_project_config()?;

        Ok(project)
    }

    pub fn find(path: &Path) -> Result<Project, ProjectError> {
        let mut current_path = path.to_path_buf();

        loop {
            let ctx_dir = current_path.join(CTX_DIR_NAME);
            if ctx_dir.is_dir() && ctx_dir.join(CTX_ROOT_FILE_NAME).is_file() {
                let root_path = current_path.canonicalize().map_err(ProjectError::CanonicalizePathError)?;
                let project_config_path = root_path
                    .join(CTX_DIR_NAME)
                    .join(METADATA_DIR_NAME)
                    .join("project_config.json");
                let project_config = Self::load_project_config(&project_config_path)?;

                let editor_path = ctx_dir.join(METADATA_DIR_NAME).join(".editor");
                let editor_communicator: Box<dyn EditorCommunicator> =
                    match project_config.editor_interface {
                        EditorInterface::VSCode => {
                            Box::new(FileBasedEditorCommunicator::new(&editor_path)?)
                        }
                        _ => Box::new(DummyEditorCommunicator),
                    };

                return Ok(Project {
                    root_path: root_path,
                    editor_communicator,
                    project_config,
                });
            }

            if !current_path.pop() {
                break;
            }
        }

        Err(ProjectError::NotFound)
    }

    pub fn project_home(&self) -> PathBuf {
        self.root_path.join(CTX_DIR_NAME)
    }

    pub fn metadata_home(&self) -> PathBuf {
        self.project_home().join(METADATA_DIR_NAME)
    }

    pub fn contexts_root(&self) -> PathBuf {
        self.project_home().join(CONTEXTS_DIR_NAME)
    }

    pub fn snippets_root(&self) -> PathBuf {
        self.project_home().join(SNIPPETS_DIR_NAME)
    }

    pub fn resolve_context(&self, name: &str) -> PathBuf {
        self.contexts_root().join(format!("{}.md", name))
    }

    pub fn resolve_snippet(&self, name: &str) -> PathBuf {
        self.snippets_root().join(format!("{}.md", name))
    }

    pub fn resolve_metadata(&self, anchor_kind: &str, uid: &Uuid) -> Result<PathBuf, ProjectError> {
        let anchor_metadata_dir =
            self.metadata_home()
                .join(format!("{}-{}", anchor_kind, uid.to_string()));
        std::fs::create_dir_all(&anchor_metadata_dir).map_err(|e| {
            ProjectError::MetadataDirCreationError(
                anchor_kind.to_string(),
                *uid,
                anchor_metadata_dir.clone(),
                e,
            )
        })?;
        Ok(anchor_metadata_dir)
    }

    pub fn project_config_path(&self) -> PathBuf {
        self.metadata_home().join("project_config.json")
    }

    pub fn create_context_file(
        &self,
        name: &str,
        initial_content: Option<String>,
    ) -> Result<PathBuf, ProjectError> {
        let file_path = self.contexts_root().join(format!("{}.md", name));
        if file_path.exists() {
            return Err(ProjectError::ContextFileAlreadyExists(file_path));
        }
        let parent_dir = file_path
            .parent()
            .ok_or(ProjectError::ParentDirNotFound)?;
        std::fs::create_dir_all(parent_dir)
            .map_err(ProjectError::ContextParentDirCreationError)?;
        let content = initial_content.unwrap_or_else(|| "".to_string());
        std::fs::write(&file_path, content).map_err(ProjectError::ContextFileCreationError)?;

        if self.project_config.git_integration_enabled {
            let mut commit = Commit::new();
            commit.files.insert(file_path.clone());
            commit.commit(&format!("feat: Create new context '{}'", name))?;            
        }

        Ok(file_path)
    }

    pub fn create_snippet_file(
        &self,
        name: &str,
        initial_content: Option<String>,
    ) -> Result<PathBuf, ProjectError> {
        let file_path = self.snippets_root().join(format!("{}.md", name));
        if file_path.exists() {
            return Err(ProjectError::SnippetFileAlreadyExists(file_path));
        }
        let parent_dir = file_path
            .parent()
            .ok_or(ProjectError::ParentDirNotFound)?;
        std::fs::create_dir_all(parent_dir)
            .map_err(ProjectError::SnippetParentDirCreationError)?;
        let content = initial_content.unwrap_or_else(|| "".to_string());
        std::fs::write(&file_path, content).map_err(ProjectError::SnippetFileCreationError)?;

        if self.project_config.git_integration_enabled {
            let mut commit = Commit::new();
            commit.files.insert(file_path.clone());
            commit.commit(&format!("feat: Create new snippet '{}'", name))?;            
         }

        Ok(file_path)
    }

    pub fn list_contexts(&self) -> Result<Vec<ContextInfo>, ProjectError> {
        let mut contexts = Vec::new();
        let contexts_root = self.contexts_root();

        if !contexts_root.exists() {
            return Ok(contexts); // Return empty if directory doesn't exist
        }

        let mut md_files = Vec::new();
        Self::collect_md_files_recursively(&contexts_root, &contexts_root, &mut md_files)?;

        for path in md_files {
            // Calculate the relative path from contexts_root to get the context name
            let relative_path = path.strip_prefix(&contexts_root).map_err(ProjectError::StripPrefixError)?;
            if let Some(file_stem) = relative_path.file_stem() {
                if let Some(name) = file_stem.to_str() {
                    contexts.push(ContextInfo {
                        name: name.to_string(),
                        path: path.clone(),
                    });
                }
            }
        }
        Ok(contexts)
    }

    pub fn list_snippets(&self) -> Result<Vec<SnippetInfo>, ProjectError> {
        let mut snippets = Vec::new();
        let snippets_root = self.snippets_root();

        if !snippets_root.exists() {
            return Ok(snippets); // Return empty if directory doesn't exist
        }

        let mut md_files = Vec::new();
        Self::collect_md_files_recursively(&snippets_root, &snippets_root, &mut md_files)?;

        for path in md_files {
            // Calculate the relative path from snippets_root to get the snippet name
            let relative_path = path.strip_prefix(&snippets_root).map_err(ProjectError::StripPrefixError)?;
            if let Some(file_stem) = relative_path.file_stem() {
                if let Some(name) = file_stem.to_str() {
                    snippets.push(SnippetInfo {
                        name: name.to_string(),
                        path: path.clone(),
                    });
                }
            }
        }
        Ok(snippets)
    }

    pub fn load_snippet(&self, name: &str) -> Result<Snippet, ProjectError> {
        let file_path = self.resolve_snippet(name);
        let content = std::fs::read_to_string(&file_path).map_err(|e| ProjectError::SnippetReadError(file_path.clone(), e))?;
        let lines = crate::semantic::parse_document(self, &content)
            .map_err(|e| ProjectError::SnippetParseError(e.to_string()))?;

        Ok(Snippet {
            name: name.to_string(),
            content: lines,
        })
    }

    pub fn save_project_config(&self) -> Result<(), ProjectError> {
        let config_path = self.project_config_path();
        let serialized = serde_json::to_string_pretty(&self.project_config).map_err(ProjectError::SerdeJson)?;
        std::fs::write(&config_path, serialized).map_err(ProjectError::ProjectConfigWriteError)?;
        Ok(())
    }

    pub fn load_project_config(project_config_path: &PathBuf) -> Result<ProjectConfig, ProjectError> {
        match std::fs::read_to_string(project_config_path) {
            Ok(content) => Ok(serde_json::from_str(&content).map_err(ProjectError::SerdeJson)?),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(ProjectConfig::default()),
            Err(e) => Err(ProjectError::ProjectConfigReadError(e)),
        }
    }

    fn save_state_to_metadata<T>(
        &self,
        anchor_kind: AnchorKind,
        uuid: &Uuid,
        state: &T,
        commit: &mut Commit,
    ) -> Result<(), ProjectError>
    where
        T: serde::Serialize,
    {
        let metadata_dir = self
            .resolve_metadata(anchor_kind.to_string().as_str(), uuid)?;
        std::fs::create_dir_all(&metadata_dir)?;
        let state_path = metadata_dir.join("state.json");
        let serialized = serde_json::to_string_pretty(state).map_err(ProjectError::SerdeJson)?;
        std::fs::write(&state_path, serialized).map_err(ProjectError::Io)?;
        commit.files.insert(state_path);
        Ok(())
    }

    fn load_state_from_metadata<T>(
        &self,
        anchor_kind: &AnchorKind,
        uid: &Uuid,
    ) -> Result<T, ProjectError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let metadata_dir = self
            .resolve_metadata(anchor_kind.to_string().as_str(), uid)?;
        let state_path = metadata_dir.join("state.json");

        match std::fs::read_to_string(&state_path) {
            Ok(content) => Ok(serde_json::from_str(&content).map_err(ProjectError::SerdeJson)?),
            Err(e) if e.kind() == ErrorKind::NotFound => Err(ProjectError::Io(e)), // Consider a more specific error if needed
            Err(e) => Err(ProjectError::Io(e)),
        }
    }

    pub fn save_inline_state(&self, uid: &Uuid, state: &InlineState, commit: &mut Commit) -> Result<(), ProjectError> {
        self.save_state_to_metadata(AnchorKind::Inline, uid, state, commit)
    }

    pub fn load_inline_state(&self, uid: &Uuid) -> Result<InlineState, ProjectError> {
        self.load_state_from_metadata(&AnchorKind::Inline, uid)
    }

    pub fn save_summary_state(&self, uid: &Uuid, state: &SummaryState, commit: &mut Commit) -> Result<(), ProjectError> {
        self.save_state_to_metadata(AnchorKind::Summary, uid, state, commit)
    }

    pub fn load_summary_state(&self, uid: &Uuid) -> Result<SummaryState, ProjectError> {
        self.load_state_from_metadata(&AnchorKind::Summary, uid)
    }

    pub fn save_answer_state(&self, uid: &Uuid, state: &AnswerState, commit: &mut Commit) -> Result<(), ProjectError> {
        self.save_state_to_metadata(AnchorKind::Answer, uid, state, commit)
    }

    pub fn load_answer_state(&self, uid: &Uuid) -> Result<AnswerState, ProjectError> {
        self.load_state_from_metadata(&AnchorKind::Answer, uid)
    }

    pub fn request_file_modification(&self, file_path: &PathBuf) -> Result<Uuid, ProjectError> {
        self.editor_communicator
            .request_file_modification(file_path).map_err(ProjectError::Editor)
    }

    pub fn notify_file_modified(&self, file_path: &PathBuf, uid: Uuid) -> Result<(), ProjectError> {
        self.editor_communicator
            .notify_file_modified(file_path, uid).map_err(ProjectError::Editor)
    }

    pub fn execute_context(
        &self,
        context_name: &str,
        agent: &ShellAgentCall,
    ) -> Result<(), ProjectError> {
        debug!("Project::execute_context called for: {}", context_name);
        let mut commit = Commit::new();
        execute::execute(self, context_name, agent, &mut commit)?;
        if self.project_config.git_integration_enabled {
            debug!("Attempting to commit for context: {}", context_name);
            commit.commit(&format!("feat: Executed context '{}'", context_name))?;           
        }
        Ok(())
    }

    fn collect_md_files_recursively(
        root: &Path,
        current_dir: &Path,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), ProjectError> {
        for entry in std::fs::read_dir(current_dir).map_err(ProjectError::ReadDirError)? {
            let entry = entry.map_err(ProjectError::DirEntryError)?;
            let path = entry.path();

            if path.is_dir() {
                Self::collect_md_files_recursively(root, &path, files)?;
            } else if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "md" {
                        files.push(path);
                    }
                }
            }
        }
        Ok(())
    }
}

/*
fn format_lines_to_string(lines: &Vec<Line>) -> String {
    lines
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<String>>()
        .join("\n")
}
*/
