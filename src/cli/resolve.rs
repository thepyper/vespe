'''use std::path::Path;
use project::Task;
use project::api::{load_task, list_all_tasks};
use anyhow::{anyhow, Result};

/// Resolves a task identifier (which can be a UID or a name) to a Task.
///
/// # Arguments
///
/// * `project_root` - The root path of the Vespe project.
/// * `identifier` - The string identifier for the task (e.g., "tsk-...") or its name.
///
/// # Returns
///
/// A `Result` containing the resolved `Task` or an error if the task cannot be found
/// or if the name is ambiguous.
pub fn resolve_task(project_root: &Path, identifier: &str) -> Result<Task> {
    // 1. Try to load directly as a UID.
    if identifier.starts_with("tsk-") {
        if let Ok(task) = load_task(project_root, identifier) {
            return Ok(task);
        }
    }

    // 2. If that fails or it's not a UID, search by name.
    let all_tasks = list_all_tasks(project_root)?;
    let matching_tasks: Vec<Task> = all_tasks
        .into_iter()
        .filter(|t| t.config.name == identifier)
        .collect();

    // 3. Check the search results.
    match matching_tasks.len() {
        0 => Err(anyhow!("Task '{}' not found.", identifier)),
        1 => Ok(matching_tasks.into_iter().next().unwrap()),
        _ => Err(anyhow!(
            "Multiple tasks found with the name '{}'. Please use a unique UID.",
            identifier
        )),
    }
}
'''