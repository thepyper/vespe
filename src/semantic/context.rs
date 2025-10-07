use crate::project::Project;
use crate::semantic::Line;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub struct Context {
    pub name: String,
    pub path: PathBuf,
    pub lines: Vec<Line>,
    pub modified: bool,
}

/// start = last line not to patch
/// end = first line not to patch
pub type Patches = BTreeMap<(usize, usize), Vec<Line>>; // (start, end) -> replacement lines

impl Context {
    pub fn load(project: &Project, name: &str) -> Result<Self> {
        let path = project.resolve_context(name);
        let content = std::fs::read_to_string(&path)?;
        let lines = crate::semantic::parse_document(project, &content)?;
        Ok(Context {
            name: name.into(),
            path,
            lines,
            modified: false,
        })
    }

    /// true = some patch has been applied
    pub fn apply_patches(&mut self, patches: Patches) -> bool {
        if patches.is_empty() {
            return false;
        }

        for ((start_line_index, end_line_index), replacement_lines) in patches.into_iter().rev() {
            // Remove the old lines
            self.lines.drain(start_line_index..end_line_index);
            // Insert the new lines
            for (i, line) in replacement_lines.into_iter().enumerate() {
                self.lines.insert(start_line_index + i, line);
            }
        }

        self.modified = true;
        true
    }

    pub fn save(&mut self) -> Result<()> {
        if !self.modified {
            return Ok(());
        }
        let formatted = crate::semantic::format_document(&self.lines);
        std::fs::write(&self.path, formatted)?;
        Ok(())
    }
}
