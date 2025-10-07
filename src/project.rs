use crate::syntax::parser::parse_document;
use crate::syntax::types::{Line, TagKind};
use anyhow::Context as AnyhowContext;
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/*
#[derive(Debug)]
pub struct Context {
    pub name: String,
    pub content: Vec<Line>,
    pub includes: BTreeMap<usize, Context>, // line index to Context
    pub inlines: BTreeMap<usize, Snippet>,  // line index to Snippet
    pub summaries: BTreeMap<usize, Context>, // line index to Context
    pub answers: BTreeSet<usize>,           // line index
}
*/

#[derive(Debug)]
pub struct Snippet {
    pub name: String,
    pub content: Vec<Line>,
}

// ... (rest of the file)

const CTX_DIR_NAME: &str = ".ctx";
const CTX_ROOT_FILE_NAME: &str = ".ctx_root";
const METADATA_DIR_NAME: &str = ".meta";
const CONTEXTS_DIR_NAME: &str = "contexts";
const SNIPPETS_DIR_NAME: &str = "snippets";

#[derive(Debug)] // Add Debug trait for easy printing
pub struct ContextInfo {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug)] // Add Debug trait for easy printing
pub struct SnippetInfo {
    pub name: String,
    pub path: PathBuf,
}

/*
pub struct ContextManager {
    contexts: HashMap<String, Vec<Line>>,
    modified_contexts: HashSet<String>,
}

impl ContextManager {
    pub fn new() -> Self {
        ContextManager {
            contexts: HashMap::new(),
            modified_contexts: HashSet::new(),
        }
    }

    pub fn insert_context(&mut self, name: String, lines: Vec<Line>) {
        self.contexts.insert(name, lines);
    }

    pub fn get_context_mut(&mut self, name: &str) -> Option<&mut Vec<Line>> {
        self.contexts.get_mut(name)
    }

    pub fn remove_context(&mut self, name: &str) -> Option<Vec<Line>> {
        self.contexts.remove(name)
    }

    pub fn contains_context(&self, name: &str) -> bool {
        self.contexts.contains_key(name)
    }

    pub fn load_context(
        &mut self,
        project: &Project,
        context_name: &str,
    ) -> anyhow::Result<&mut Vec<Line>> {
        if !self.contains_context(context_name) {
            let lines = project.read_and_parse_context_file(context_name)?;
            self.insert_context(context_name.to_string(), lines);
        }
        self.get_context_mut(context_name).context(format!(
            "Context '{}' not found in ContextManager after loading",
            context_name
        ))
    }

    pub fn get_context(&mut self, context_name: &str) -> anyhow::Result<&mut Vec<Line>> {
        self.get_context_mut(context_name).context(format!(
            "Context '{}' not found in ContextManager",
            context_name
        ))
    }

    pub fn mark_as_modified(&mut self, context_name: &str) {
        self.modified_contexts.insert(context_name.to_string());
    }

    pub fn save_modified_contexts(&self, project: &Project) -> anyhow::Result<()> {
        for context_name in &self.modified_contexts {
            if let Some(lines) = self.contexts.get(context_name) {
                project.update_context_lines(context_name, lines.clone())?;
            }
        }
        Ok(())
    }
}
*/

pub struct Project {
    root_path: PathBuf,
}

#[allow(dead_code)]
impl Project {
    pub fn init(path: &Path) -> Result<Project> {
        let ctx_dir = path.join(CTX_DIR_NAME);
        if ctx_dir.is_dir() && ctx_dir.join(CTX_ROOT_FILE_NAME).is_file() {
            anyhow::bail!("ctx project already initialized in this directory.");
        }

        std::fs::create_dir_all(&ctx_dir).context("Failed to create .ctx directory")?;

        let ctx_root_file = ctx_dir.join(CTX_ROOT_FILE_NAME);
        std::fs::write(&ctx_root_file, "Feel The BuZZ!!")
            .context("Failed to write .ctx_root file")?;

        Ok(Project {
            root_path: path.canonicalize()?,
        })
    }

    pub fn find(path: &Path) -> Result<Project> {
        let mut current_path = path.to_path_buf();

        loop {
            let ctx_dir = current_path.join(CTX_DIR_NAME);
            if ctx_dir.is_dir() && ctx_dir.join(CTX_ROOT_FILE_NAME).is_file() {
                return Ok(Project {
                    root_path: current_path.canonicalize()?,
                });
            }

            if !current_path.pop() {
                break;
            }
        }

        anyhow::bail!("No .ctx project found in the current directory or any parent directory.")
    }

    pub fn project_home(&self) -> PathBuf {
        self.root_path.join(CTX_DIR_NAME)
    }

    pub fn metadata_home(&self) -> PathBuf {
        self.project_home().join(METADATA_DIR_NAME)
    }

    pub fn contexts_root(&self) -> PathBuf {
        self.project_home().join(CONTEXTS_DIR_NAME)
    }

    pub fn snippets_root(&self) -> PathBuf {
        self.project_home().join(SNIPPETS_DIR_NAME)
    }

    pub fn resolve_context(&self, name: &str) -> PathBuf {
        self.contexts_root().join(format!("{}.md", name))
    }

    pub fn resolve_snippet(&self, name: &str) -> PathBuf {
        self.snippets_root().join(format!("{}.md", name))
    }

    pub fn resolve_metadata(&self, anchor_kind: &str, uid: &Uuid) -> Result<PathBuf> {
        let anchor_metadata_dir =
            self.metadata_home()
                .join(format!("{}-{}", anchor_kind, uid.to_string()));
        std::fs::create_dir_all(&anchor_metadata_dir).context(format!(
            "Failed to create metadata directory for anchor {}-{}: {}",
            anchor_kind,
            uid,
            anchor_metadata_dir.display()
        ))?;
        Ok(anchor_metadata_dir)
    }

    pub fn create_context_file(&self, name: &str) -> Result<PathBuf> {
        let file_path = self.contexts_root().join(format!("{}.md", name));
        let parent_dir = file_path
            .parent()
            .context("Failed to get parent directory")?;
        std::fs::create_dir_all(parent_dir)
            .context("Failed to create parent directories for context file")?;
        std::fs::write(&file_path, "").context("Failed to create context file")?;
        Ok(file_path)
    }

    pub fn create_snippet_file(&self, name: &str) -> Result<PathBuf> {
        let file_path = self.snippets_root().join(format!("{}.md", name));
        let parent_dir = file_path
            .parent()
            .context("Failed to get parent directory")?;
        std::fs::create_dir_all(parent_dir)
            .context("Failed to create parent directories for snippet file")?;
        std::fs::write(&file_path, "").context("Failed to create snippet file")?;
        Ok(file_path)
    }

    pub fn list_contexts(&self) -> Result<Vec<ContextInfo>> {
        let mut contexts = Vec::new();
        let contexts_root = self.contexts_root();

        if !contexts_root.exists() {
            return Ok(contexts); // Return empty if directory doesn't exist
        }

        for entry in std::fs::read_dir(&contexts_root)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "md" {
                        if let Some(file_stem) = path.file_stem() {
                            if let Some(name) = file_stem.to_str() {
                                contexts.push(ContextInfo {
                                    name: name.to_string(),
                                    path: path.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }
        Ok(contexts)
    }

    pub fn list_snippets(&self) -> Result<Vec<SnippetInfo>> {
        let mut snippets = Vec::new();
        let snippets_root = self.snippets_root();

        if !snippets_root.exists() {
            return Ok(snippets); // Return empty if directory doesn\'t exist
        }

        for entry in std::fs::read_dir(&snippets_root)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "md" {
                        if let Some(file_stem) = path.file_stem() {
                            if let Some(name) = file_stem.to_str() {
                                snippets.push(SnippetInfo {
                                    name: name.to_string(),
                                    path: path.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }
        Ok(snippets)
    }

    pub fn load_snippet(&self, name: &str) -> Result<Snippet> {
        let file_path = self.resolve_snippet(name);
        let content = std::fs::read_to_string(&file_path).context(format!(
            "Failed to read snippet file: {}",
            file_path.display()
        ))?;
        let lines = parse_document(&content)
            .map_err(|e| anyhow::anyhow!(e))
            .context("Failed to parse document")?;

        Ok(Snippet {
            name: name.to_string(),
            content: lines,
        })
    }

    pub fn load_context(
        &self,
        name: &str,
        loading_contexts: &mut HashSet<String>,
    ) -> Result<Context> {
        if loading_contexts.contains(name) {
            anyhow::bail!("Circular dependency detected for context: {}", name);
        }
        loading_contexts.insert(name.to_string());

        let file_path = self.resolve_context(name);
        let content = std::fs::read_to_string(&file_path).context(format!(
            "Failed to read context file: {}",
            file_path.display()
        ))?;
        let lines = parse_document(&content)
            .map_err(|e| anyhow::anyhow!(e))
            .context("Failed to parse document")?;

        let mut includes = BTreeMap::new();
        let mut inlines = BTreeMap::new();
        let mut summaries = BTreeMap::new();
        let mut answers = BTreeSet::new();

            for (line_index, line) in lines.iter().enumerate() {
                if let Line::Tagged { tag, arguments, .. } = line {                if let Some(arg_name) = arguments.first() {
                    match tag {
                        TagKind::Include => {
                            let included_context = self.load_context(arg_name, loading_contexts)?;
                            includes.insert(line_index, included_context);
                        }
                        TagKind::Summary => {
                            let summarized_context =
                                self.load_context(arg_name, loading_contexts)?;
                            summaries.insert(line_index, summarized_context);
                        }
                        TagKind::Inline => {
                            let inlined_snippet = self.load_snippet(arg_name)?;
                            inlines.insert(line_index, inlined_snippet);
                        }
                        TagKind::Answer => {
                            answers.insert(line_index);
                        }
                    }
                }
            }
        }

        loading_contexts.remove(name);

        Ok(Context {
            name: name.to_string(),
            content: lines,
            includes,
            inlines,
            summaries,
            answers,
        })
    }

    pub fn get_context_tree(&self, context_name: &str) -> Result<Context> {
        let mut loading_contexts = HashSet::new();
        self.load_context(context_name, &mut loading_contexts)
    }

    pub fn read_and_parse_context_file(&self, name: &str) -> Result<Vec<Line>> {
        let file_path = self.resolve_context(name);
        let content = std::fs::read_to_string(&file_path).context(format!(
            "Failed to read context file: {}",
            file_path.display()
        ))?;
        parse_document(&content)
            .map_err(|e| anyhow::anyhow!(e))
            .context("Failed to parse document")
    }

    pub fn load_context_lines(&self, name: &str) -> Result<Vec<Line>> {
        self.read_and_parse_context_file(name)
    }

    pub fn load_snippet_lines(&self, name: &str) -> Result<Vec<Line>> {
        let snippet = self.load_snippet(name)?;
        Ok(snippet.content)
    }

    pub fn update_context_lines(&self, name: &str, lines: Vec<Line>) -> Result<()> {
        let file_path = self.resolve_context(name);
        let content = format_lines_to_string(&lines);
        std::fs::write(&file_path, content).context(format!(
            "Failed to write context file: {}",
            file_path.display()
        ))?;
        Ok(())
    }

    pub fn update_snippet_lines(&self, name: &str, lines: Vec<Line>) -> Result<()> {
        let file_path = self.resolve_snippet(name);
        let content = format_lines_to_string(&lines);
        std::fs::write(&file_path, content).context(format!(
            "Failed to write snippet file: {}",
            file_path.display()
        ))?;
        Ok(())
    }
}

fn format_lines_to_string(lines: &Vec<Line>) -> String {
    lines
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<String>>()
        .join("\n")
}
