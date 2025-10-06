use std::collections::HashSet;
use crate::project::{Project, ContextManager};

pub fn decorate_recursive_file(
    project: &Project,
    context_manager: &mut ContextManager,
    context_name: &str,
) -> anyhow::Result<bool> {
    let mut decorated_set = HashSet::new();
    let modified = _decorate_recursive_file(project, context_manager, context_name, &mut decorated_set)?;

    if modified {
        context_manager.mark_as_modified(context_name);
    }
    Ok(modified)
}

fn _decorate_recursive_file(
    project: &Project,
    context_manager: &mut ContextManager,
    context_name: &str,
    decorated_set: &mut HashSet<String>,
) -> anyhow::Result<bool> {
    if decorated_set.contains(context_name) {
        return Ok(false);
    }
    decorated_set.insert(context_name.to_string());

    let mut modified = false;

    let context_lines = context_manager.get_context(context_name)?;

    // First pass: decorate the current context
    if crate::decorator::decorate_context_in_memory(context_lines)? {
        modified = true;
    }

    // Second pass: follow @include directives and decorate recursively
    let mut includes_to_decorate = Vec::new();
    for line in context_lines.iter() {
        if let Some(included_context_name) = line.get_include_path() {
            includes_to_decorate.push(included_context_name.to_string());
        }
    }

    for included_context_name in includes_to_decorate.into_iter() {
        if _decorate_recursive_file(project, context_manager, &included_context_name, decorated_set)? {
            context_manager.mark_as_modified(&included_context_name);
            modified = true;
        }
    }

    Ok(modified)
}
