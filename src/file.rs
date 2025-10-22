
use anyhow::Result;
use std::path::Path;

pub trait FileAccessor {
    /// Read whole file to a string
    fn read_file(&self, path: &Path) -> Result<String>;
    /// Require exclusive access to a file
    fn lock_file(&self, path: &Path) -> Result<()>;
    /// Release excludive access to a file
    fn unlock_file(&self, path: &Path) -> Result<()>;
    /// Write whole file, optional comment to the operation
    fn write_file(&self, path: &Path, content: &str, comment: Option<&str>) -> Result<()>; 
}

