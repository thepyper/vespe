
use anyhow::Result;

trait FileAccessor {
    /// Read whole file to a string
    pub fn read_file(&self, path: &Path) -> Result<String>;
    /// Require exclusive access to a file
    pub fn lock_file(&self, path: &Path) -> Result<()>;
    /// Release excludive access to a file
    pub fn unlock_file(&self, path: &Path) -> Result<()>;
    /// Write whole file, optional comment to the operation
    pub fn write_file(&self, path: &Path, content: &str, comment: Option<&str>) -> Result<()>; 
}

