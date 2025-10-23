use anyhow::Result;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use std::sync::Mutex;
use uuid::{uuid, Uuid};

use super::editor::EditorCommunicator;

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
struct ProjectFileAccessorMutable 
{
    /// Set of modified files
    modified_files: HashSet<PathBuf>,
    /// List of commit messages for file modification
    modified_files_comments: Vec<String>,
}

pub struct ProjectFileAccessor {
    /// Editor interface to use
    editor_interface: Option<Box<dyn EditorCommunicator>>,
    /// Mutable part of the struct to allow fine-grained lock strategy, only lock when needed
    mutable: Mutex<ProjectFileAccessorMutable>,
}

impl ProjectFileAccessor {
    pub fn new(editor_interface: Option<Box<dyn EditorCommunicator>>) -> Self {
        ProjectFileAccessor {
            editor_interface,
            mutable: Mutex::new(
                ProjectFileAccessorMutable {
                    modified_files: HashSet::new(),
                    modified_files_comments: Vec::new(),
                }
            )
        }
    }
    pub fn modified_files(&self) -> Vec<PathBuf> {
        self.mutable.lock().unwrap().modified_files.iter().cloned().collect::<Vec<PathBuf>>()
    }
    pub fn modified_files_comments(&self) -> String {
        self.mutable.lock().unwrap().modified_files_comments.join("\n")
    }
}

const DUMMY_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000000");

impl FileAccessor for ProjectFileAccessor {
    /// Read whole file to a string
    fn read_file(&self, path: &Path) -> Result<String>
    {
        Ok(std::fs::read_to_string(path)?)
    }
    /// Require exclusive access to a file
    fn lock_file(&self, path: &Path) -> Result<Uuid>
    {
        match &self.editor_interface {
            None => Ok(DUMMY_ID),
            Some(x) => x.request_file_modification(path),
        }
    }
    /// Release excludive access to a file
    fn unlock_file(&self, uuid: &Uuid) -> Result<()>
    {
        match &self.editor_interface {
            None => Ok(()),
            Some(x) => x.notify_file_modified(*uuid),
        }
    }
    /// Write whole file, optional comment to the operation
    fn write_file(&self, path: &Path, content: &str, comment: Option<&str>) -> Result<()>
    {
        std::fs::write(path, content)?;
        let mut mutable = self.mutable.lock().unwrap();
        mutable.modified_files.insert(path.into());
        if let Some(comment) = comment {
            mutable.modified_files_comments.push(comment.into());
        }         
        Ok(())
    }
}

