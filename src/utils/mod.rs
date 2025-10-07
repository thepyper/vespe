
use crate::project::Project;
use crate::semantic::{self, Line};
use anyhow::Result;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use uuid::Uuid;

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
        Ok(Context{ 
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

        // TODO apply in reverse start order to avoid shifting indices

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

pub struct AnchorIndex {
    begin: HashMap<Uuid, usize>, // uid -> line index
    end: HashMap<Uuid, usize>,   // uid -> line index
}

impl AnchorIndex {
    pub fn new(lines: &[semantic::Line]) -> Self {
        let mut begin = HashMap::new();
        let mut end = HashMap::new();

        for (i, line) in lines.iter().enumerate() {
            if line.is_begin_anchor() {
                begin.insert(line.get_uid(), i);
            } else if line.is_end_anchor() {
                end.insert(line.get_uid(), i);
            }
        }

        Self { begin, end }
    }

    pub fn get_begin(&self, uid: &Uuid) -> Option<usize> {
        self.begin.get(uid).copied()
    }

    pub fn get_end(&self, uid: &Uuid) -> Option<usize> {
        self.end.get(uid).copied()
    }
}
        