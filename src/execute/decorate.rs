use anyhow::Result;
use std::collections::HashSet;

use crate::ast::types::{LineKind, TagKind};
use crate::decorator;
use crate::project::{ContextManager, Project};

pub fn decorate_recursive_file(
    project: &Project,
    context_manager: &mut ContextManager,
    context_name: &str,
) -> Result<()> {
    let context_lines = context_manager.load_context(project, context_name)?;

    let mut modified_current_context = false;

    let modified = decorator::decorate_context_in_memory(context_lines)?;
    if modified {
        modified_current_context = true;
    }

    let mut decorated_set = HashSet::new();
    decorated_set.insert(context_name.to_string());

    let _ = _decorate_recursive_file(
        project,
        context_manager,
        context_name,
        &mut decorated_set,
    )?;

    if modified_current_context {
        context_manager.mark_as_modified(context_name);
    }

    Ok(())
}

fn _decorate_recursive_file(
    project: &Project,
    context_manager: &mut ContextManager,
    context_name: &str,
    decorated_set: &mut HashSet<String>,
) -> Result<()> {
    if decorated_set.contains(context_name) {
        return Ok(());
    }
    decorated_set.insert(context_name.to_string());

    let context_lines = context_manager.load_context(project, context_name)?;

    let mut includes_to_decorate = Vec::new();

    for line in context_lines.iter() {
        if let LineKind::Tagged { tag: TagKind::Include, arguments: include_path, .. } = &line.kind {
            let include_path_str = include_path.first().map(|s| s.as_str()).unwrap_or("");
            if !decorated_set.contains(include_path_str) {
                includes_to_decorate.push(include_path_str.to_string());
            }
        }
    }

    for included_context_name in includes_to_decorate.into_iter() {
        decorated_set.insert(included_context_name.clone());
        let included_lines = context_manager.load_context(project, &included_context_name)?;

        let modified = decorator::decorate_context_in_memory(included_lines)?;
        if modified {
            context_manager.mark_as_modified(&included_context_name);
        }

        _decorate_recursive_file(
            project,
            context_manager,
            &included_context_name,
            decorated_set,
        )?;
    }

    Ok(())
}