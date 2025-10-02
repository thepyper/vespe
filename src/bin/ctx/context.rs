use std::path::PathBuf;

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

pub struct Context;

impl Context {
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
