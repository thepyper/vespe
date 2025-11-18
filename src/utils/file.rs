use thiserror::Error as ThisError;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::{uuid, Uuid};
use super::Result;

use crate::editor::EditorCommunicator;
use super::git::git_commit_files;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Failed to read file '{path}': {source}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Editor interface error: {message}: {source}")]
    EditorInterface {
        message: String,
        #[source]
        source: anyhow::Error,
    },
    #[error("Failed to write file '{path}': {source}")]
    FileWrite {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Git error: {0}")]
    Git(#[from] super::git::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Mutex poisoned")]
    MutexPoisoned,
}

pub trait FileAccessor {
    /// Read whole file to a string
    fn read_file(&self, path: &Path) -> Result<String> {
        std::fs::read_to_string(path)
            .map_err(|e| Error::FileRead {
                path: path.to_path_buf(),
                source: e,
            })
    }
    /// Require exclusive access to a file
    fn lock_file(&self, path: &Path) -> Result<Uuid>;
    /// Release excludive access to a file
    fn unlock_file(&self, uuid: &Uuid) -> Result<()>;
    /// Write whole file, optional comment to the operation
    fn write_file(&self, path: &Path, content: &str, comment: Option<&str>) -> Result<()>;
}

/// Mutable part of ProjectFileAccessor struct
struct ProjectFileAccessorMutable {
    /// Set of modified files
    modified_files: HashSet<PathBuf>,
    /// List of commit messages for file modification
    modified_files_comments: Vec<String>,
}

pub struct ProjectFileAccessor {
    /// Project root path
    root_path: PathBuf,
    /// Editor interface to use
    editor_interface: Option<Arc<dyn EditorCommunicator>>,
    /// Mutable part of the struct to allow fine-grained lock strategy, only lock when needed
    mutable: Mutex<ProjectFileAccessorMutable>,
}

impl ProjectFileAccessor {
    pub fn new(root_path: &Path, editor_interface: Option<Arc<dyn EditorCommunicator>>) -> Self {
        ProjectFileAccessor {
            root_path: root_path.to_path_buf(),
            editor_interface,
            mutable: Mutex::new(ProjectFileAccessorMutable {
                modified_files: HashSet::new(),
                modified_files_comments: Vec::new(),
            }),
        }
    }
    pub fn modified_files(&self) -> Result<Vec<PathBuf>> {
        Ok(self.mutable
            .lock()
            .map_err(|_| Error::MutexPoisoned)?
            .modified_files
            .iter()
            .cloned()
            .collect::<Vec<PathBuf>>())
    }
    pub fn modified_files_comments(&self) -> Result<String> {
        Ok(self.mutable
            .lock()
            .map_err(|_| Error::MutexPoisoned)?
            .modified_files_comments
            .join("\n"))
    }
    pub fn commit(&self, title_message: Option<String>) -> Result<()> {
        let mut mutable = self.mutable.lock().map_err(|_| Error::MutexPoisoned)?;
        if !mutable.modified_files.is_empty() {
            let message_1 = match title_message {
                Some(x) => format!("{}\n", x),
                None => "".into(),
            };
            let message_2 = mutable.modified_files_comments.join("\n");
            let _ = git_commit_files(
                &self.root_path,
                &mutable
                    .modified_files
                    .iter()
                    .cloned()
                    .collect::<Vec<PathBuf>>(),
                &format!("{}{}", message_1, message_2),
            )?;
        }
        mutable.modified_files.clear();
        mutable.modified_files_comments.clear();
        Ok(())
    }
}

const DUMMY_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000000");

impl FileAccessor for ProjectFileAccessor {
    /// Read whole file to a string
    fn read_file(&self, path: &Path) -> Result<String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::FileRead {
                path: path.to_path_buf(),
                source: e,
            })?;
        Ok(content)
    }
    /// Require exclusive access to a file
    fn lock_file(&self, path: &Path) -> Result<Uuid> {
        match &self.editor_interface {
            None => Ok(DUMMY_ID),
            Some(x) => Ok(x
                .save_and_lock_file(path)
                .map_err(|e| Error::EditorInterface {
                    message: "Failed to save and lock file".to_string(),
                    source: e,
                })?),
        }
    }
    /// Release excludive access to a file
    fn unlock_file(&self, uuid: &Uuid) -> Result<()> {
        match &self.editor_interface {
            None => Ok(()),
            Some(x) => x
                .unlock_and_reload_file(*uuid)
                .map_err(|e| Error::EditorInterface {
                    message: "Failed to unlock and reload file".to_string(),
                    source: e,
                }),
        }
    }
    /// Write whole file, optional comment to the operation
    fn write_file(&self, path: &Path, content: &str, comment: Option<&str>) -> Result<()> {
        tracing::debug!("Writing file {:?}", path);
        std::fs::write(path, content)
            .map_err(|e| Error::FileWrite {
                path: path.to_path_buf(),
                source: e,
            })?;
        let mut mutable = self.mutable.lock().unwrap();
        mutable.modified_files.insert(path.into());
        if let Some(comment) = comment {
            mutable.modified_files_comments.push(comment.into());
        }
        Ok(())
    }
}

/// A RAII guard to ensure a file lock is released.
pub struct FileLock {
    file_access: Arc<dyn FileAccessor>,
    lock_id: Option<Uuid>,
}

impl FileLock {
    /// Creates a new `FileLock`, acquiring a lock on the given path.
    pub fn new(file_access: Arc<dyn FileAccessor>, path: &Path) -> Result<Self> {
        let lock_id = file_access.lock_file(path)?;
        Ok(Self {
            file_access,
            lock_id: Some(lock_id),
        })
    }
}

impl Drop for FileLock {
    /// Releases the file lock when the `FileLock` goes out of scope.
    fn drop(&mut self) {
        if let Some(lock_id) = self.lock_id.take() {
            if let Err(e) = self.file_access.unlock_file(&lock_id) {
                tracing::error!("Failed to unlock file with id {}: {}", lock_id, e);
            }
        }
    }
}
