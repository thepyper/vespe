
use anyhow::{Context as AnyhowContext, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::io::Write;

use super::context::{Context, ContextTreeItem, Line};

#[derive(Debug)]
pub struct AnswerTagLocation {
    pub file_path: PathBuf,
    pub line_number: usize,
}
pub struct Project {
    pub root_path: PathBuf,
}

impl Project {
    pub fn compose(&self, name: &str) -> Result<(String, Option<AnswerTagLocation>)> {
        let path = self.resolve_context(name)?;
        let mut visited = HashSet::new();
        self.compose_recursive(&path, &mut visited)
    }

    fn compose_recursive(&self, path: &Path, visited: &mut HashSet<PathBuf>) -> Result<(String, Option<AnswerTagLocation>)> {
        if visited.contains(path) {
            return Ok((String::new(), None)); // Circular include
        }
        visited.insert(path.to_path_buf());
        
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {:?}", path))?;
        
        let mut output = String::new();
        
        for line in Context::parse(&content, path.to_path_buf()) {
            match line {
                Line::Include { context_name } => {
                    let include_path = self.resolve_context(&context_name)?;
                    let (composed_content, answer_tag) = self.compose_recursive(&include_path, visited)?;
                    output.push_str(&composed_content);
                    output.push('\n');
                    if answer_tag.is_some() {
                        return Ok((output, answer_tag));
                    }
                }
                Line::Text(text) => {
                    output.push_str(&text);
                    output.push('\n');
                }
                Line::Answer { file_path, line_number } => {
                    return Ok((output, Some(AnswerTagLocation { file_path, line_number })));
                }
            }
        }
        
        Ok((output, None))
    }

    pub fn context_tree(&self, name: &str) -> Result<ContextTreeItem> {
        let path = self.resolve_context(name)?;
        self.context_tree_recursive(&path, &mut HashSet::new())
    }

    fn context_tree_recursive(&self, path: &Path, visited: &mut HashSet<PathBuf>) -> Result<ContextTreeItem> {
        if visited.contains(path) {
            return Ok(ContextTreeItem::Leaf { name: Context::to_name(&path.file_name().unwrap().to_string_lossy()) });
        }
        visited.insert(path.to_path_buf());
    
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {:?}", path))?;
    
        let mut children = Vec::new();
        let current_name = Context::to_name(&path.file_name().unwrap().to_string_lossy());
    
        for line in Context::parse(&content, path.to_path_buf()) {
            if let Line::Include { context_name } = line {
                let include_path = self.resolve_context(&context_name)?;
                children.push(self.context_tree_recursive(&include_path, visited)?);
            }
        }
    
        if children.is_empty() {
            Ok(ContextTreeItem::Leaf { name: current_name })
        } else {
            Ok(ContextTreeItem::Node { name: current_name, children })
        }
    }

    pub fn contexts_dir(&self) -> Result<PathBuf> {
        let path = self.root_path.join("contexts");
        std::fs::create_dir_all(&path)?;
        Ok(path)
    }

    pub fn list_contexts(&self) -> Result<Vec<String>> {
        let contexts_path = self.contexts_dir()?;
        let mut context_names = Vec::new();
    
        for entry in std::fs::read_dir(contexts_path)? {
            let entry = entry?;
            if entry.path().extension() == Some("md".as_ref()) {
                let name = entry.file_name().to_string_lossy().to_string();
                context_names.push(Context::to_name(&name));
            }
        }
        Ok(context_names)
    }

     pub fn new_context(&self, name: &str) -> Result<()> {
        let path = self.contexts_dir()?.join(Context::to_filename(name));
        
        if path.exists() {
            anyhow::bail!("Context '{}' already exists", name);
        }
        
        std::fs::write(&path, format!("# {}\n\n", name))?;
        println!("Created {}", path.display());
        Ok(())
    }

    pub fn edit_context(&self, name: &str) -> Result<()> {
        let path = self.resolve_context(name)?;
        
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
        
        std::process::Command::new(editor)
            .arg(&path)
            .status()?;
        
        Ok(())
    }

    pub fn resolve_context(&self, name: &str) -> Result<PathBuf> {
        let path = self.contexts_dir()?.join(Context::to_filename(name));
        if !path.is_file() {
            anyhow::bail!("Context '{}' does not exist", name);
        }
        Ok(path)
    }

    pub fn execute_context(&self, name: &str) -> Result<()> {
        loop {
            let (composed_content, answer_tag_option) = self.compose(name)?;
            if let Some(answer_tag) = answer_tag_option {
                println!("Executing LLM for context: {}", name);
                println!("Found @answer tag in {:?} at line {}", answer_tag.file_path, answer_tag.line_number);

                let llm_response = self._execute_llm_command(composed_content)?;
                println!("LLM Response:\n{}", llm_response);

                // 2. Replace @answer in the file
                let file_content = std::fs::read_to_string(&answer_tag.file_path)?;
                let mut lines: Vec<String> = file_content.lines().map(String::from).collect();

                if answer_tag.line_number < lines.len() {
                    lines[answer_tag.line_number] = llm_response.trim().to_string();
                } else {
                    anyhow::bail!("Answer tag line number out of bounds for file {:?}", answer_tag.file_path);
                }

                // 3. Rewrite the file
                std::fs::write(&answer_tag.file_path, lines.join("\n"))?;
                println!("Rewrote file: {:?}", answer_tag.file_path);

            } else {
                // No more @answer tags, break the loop
                println!("No more @answer tags found. Context execution complete.");
                break;
            }
        }
        Ok(())
    }

    fn _execute_llm_command(&self, composed_content: String) -> Result<String> {
        let mut command = {
            #[cfg(windows)]
            {
                let mut cmd = Command::new("cmd");
                cmd.arg("/C").arg("gemini");
                cmd
            }
            #[cfg(not(windows))]
            {
                Command::new("gemini")
            }
        };

        command
            .arg("-p")
            .arg("-y")
            .arg("-m")
            .arg("gemini-2.5-flash")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());

        let mut child = command
            .spawn()
            .context("Failed to spawn gemini command. Is 'gemini' in your PATH?")?;

        child.stdin.as_mut().unwrap().write_all(composed_content.as_bytes())?;
        let output = child.wait_with_output()?;

        if !output.status.success() {
            anyhow::bail!("Gemini command failed: {:?}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn init(path: &Path) -> Result<Project> {
        let ctx_dir = path.join(".ctx");
        if ctx_dir.is_dir() && ctx_dir.join(".ctx_root").is_file() {
            anyhow::bail!("ctx project already initialized in this directory.");
        }

        std::fs::create_dir_all(&ctx_dir).context("Failed to create .ctx directory")?;

        let ctx_root_file = ctx_dir.join(".ctx_root");
        std::fs::write(&ctx_root_file, "Feel The BuZZ!!")
            .context("Failed to write .ctx_root file")?;

        Ok(Project {
            root_path: ctx_dir.canonicalize()?,
        })
    }

    pub fn find(path: &Path) -> Result<Project> {
        let mut current_path = path.to_path_buf();

        loop {
            let ctx_dir = current_path.join(".ctx");
            if ctx_dir.is_dir() && ctx_dir.join(".ctx_root").is_file() {
                return Ok(Project {
                    root_path: ctx_dir.canonicalize()?,
                });
            }

            if !current_path.pop() {
                break;
            }
        }

        anyhow::bail!("No .ctx project found in the current directory or any parent directory.")
    }
}
