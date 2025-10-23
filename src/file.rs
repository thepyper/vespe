
use anyhow::Result;
use std::path::Path;

pub trait FileAccessor {
    /// Read whole file to a string
    fn read_file(&mut self, path: &Path) -> Result<String>;
    /// Require exclusive access to a file
    fn lock_file(&mut self, path: &Path) -> Result<()>;
    /// Release excludive access to a file
    fn unlock_file(&mut self, path: &Path) -> Result<()>;
    /// Write whole file, optional comment to the operation
    fn write_file(&mut self, path: &Path, content: &str, comment: Option<&str>) -> Result<()>; 
}

pub ProjectFileAccessor {
    editor_interface: Option<Box<dyn EditorCommunicator>>,
    modified_files: HashSet<PathBuf>,
    modified_files_comments: Vec<String>,
}

impl ProjectFileAccessor {
    pub fn new(editor_interface: Option<Box<dyn EditorCommunicator>>) -> Self {
        ProjectFileAccessor {
            editor_interface,
            modified_files: HashSet::new(),
            modified_files_comments: Vec::new(),
        }
    }
    pub fn modified_files(&self) -> Vec<PathBuf> {
        self.modified_files.iter().cloned().collect::<Vec<PathBuf>>()
    }
    pub fn modified_files_comments(&self) -> String {
        self.modified_files_comments.join("\n")
    }
}

impl FileAccessor for ProjectFileAccessor {
    /// Read whole file to a string
    fn read_file(&mut self, path: &Path) -> Result<String>
    {
        std::fs::read_to_string(path)
    }
    /// Require exclusive access to a file
    fn lock_file(&mut self, path: &Path) -> Result<()>
    {
        match self.editor_interface {
            None => Ok(()),
            Some(x) => x.request_file_modification(path),
        }
    }
    /// Release excludive access to a file
    fn unlock_file(&mut self, path: &Path) -> Result<()>
    {
        match self.editor_interface {
            None => Ok(()),
            Some(x) => x.notify_file_modified(path),
        }
    }
    /// Write whole file, optional comment to the operation
    fn write_file(&mut self, path: &Path, content: &str, comment: Option<&str>) -> Result<()>
    {
        std::fs::write(path, content)?;
        self.modified_files.insert(path);
        if let Some(comment) = comment {
            self.modified_files_comments.push(comment);
        }         
        Ok(())
    }
}

