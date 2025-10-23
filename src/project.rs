use crate::agent::ShellAgentCall;
use crate::git::Commit;

use anyhow::anyhow;
use anyhow::Context as AnyhowContext;
use anyhow::Result;

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use tracing::debug;
use uuid::Uuid;

use crate::config::{EditorInterface, ProjectConfig};
use crate::editor::{
    lockfile::FileBasedEditorCommunicator, EditorCommunicator,
};

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

pub struct Project {
    root_path: PathBuf,
    editor_communicator: Option<Box<dyn EditorCommunicator>>,
    project_config: ProjectConfig,
}

#[allow(dead_code)]
impl Project {
    pub fn init(path: &Path) -> Result<Project> {
        let ctx_dir = path.join(CTX_DIR_NAME);
        if ctx_dir.is_dir() && ctx_dir.join(CTX_ROOT_FILE_NAME).is_file() {
            anyhow::bail!("ctx project already initialized in this directory.");
        }

        std::fs::create_dir_all(&ctx_dir).context("Failed to create .ctx directory")?;

        let ctx_root_file = ctx_dir.join(CTX_ROOT_FILE_NAME);
        std::fs::write(&ctx_root_file, "Feel The BuZZ!!")
            .context("Failed to write .ctx_root file")?;

        let project = Project {
            root_path: path.canonicalize()?,
            editor_communicator: None,
            project_config: ProjectConfig::default(),
        };

        if project.project_config.git_integration_enabled {
            let mut commit = Commit::new();
            commit.files.insert(ctx_root_file);
            commit.commit("feat: Initialize .ctx project\nInitial commit of the .ctx project structure, including the .ctx directory and .ctx_root file.")?;
        }

        project.save_project_config()?;

        Ok(project)
    }

    pub fn find(path: &Path) -> Result<Project> {
        let mut current_path = path.to_path_buf();

        loop {
            let ctx_dir = current_path.join(CTX_DIR_NAME);
            if ctx_dir.is_dir() && ctx_dir.join(CTX_ROOT_FILE_NAME).is_file() {
                let root_path = current_path.canonicalize()?;
                let project_config_path = root_path
                    .join(CTX_DIR_NAME)
                    .join(METADATA_DIR_NAME)
                    .join("project_config.json");
                let project_config = Self::load_project_config(&project_config_path)?;

                let editor_path = ctx_dir.join(METADATA_DIR_NAME).join(".editor");
                let editor_communicator: Box<dyn EditorCommunicator> =
                    match project_config.editor_interface {
                        EditorInterface::VSCode => {
                            Some(Box::new(FileBasedEditorCommunicator::new(&editor_path)?))
                        }
                        _ => None,
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

        anyhow::bail!("No .ctx project found in the current directory or any parent directory.")
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

    /*
    pub fn snippets_root(&self) -> PathBuf {
        self.project_home().join(SNIPPETS_DIR_NAME)
    }
    */

    pub fn resolve_context(&self, name: &str) -> PathBuf {
        self.contexts_root().join(format!("{}.md", name))
    }

    /*
    pub fn resolve_snippet(&self, name: &str) -> PathBuf {
        self.snippets_root().join(format!("{}.md", name))
    }
    */

    /*
    pub fn resolve_metadata(&self, anchor_kind: &str, uid: &Uuid) -> Result<PathBuf> {
        let anchor_metadata_dir =
            self.metadata_home()
                .join(format!("{}-{}", anchor_kind, uid.to_string()));
        std::fs::create_dir_all(&anchor_metadata_dir).context(format!(
            "Failed to create metadata directory for anchor {}-{}: {}",
            anchor_kind,
            uid,
            anchor_metadata_dir.display()
        ))?;
        Ok(anchor_metadata_dir)
    }
    */

    pub fn project_config_path(&self) -> PathBuf {
        self.metadata_home().join("project_config.json")
    }

    pub fn create_context_file(
        &self,
        name: &str,
        initial_content: Option<String>,
    ) -> Result<PathBuf> {
        let file_path = self.contexts_root().join(format!("{}.md", name));
        if file_path.exists() {
            anyhow::bail!("Context file already exists: {}", file_path.display());
        }
        let parent_dir = file_path
            .parent()
            .context("Failed to get parent directory")?;
        std::fs::create_dir_all(parent_dir)
            .context("Failed to create parent directories for context file")?;
        let content = initial_content.unwrap_or_else(|| "".to_string());
        std::fs::write(&file_path, content).context("Failed to create context file")?;

        if self.project_config.git_integration_enabled {
            let mut commit = Commit::new();
            commit.files.insert(file_path.clone());
            commit.commit(&format!("feat: Create new context '{}'", name))?;
        }

        Ok(file_path)
    }

    /*
    pub fn create_snippet_file(
        &self,
        name: &str,
        initial_content: Option<String>,
    ) -> Result<PathBuf> {
        let file_path = self.snippets_root().join(format!("{}.md", name));
        if file_path.exists() {
            anyhow::bail!("Snippet file already exists: {}", file_path.display());
        }
        let parent_dir = file_path
            .parent()
            .context("Failed to get parent directory")?;
        std::fs::create_dir_all(parent_dir)
            .context("Failed to create parent directories for snippet file")?;
        let content = initial_content.unwrap_or_else(|| "".to_string());
        std::fs::write(&file_path, content).context("Failed to create snippet file")?;

        if self.project_config.git_integration_enabled {
            let mut commit = Commit::new();
            commit.files.insert(file_path.clone());
            commit.commit(&format!("feat: Create new snippet '{}'", name))?;
        }

        Ok(file_path)
    }
    */

    pub fn list_contexts(&self) -> Result<Vec<ContextInfo>> {
        let mut contexts = Vec::new();
        let contexts_root = self.contexts_root();

        if !contexts_root.exists() {
            return Ok(contexts); // Return empty if directory doesn't exist
        }

        let mut md_files = Vec::new();
        Self::collect_md_files_recursively(&contexts_root, &contexts_root, &mut md_files)?;

        for path in md_files {
            // Calculate the relative path from contexts_root to get the context name
            let relative_path = path.strip_prefix(&contexts_root)?;
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

    /*
    pub fn list_snippets(&self) -> Result<Vec<SnippetInfo>> {
        let mut snippets = Vec::new();
        let snippets_root = self.snippets_root();

        if !snippets_root.exists() {
            return Ok(snippets); // Return empty if directory doesn't exist
        }

        let mut md_files = Vec::new();
        Self::collect_md_files_recursively(&snippets_root, &snippets_root, &mut md_files)?;

        for path in md_files {
            // Calculate the relative path from snippets_root to get the snippet name
            let relative_path = path.strip_prefix(&snippets_root)?;
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

    pub fn load_snippet(&self, name: &str) -> Result<Snippet> {
        let file_path = self.resolve_snippet(name);
        let content = std::fs::read_to_string(&file_path).context(format!(
            "Failed to read snippet file: {}",
            file_path.display()
        ))?;
        let lines = crate::semantic::parse_document(self, &content)
            .map_err(|e| anyhow!("Failed to parse snippet document: {}", e))
            .context("Failed to parse document")?;

        Ok(Snippet {
            name: name.to_string(),
            content: lines,
        })
    }
    */

    pub fn save_project_config(&self) -> Result<()> {
        let config_path = self.project_config_path();
        std::fs::create_dir_all(
            config_path.parent().unwrap(), // TODO brutto!!
        )
        .context(format!("Failed to create metadata directory",))?;
        let serialized = serde_json::to_string_pretty(&self.project_config)?;
        std::fs::write(&config_path, serialized).context("Failed to write project config file")?;
        Ok(())
    }

    pub fn load_project_config(project_config_path: &PathBuf) -> Result<ProjectConfig> {
        match std::fs::read_to_string(project_config_path) {
            Ok(content) => Ok(serde_json::from_str(&content)?),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(ProjectConfig::default()),
            Err(e) => Err(anyhow::Error::new(e).context("Failed to read project config file")),
        }
    }

    /*
    fn save_state_to_metadata<T>(
        &self,
        anchor_kind: AnchorKind,
        uuid: &Uuid,
        state: &T,
        commit: &mut Commit,
    ) -> std::result::Result<(), SemanticError>
    where
        T: serde::Serialize,
    {
        let metadata_dir = self
            .resolve_metadata(anchor_kind.to_string().as_str(), uuid)
            .map_err(SemanticError::AnyhowError)?;
        std::fs::create_dir_all(&metadata_dir)?;
        let state_path = metadata_dir.join("state.json");
        let serialized = serde_json::to_string_pretty(state)?;
        std::fs::write(&state_path, serialized)?;
        commit.files.insert(state_path);
        Ok(())
    }

    fn load_state_from_metadata<T>(
        &self,
        anchor_kind: &AnchorKind,
        uid: &Uuid,
    ) -> std::result::Result<T, SemanticError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let metadata_dir = self
            .resolve_metadata(anchor_kind.to_string().as_str(), uid)
            .map_err(SemanticError::AnyhowError)?;
        let state_path = metadata_dir.join("state.json");

        match std::fs::read_to_string(&state_path) {
            Ok(content) => Ok(serde_json::from_str(&content)?),
            Err(e) if e.kind() == ErrorKind::NotFound => Err(SemanticError::Generic(format!(
                "State file not found for anchor {}-{}",
                anchor_kind, uid
            ))),
            Err(e) => Err(SemanticError::IoError(e)),
        }
    }

    pub fn save_inline_state(
        &self,
        uid: &Uuid,
        state: &InlineState,
        commit: &mut Commit,
    ) -> Result<()> {
        self.save_state_to_metadata(AnchorKind::Inline, uid, state, commit)
            .map_err(|e| anyhow::Error::new(e))
    }

    pub fn load_inline_state(&self, uid: &Uuid) -> Result<InlineState> {
        self.load_state_from_metadata(&AnchorKind::Inline, uid)
            .map_err(|e| anyhow::Error::new(e))
    }

    pub fn save_summary_state(
        &self,
        uid: &Uuid,
        state: &SummaryState,
        commit: &mut Commit,
    ) -> Result<()> {
        self.save_state_to_metadata(AnchorKind::Summary, uid, state, commit)
            .map_err(|e| anyhow::Error::new(e))
    }

    pub fn load_summary_state(&self, uid: &Uuid) -> Result<SummaryState> {
        self.load_state_from_metadata(&AnchorKind::Summary, uid)
            .map_err(|e| anyhow::Error::new(e))
    }

    pub fn save_answer_state(
        &self,
        uid: &Uuid,
        state: &AnswerState,
        commit: &mut Commit,
    ) -> Result<()> {
        self.save_state_to_metadata(AnchorKind::Answer, uid, state, commit)
            .map_err(|e| anyhow::Error::new(e))
    }

    pub fn load_answer_state(&self, uid: &Uuid) -> Result<AnswerState> {
        self.load_state_from_metadata(&AnchorKind::Answer, uid)
            .map_err(|e| anyhow::Error::new(e))
    }

    pub fn save_derive_state(
        &self,
        uid: &Uuid,
        state: &DeriveState,
        commit: &mut Commit,
    ) -> Result<()> {
        self.save_state_to_metadata(AnchorKind::Derive, uid, state, commit)
            .map_err(|e| anyhow::Error::new(e))
    }

    pub fn load_derive_state(&self, uid: &Uuid) -> Result<DeriveState> {
        self.load_state_from_metadata(&AnchorKind::Derive, uid)
            .map_err(|e| anyhow::Error::new(e))
    }

    pub fn request_file_modification(&self, file_path: &PathBuf) -> Result<Uuid> {
        self.editor_communicator
            .request_file_modification(file_path)
    }

    pub fn notify_file_modified(&self, file_path: &PathBuf, uid: Uuid) -> Result<()> {
        self.editor_communicator
            .notify_file_modified(file_path, uid)
    }

    pub fn execute_context(&self, context_name: &str, agent: &ShellAgentCall) -> Result<()> {
        debug!("Project::execute_context called for: {}", context_name);
        let mut commit = Commit::new();
        execute::execute(self, context_name, agent, &mut commit)?;
        if self.project_config.git_integration_enabled {
            debug!("Attempting to commit for context: {}", context_name);
            commit.commit(&format!("feat: Executed context '{}'", context_name))?;
        }
        Ok(())
    }
*/

    fn collect_md_files_recursively(
        root: &Path,
        current_dir: &Path,
        files: &mut Vec<PathBuf>,
    ) -> Result<()> {
        for entry in std::fs::read_dir(current_dir)? {
            let entry = entry?;
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

impl FileAccessor for Project {
    /// Read whole file to a string
    fn read_file(&self, path: &Path) -> Result<String>
    {
        std::fs::read_to_string(path)
    }
    /// Require exclusive access to a file
    fn lock_file(&self, path: &Path) -> Result<()>
    {
        match self.editor_interface {
            None => Ok(()),
            Some(x) => x.request_file_modification(path),
        }
    }
    /// Release excludive access to a file
    fn unlock_file(&self, path: &Path) -> Result<()>
    {
        match self.editor_interface {
            None => Ok(()),
            Some(x) => x.notify_file_modified(path),
        }
    }
    /// Write whole file, optional comment to the operation
    fn write_file(&self, path: &Path, content: &str, comment: Option<&str>) -> Result<()>
    {
        std::fs::write(path, content)?;
        match self.git_integration_enabled {
            // TODO logica per commit, faccio 1 solo?
        }
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
