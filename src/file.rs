use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::{uuid, Uuid};

use super::editor::EditorCommunicator;
use super::git::git_commit_files;

pub trait FileAccessor {
    /// Read whole file to a string
    fn read_file(&self, path: &Path) -> Result<String>;
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
    /// Editor interface to use
    editor_interface: Option<Arc<dyn EditorCommunicator>>,
    /// Mutable part of the struct to allow fine-grained lock strategy, only lock when needed
    mutable: Mutex<ProjectFileAccessorMutable>,
}

impl ProjectFileAccessor {
    pub fn new(editor_interface: Option<Arc<dyn EditorCommunicator>>) -> Self {
        ProjectFileAccessor {
            editor_interface,
            mutable: Mutex::new(ProjectFileAccessorMutable {
                modified_files: HashSet::new(),
                modified_files_comments: Vec::new(),
            }),
        }
    }
    pub fn modified_files(&self) -> Vec<PathBuf> {
        self.mutable
            .lock()
            .unwrap()
            .modified_files
            .iter()
            .cloned()
            .collect::<Vec<PathBuf>>()
    }
    pub fn modified_files_comments(&self) -> String {
        self.mutable
            .lock()
            .unwrap()
            .modified_files_comments
            .join("\n")
    }
    pub fn commit(&self, title_message: Option<String>) -> Result<()> {
        let mut mutable = self.mutable.lock().unwrap();
        if !mutable.modified_files.is_empty() {
            let message_1 = match title_message {
                Some(x) => format!("{}\n", x),
                None => "".into(),
            };
            let message_2 = mutable.modified_files_comments.join("\n");
            let _ = git_commit_files(
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
        Ok(std::fs::read_to_string(path)?)
    }
    /// Require exclusive access to a file
    fn lock_file(&self, path: &Path) -> Result<Uuid> {
        match &self.editor_interface {
            None => Ok(DUMMY_ID),
            Some(x) => x.save_and_lock_file(path),
        }
    }
    /// Release excludive access to a file
    fn unlock_file(&self, uuid: &Uuid) -> Result<()> {
        match &self.editor_interface {
            None => Ok(()),
            Some(x) => x.unlock_and_reload_file(*uuid),
        }
    }
    /// Write whole file, optional comment to the operation
    fn write_file(&self, path: &Path, content: &str, comment: Option<&str>) -> Result<()> {
        tracing::debug!("Writing file {:?}", path);
        std::fs::write(path, content)?;
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
