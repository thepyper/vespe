use anyhow::Result;
use std::collections::HashSet;

use crate::syntax::types::{Line, TagKind};
use crate::decorator;
use crate::project::{ContextManager, Project};

pub fn decorate_recursive_file(
    project: &Project,
    context_manager: &mut ContextManager,
    context_name: &str,
) -> Result<()> {
    let mut decorated_set = HashSet::new();
    _decorate_recursive_file(project, context_manager, context_name, &mut decorated_set)?;
    Ok(())
}

fn _decorate_recursive_file(
    project: &Project,
    context_manager: &mut ContextManager,
    context_name: &str,
    decorated_set: &mut HashSet<String>,
) -> Result<()> {
    // Decorate the current context and mark it as modified if changes occur.
    _decorate_and_mark_context(project, context_manager, context_name)?;
    decorated_set.insert(context_name.to_string());

    // Extract includes to decorate from the current context.
    let current_context_includes = {
        let context_lines = context_manager.load_context(project, context_name)?;
        let mut includes_to_decorate = Vec::new();
        for line in context_lines.iter() {
            if let Line::Tagged {
                tag: TagKind::Include,
                arguments,
                ..
            } = line
            {
                let include_path_str = arguments.first().map(|s| s.as_str()).unwrap_or("");
                if !decorated_set.contains(include_path_str) {
                    includes_to_decorate.push(include_path_str.to_string());
                }
            }
        }
        includes_to_decorate
    };

    // Recursively decorate included contexts.
    for included_context_name in current_context_includes.into_iter() {
        _decorate_recursive_file(
            project,
            context_manager,
            &included_context_name,
            decorated_set,
        )?;
    }

    Ok(())
}

fn _decorate_and_mark_context(
    project: &Project,
    context_manager: &mut ContextManager,
    context_name: &str,
) -> Result<()> {
    let context_lines = context_manager.load_context(project, context_name)?;
    let modified = decorator::decorate_context_in_memory(context_lines)?;
    if modified {
        context_manager.mark_as_modified(context_name);
    }
    Ok(())
}