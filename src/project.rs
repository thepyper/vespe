use crate::constants::{CONTEXTS_DIR_NAME, CTX_DIR_NAME, CTX_ROOT_FILE_NAME, METADATA_DIR_NAME};
use crate::file::ProjectFileAccessor;
use crate::git::Commit;
use crate::path::{PathResolver, ProjectPathResolver};
use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Context as AnyhowContext;
use anyhow::Result;

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use tracing::debug;

use crate::config::{EditorInterface, ProjectConfig};
use crate::editor::{lockfile::FileBasedEditorCommunicator, EditorCommunicator};

#[derive(Debug)] // Add Debug trait for easy printing
pub struct ContextInfo {
    pub name: String,
    pub path: PathBuf,
}

pub struct Project {
    //root_path: PathBuf,
    editor_interface: Option<Arc<dyn EditorCommunicator>>,
    file_access: Arc<ProjectFileAccessor>,
    path_res: Arc<ProjectPathResolver>,
    project_config: ProjectConfig,
}

#[allow(dead_code)]
impl Project {
    pub fn init(path: &Path) -> Result<()> {
        let ctx_dir = path.join(CTX_DIR_NAME);
        if ctx_dir.is_dir() && ctx_dir.join(CTX_ROOT_FILE_NAME).is_file() {
            anyhow::bail!("ctx project already initialized in this directory.");
        }

        std::fs::create_dir_all(&ctx_dir).context("Failed to create .ctx directory")?;

        let ctx_root_file = ctx_dir.join(CTX_ROOT_FILE_NAME);
        std::fs::write(&ctx_root_file, "Feel The BuZZ!!")
            .context("Failed to write .ctx_root file")?;

        /* TODO ???
        if project.project_config.git_integration_enabled {
            let mut commit = Commit::new();
            commit.files.insert(ctx_root_file);
            commit.commit("feat: Initialize .ctx project\nInitial commit of the .ctx project structure, including the .ctx directory and .ctx_root file.")?;
        }
        */

        // TODO project.save_project_config()?;
        Ok(())
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
                let editor_interface: Option<Arc<dyn EditorCommunicator>> =
                    match project_config.editor_interface {
                        EditorInterface::VSCode => {
                            Some(Arc::new(FileBasedEditorCommunicator::new(&editor_path)?)
                                as Arc<dyn EditorCommunicator>)
                        }
                        _ => None,
                    };

                let file_access = Arc::new(ProjectFileAccessor::new(editor_interface.clone()));
                let path_res = Arc::new(ProjectPathResolver::new(root_path.clone()));

                return Ok(Project {
                    editor_interface,
                    file_access,
                    path_res,
                    project_config,
                });
            }

            if !current_path.pop() {
                break;
            }
        }

        anyhow::bail!("No .ctx project found in the current directory or any parent directory.")
    }

    pub fn execute_context(&self, context_name: &str) -> Result<()> {
        crate::execute2::execute_context(
            self.file_access.clone(),
            self.path_res.clone(),
            context_name,
        )?;
        self.file_access.commit()?;
        Ok(())
    }

    pub fn project_home(&self) -> PathBuf {
        self.path_res.project_home()
    }

    pub fn contexts_root(&self) -> PathBuf {
        self.path_res.contexts_root()
    }

    pub fn project_config_path(&self) -> PathBuf {
        self.path_res.metadata_home().join("project_config.json")
    }

    pub fn create_context_file(
        &self,
        name: &str,
        initial_content: Option<String>,
    ) -> Result<PathBuf> {
        let file_path = self.path_res.resolve_context(name)?;
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

    pub fn list_contexts(&self) -> Result<Vec<ContextInfo>> {
        /* TODO REDO
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
        */
        Ok(vec![])
    }

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
}
