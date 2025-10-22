
trait FileAccessor {
    /// Read whole file to a string
    fn read_file(path: &Path) -> Result<String>;
    /// Require exclusive access to a file
    fn lock_file(path: &Path) -> Result<()>;
    /// Release excludive access to a file
    fn unlock_file(path: &Path) -> Result<()>;
    /// Write whole file, optional comment to the operation
    fn write_file(path: &Path, comment: Option<&str>) -> Result<()>; 
}

