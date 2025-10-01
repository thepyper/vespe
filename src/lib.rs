/*
pub mod agent;
pub mod task;
pub mod error;
pub mod utils;
pub mod tool;
pub mod memory;
pub mod registry;

pub mod project;

pub use tool::*;
pub use project::*;
pub use task::*;
pub use agent::*;
pub use error::*;
pub use utils::*;
pub use memory::*;
*/

use anyhow::Result;

pub enum TaskStatePlanSectionItem{
    LocalTask(String),
    ReferencedTask(String),
}

pub enum TaskStateSection {
    Intent{ title: String, text: String },
    Plan{ title: String, items: Vec<TaskStatePlanSectionItem> },
    Text{ title: String, items: String }, 
}

pub struct TaskState {
    /// Original markdown file 
    md: String,
    /// Original markdown file parsed ast
    mdast: markdown::mdast::Node,
    /// State structure parsed
    sections: Vec<TaskStateSection>,
}

pub struct TaskMetadata {
    name: String,
}

pub struct Task {
    uid: String,
    meta: TaskMetadata,
    state: TaskState,
}

impl TaskState {
    
    fn from_md(md : &str) -> Result<TaskState> {
        unimplemented!();
    }        
}