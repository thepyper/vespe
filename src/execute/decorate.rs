use std::collections::HashSet;
use crate::project::{Project, ContextManager};

pub fn decorate_recursive_file(
    project: &Project,
    context_manager: &mut ContextManager,
    context_name: &str,
) -> anyhow::Result<()> {
    let mut decorated_set = HashSet::new();
    _decorate_recursive_file(project, context_manager, context_name, &mut decorated_set)
}

fn _decorate_recursive_file(
    project: &Project,
    context_manager: &mut ContextManager,
    context_name: &str,
    decorated_set: &mut HashSet<String>,
) -> anyhow::Result<()> {
    if decorated_set.contains(context_name) {
        return Ok(());
    }
    decorated_set.insert(context_name.to_string());

    let context_lines = context_manager.get_context(context_name)?;
    let mut modified_current_context = false;

    // First pass: decorate the current context
    if crate::decorator::decorate_context_in_memory(context_lines)? {
        modified_current_context = true;
    }

    // Second pass: follow @include directives and decorate recursively
    let mut includes_to_decorate = Vec::new();
    for line in context_lines.iter() {
        if let Some(included_context_name) = line.get_include_path() {
            includes_to_decorate.push(included_context_name.to_string());
        }
    }

    for included_context_name in includes_to_decorate.into_iter() {
        _decorate_recursive_file(project, context_manager, &included_context_name, decorated_set)?;
    }

    if modified_current_context {
        context_manager.mark_as_modified(context_name);
    }

    Ok(())
}
