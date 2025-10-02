
use anyhow::{Context as AnyhowContext, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::agent_call::AgentCall;

#[derive(Debug, PartialEq)]
pub enum LineData {
    Include { context_name: String },
    Answer,
    Summary { context_name: String },
    Text(String),
}

#[derive(Debug, PartialEq)]
pub struct Line {
    pub data: LineData,
    pub source_file: PathBuf,
    pub source_line_number: usize,
}

#[derive(Debug)]
pub enum ContextTreeItem {
    Node { name: String, children: Vec<ContextTreeItem> },
    Leaf { name: String },
}

pub struct Project {
    pub root_path: PathBuf,
}

impl Project {
    pub fn compose(&self, name: &str, agent: &dyn AgentCall) -> Result<Vec<Line>> {
        let path = self.resolve_context(name)?;
        let mut visited = HashSet::new();
        self.compose_recursive(&path, &mut visited, agent)
    }

    fn compose_recursive(&self, path: &Path, visited: &mut HashSet<PathBuf>, agent: &dyn AgentCall) -> Result<Vec<Line>> {
        if visited.contains(path) {
            return Ok(Vec::new()); // Circular include
        }
        visited.insert(path.to_path_buf());

        let mut composed_lines = Vec::new(); // Initialize composed_lines
        
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {:?}", path))?;
        
        for line in Self::parse(&content, path.to_path_buf()) {
            match line.data {
                LineData::Include { context_name } => {
                    let include_path = self.resolve_context(&context_name)?;
                    let included_lines = self.compose_recursive(&include_path, visited, agent)?;
                    composed_lines.extend(included_lines);
                }
                LineData::Summary { context_name } => {
                    let summarized_text = self._handle_summary_tag(&context_name, visited, agent)?;
                    composed_lines.push(Line {
                        data: LineData::Text(summarized_text),
                        source_file: line.source_file,
                        source_line_number: line.source_line_number,
                    });
                }
                _ => {
                    composed_lines.push(line);
                }
            }
        }
        
        Ok(composed_lines)
    }

    pub fn context_tree(&self, name: &str) -> Result<ContextTreeItem> {
        let path = self.resolve_context(name)?;
        self.context_tree_recursive(&path, &mut HashSet::new())
    }

    fn context_tree_recursive(&self, path: &Path, visited: &mut HashSet<PathBuf>) -> Result<ContextTreeItem> {
        if visited.contains(path) {
            return Ok(ContextTreeItem::Leaf { name: Self::to_name(&path.file_name().unwrap().to_string_lossy()) });
        }
        visited.insert(path.to_path_buf());
    
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {:?}", path))?;
    
        let mut children = Vec::new();
        let current_name = Self::to_name(&path.file_name().unwrap().to_string_lossy());
    
        for line in Self::parse(&content, path.to_path_buf()) {
            if let LineData::Include { context_name } = line.data {
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
                context_names.push(Self::to_name(&name));
            }
        }
        Ok(context_names)
    }

     pub fn new_context(&self, name: &str) -> Result<()> {
        let path = self.contexts_dir()?.join(Self::to_filename(name));
        
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
        let path = self.contexts_dir()?.join(Self::to_filename(name));
        if !path.is_file() {
            anyhow::bail!("Context '{}' does not exist", name);
        }
        Ok(path)
    }

    pub fn summaries_dir(&self) -> Result<PathBuf> {
        let path = self.root_path.join("summaries");
        std::fs::create_dir_all(&path)?;
        Ok(path)
    }

    pub fn execute_context(&self, name: &str, agent: &dyn AgentCall) -> Result<()> {
        loop {
            let composed_lines = self.compose(name, agent)?;
            
            let mut answer_line_index: Option<usize> = None;
            let mut prompt_content = String::new();

            for (i, line) in composed_lines.iter().enumerate() {
                if let LineData::Answer = line.data {
                    answer_line_index = Some(i);
                    break;
                }
                if let LineData::Text(text) = &line.data {
                    prompt_content.push_str(text);
                    prompt_content.push('\n');
                }
            }

            if let Some(index) = answer_line_index {
                let answer_line = &composed_lines[index];
                println!("Executing LLM for context: {}", name);
                println!("Found @answer tag in {:?} at line {}", answer_line.source_file, answer_line.source_line_number);

                let llm_response = self._execute_answer_llm_command(prompt_content, agent)?;
                println!("LLM Response:\n{}", llm_response);

                // 2. Replace @answer in the file
                let file_content = std::fs::read_to_string(&answer_line.source_file)?;
                let mut lines: Vec<String> = file_content.lines().map(String::from).collect();

                if answer_line.source_line_number < lines.len() {
                    lines[answer_line.source_line_number] = llm_response.trim().to_string();
                } else {
                    anyhow::bail!("Answer tag line number out of bounds for file {:?}", answer_line.source_file);
                }

                // 3. Rewrite the file
                std::fs::write(&answer_line.source_file, lines.join("\n"))?;
                println!("Rewrote file: {:?}", answer_line.source_file);

            } else {
                // No more @answer tags, break the loop
                println!("No more @answer tags found. Context execution complete.");
                break;
            }
        }
        Ok(())
    }

    fn _execute_answer_llm_command(&self, composed_content: String, agent: &dyn AgentCall) -> Result<String> {
        agent.call_llm(composed_content)
    }

    fn _execute_summary_llm_command(&self, prompt: String, agent: &dyn AgentCall) -> Result<String> {
        agent.call_llm(prompt)
    }

    fn _handle_summary_tag(&self, context_name: &str, visited: &mut HashSet<PathBuf>, agent: &dyn AgentCall) -> Result<String> {
        use sha2::{Sha256, Digest};
        use handlebars::Handlebars;
        use serde_json::json;

        let summary_target_path = self.resolve_context(context_name)?;
        let original_content_to_summarize = std::fs::read_to_string(&summary_target_path)
            .with_context(|| format!("Failed to read context for summary: {:?}", summary_target_path))?;
        let mut hasher = Sha256::new();
        hasher.update(original_content_to_summarize.as_bytes());
        let original_hash = format!("{:x}", hasher.finalize());

        let summary_filename = format!("{}.summary", Self::to_filename(context_name));
        let summary_file_path = self.summaries_dir()?.join(summary_filename);

        let mut summarized_text = String::new();
        let mut cache_hit = false;

        if summary_file_path.exists() {
            let cached_summary_json = std::fs::read_to_string(&summary_file_path)
                .with_context(|| format!("Failed to read cached summary file: {:?}", summary_file_path))?;
            let cached_data: serde_json::Value = serde_json::from_str(&cached_summary_json)
                .with_context(|| format!("Failed to parse cached summary JSON from {:?}", summary_file_path))?;
            if let (Some(cached_hash), Some(cached_content)) = (
                cached_data["original_hash"].as_str(),
                cached_data["summary_content"].as_str(),
            ) {
                if cached_hash == original_hash {
                    summarized_text = cached_content.to_string();
                    cache_hit = true;
                    println!("Using cached summary for {}", context_name);
                }
            }
        }

        if !cache_hit {
            println!("Generating new summary for {}", context_name);
            // Recursively compose the content to be summarized
            let mut summary_visited = visited.clone(); // Clone visited for sub-composition
            let lines_to_summarize = self.compose_recursive(&summary_target_path, &mut summary_visited, agent)?;
            let content_to_summarize: String = lines_to_summarize.into_iter()
                .filter_map(|l| if let LineData::Text(t) = l.data { Some(t) } else { None })
                .collect::<Vec<String>>()
                .join("\n");

            // Use handlebars for templating the prompt
            let mut handlebars = Handlebars::new();
            handlebars.register_template_string("summary_prompt", "Summarize the following content concisely:\n\n{{content}}")
                .context("Failed to register handlebars template")?;
            let prompt_data = json!({ "content": content_to_summarize });
            let llm_prompt = handlebars.render("summary_prompt", &prompt_data)
                .context("Failed to render handlebars template")?;

            summarized_text = self._execute_summary_llm_command(llm_prompt, agent)?;

            // Save to cache
            let cache_data = json!({ "original_hash": original_hash, "summary_content": summarized_text });
            std::fs::write(&summary_file_path, serde_json::to_string_pretty(&cache_data)?)
                .with_context(|| format!("Failed to write cached summary to {:?}", summary_file_path))?;
        }
        Ok(summarized_text)
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

    pub fn parse(content: &str, file_path: PathBuf) -> Vec<Line> {
        content
            .lines()
            .enumerate()
            .map(|(line_number, line)| {
                if let Some(context_name) = line.strip_prefix("@include ") {
                    Line {
                        data: LineData::Include {
                            context_name: context_name.trim().to_string(),
                        },
                        source_file: file_path.clone(),
                        source_line_number: line_number,
                    }
                } else if let Some(context_name) = line.strip_prefix("@summary ") {
                    Line {
                        data: LineData::Summary {
                            context_name: context_name.trim().to_string(),
                        },
                        source_file: file_path.clone(),
                        source_line_number: line_number,
                    }
                } else if line.trim() == "@answer" {
                    Line {
                        data: LineData::Answer,
                        source_file: file_path.clone(),
                        source_line_number: line_number,
                    }
                } else {
                    Line {
                        data: LineData::Text(line.to_string()),
                        source_file: file_path.clone(),
                        source_line_number: line_number,
                    }
                }
            })
            .collect()
    }

    pub fn to_name(name: &str) -> String {
        name.strip_suffix(".md").unwrap_or(name).to_string()
    }

    pub fn to_filename(name: &str) -> String {
        if name.ends_with(".md") {
            name.to_string()
        } else {
            format!("{}.md", name)
        }
    }
}
