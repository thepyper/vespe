use std::collections::HashSet;
use crate::project::Project;
use crate::ast::types::Line;

pub fn decorate_recursive_file(
    project: &mut Project,
    context_name: &str,
) -> anyhow::Result<()> {
    let mut decorated_set = HashSet::new();
    let mut context_lines = project.load_context_lines(context_name)?;

    _decorate_recursive_file(project, context_name, &mut context_lines, &mut decorated_set)?;

    project.update_context_lines(context_name, context_lines)?;
    Ok(())
}

fn _decorate_recursive_file(
    project: &mut Project,
    context_name: &str,
    context_lines: &mut Vec<Line>,
    decorated_set: &mut HashSet<String>,
) -> anyhow::Result<()> {
    if decorated_set.contains(context_name) {
        return Ok(());
    }
    decorated_set.insert(context_name.to_string());

    // First pass: decorate the current context
    crate::decorator::decorate_context_in_memory(context_lines)?;

    // Second pass: follow @include directives and decorate recursively
    let mut includes_to_decorate = Vec::new();
    for line in context_lines.iter() {
        if let Some(included_context_name) = line.get_include_path() {
            includes_to_decorate.push(included_context_name.to_string());
        }
    }

    for included_context_name in includes_to_decorate.into_iter() {
        let mut included_lines = project.load_context_lines(&included_context_name)?;
        _decorate_recursive_file(project, &included_context_name, &mut included_lines, decorated_set)?;
        project.update_context_lines(&included_context_name, included_lines)?;
    }

    Ok(())
}
