
use std::path::{Path, PathBuf};

use crate::semantic::Line;

struct Context {
    name: String,
    path: PathBuf,
    lines: Vec<Line>,
    patches: BTreeMap<(usize, usize), Vec<Line>>, 
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
        })
    }
}

struct ContextManager {
    contexts: HashMap<String, Context>,
    modified: HashSet<String>,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
            modified: HashSet::new(),
        }
    }

    pub fn load_context(&mut self, project: &Project, name: &str) -> Result<&Context> {
        let entry = self.contexts.entry(name.to_string());
        if entry.or_insert_with(|| Context::load(project, name)?).is_none() {
            return Err(format!("Failed to load context: {}", name).into());
        }
        entry.get()
    }
}
        