use anyhow::Result;
use std::collections::HashSet;
use crate::project::Project;
use crate::ast::types::Line;
use crate::decorator;
use crate::injector;

pub fn execute(project: &Project, context_name: &str) -> Result<()> {
    // Load context context_name as Vec<Line>
    let mut context_lines = project.load_context_lines(context_name)?;

    // Call decorate_recursive_file
    let mut decorated_set = HashSet::new();
    decorate_recursive_file(project, context_name, &mut context_lines, &mut decorated_set)?;

    inject_recursive_inline(project, context_name)?;

    // Other things to do after
    // For now, let’s just write the decorated content back to a temporary file or update the project’s in-memory representation
    // This part will need to be refined based on how the ‘project’ struct manages its contexts.
    // For now, let’s assume we update the project’s in-memory context.
    project.update_context_lines(context_name, context_lines)?;


    Ok(())
}

fn decorate_recursive_file(
    project: &Project,
    context_name: &str,
    lines: &mut Vec<Line>,
    decorated_set: &mut HashSet<String>,
    ) -> Result<()> {
    if !decorated_set.insert(context_name.to_string()) {
        // Already decorated, prevent circular loops
        return Ok(());
    }

    // Execute decorate on the current context lines
    let _ = decorator::decorate_context_in_memory(lines)?;

    // Iterate over lines, follow @include to perform recursive decoration
    let mut included_contexts_to_decorate = Vec::new();
    for line in lines.iter() {
        if let Some(include_path) = line.get_include_path() {
            included_contexts_to_decorate.push(include_path.to_string());
        }
    }

    for included_context_name in included_contexts_to_decorate {
        let mut included_lines = project.load_context_lines(&included_context_name)?;
        decorate_recursive_file(project, &included_context_name, &mut included_lines, decorated_set)?;
        project.update_context_lines(&included_context_name, included_lines)?;
    }

    Ok(())
}

pub fn inject_recursive_inline(
    project: &Project,
    context_name: &str,
) -> Result<()> {
    let mut inlined_set = HashSet::new();
    _inject_recursive_inline(project, context_name, &mut inlined_set)?;
    Ok(())
}

fn _inject_recursive_inline(
    project: &Project,
    context_name: &str,
    inlined_set: &mut HashSet<String>,
) -> Result<()> {
    if !inlined_set.insert(context_name.to_string()) {
        // Already inlined, prevent circular loops
        return Ok(());
    }

    let mut lines = project.load_context_lines(context_name)?;
    let mut modified = false;

    // Iterate over lines, find those with inline tags, with anchor begin
    let mut i = 0;
    while i < lines.len() {
        if let Some((anchor_kind, anchor_uid, snippet_name)) = lines[i].get_inline_tag_info() {
            // Load the referred snippet
            let snippet_lines = project.load_snippet_lines(&snippet_name)?;

            // Use injector::inject_content_in_memory
            let injection_result = injector::inject_content_in_memory(
                &mut lines,
                anchor_kind,
                anchor_uid,
                snippet_lines,
            )?;
            if injection_result {
                modified = true;
            }
        }
        i += 1;
    }

    // Recursively process @include tags
    let mut included_contexts_to_inline = Vec::new();
    for line in lines.iter() {
        if let Some(include_path) = line.get_include_path() {
            included_contexts_to_inline.push(include_path.to_string());
        }
    }

    for included_context_name in included_contexts_to_inline {
        _inject_recursive_inline(project, &included_context_name, inlined_set)?;
    }

    if modified {
        project.update_context_lines(context_name, lines)?;
    }

    Ok(())
}