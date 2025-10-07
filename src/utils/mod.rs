
use std::path::{Path, PathBuf};

use crate::semantic::Line;

struct Context {
    name: String,
    path: PathBuf,
    lines: Vec<Line>,
    patches: BTreeMap<(usize, usize), Vec<Line>>, 
    modified: bool,
}

impl Context {
     pub fn load(project: &Project, name: &str) -> Result<Self> {
        let path = project.resolve_context(name);
        let lines = std::fs::read_to_string(path)?;
        let lines = crate::syntax::parse_document(&lines)?;
        let lines = crate::semantic::enrich_syntax_document(project, lines)?;
        Ok(Context{ 
            name: name.into(),
            path,
            lines,
            patches: BTreeMap::new(),
            modified: false,
        })
    }

    /// start = last line not to patch
    /// end = first line not to patch
    pub fn add_patch(&mut self, start: usize, end: usize, new_lines: Vec<Line>) {
        self.patches.insert((start, end), new_lines);
    }

    pub fn apply_patches(&mut self) {
        if self.patches.is_empty() {
            return;
        }

        // TODO apply in reverse start order to avoid shifting indices

        self.patches.clear();
        self.modified = true;
    }

    pub fn save(&mut self) -> Result<()> {
        self.apply_patches();
        if (!self.modified) {
            return Ok(());
        }
        let formatted = crate::semantic::format_document(&self.lines);
        std::fs::write(&self.path, formatted)?;
        Ok(())
    }
}

struct ContextManager {
    contexts: HashMap<String, Context>,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
        }
    }

    pub fn load_context(&mut self, project: &Project, name: &str) -> Result<&Context> {
        let entry = self.contexts.entry(name.to_string());
        if entry.or_insert_with(|| Context::load(project, name)?).is_none() {
            return Err(format!("Failed to load context: {}", name).into());
        }
        entry.get()
    }

    pub fn save_all(&mut self) -> Result<()> {
        for context in self.contexts.values_mut() {
            context.save()?;
        }
        Ok(())
    }
}
        