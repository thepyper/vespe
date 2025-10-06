
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use crate::ast::types::{AnchorKind};
use uuid::Uuid;

const CTX_DIR_NAME: &str = ".ctx";
const CTX_ROOT_FILE_NAME: &str = ".ctx_root";
const METADATA_DIR_NAME: &str = ".meta";
const CONTEXTS_DIR_NAME: &str = "contexts";
const SNIPPETS_DIR_NAME: &str = "snippets";

pub struct Project {
    root_path: PathBuf,
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

        Ok(Project {
            root_path: ctx_dir.canonicalize()?,
        })
    }

    pub fn find(path: &Path) -> Result<Project> {
        let mut current_path = path.to_path_buf();

        loop {
            let ctx_dir = current_path.join(CTX_DIR_NAME);
            if ctx_dir.is_dir() && ctx_dir.join(CTX_ROOT_FILE_NAME).is_file() {
                return Ok(Project {
                    root_path: ctx_dir.canonicalize()?,
                });
            }

            if !current_path.pop() {
                break;
            }
        }

        anyhow::bail!("No .ctx project found in the current directory or any parent directory.")
    }

    pub fn project_home(&self) -> PathBuf {
        self.root_path.clone()
    }

    pub fn metadata_home(&self) -> PathBuf {
        self.root_path.join(METADATA_DIR_NAME)
    }

    pub fn contexts_root(&self) -> PathBuf {
        self.root_path.join(CONTEXTS_DIR_NAME)
    }

    pub fn snippets_root(&self) -> PathBuf {
        self.root_path.join(SNIPPETS_DIR_NAME)
    }

    pub fn resolve_context(&self, name: &str) -> PathBuf {
        self.contexts_root().join(format!("{}.md", name))
    }

    pub fn resolve_snippet(&self, name: &str) -> PathBuf {
        self.snippets_root().join(format!("{}.md", name))
    }

    pub fn resolve_metadata(&self, kind: &AnchorKind, uid: &Uuid) -> PathBuf {
        self.metadata_home().join(format!("{}-{}.md", kind, uid))
    }
}