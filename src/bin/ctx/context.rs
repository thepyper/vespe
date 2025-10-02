use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum Line {
    Include { context_name: String },
    Answer { file_path: PathBuf, line_number: usize },
    Text(String),
}

#[derive(Debug)]
pub enum ContextTreeItem {
    Node { name: String, children: Vec<ContextTreeItem> },
    Leaf { name: String },
}

pub struct Context;

impl Context {
    pub fn parse(content: &str, file_path: PathBuf) -> Vec<Line> {
        content
            .lines()
            .enumerate()
            .map(|(line_number, line)| {
                if let Some(context_name) = line.strip_prefix("@include ") {
                    Line::Include {
                        context_name: context_name.trim().to_string(),
                    }
                } else if line.trim() == "@answer" {
                    Line::Answer { file_path: file_path.clone(), line_number }
                } else {
                    Line::Text(line.to_string())
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
