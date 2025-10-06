use std::collections::HashSet;
use std::path::PathBuf;
use crate::project::{Project, Line};
use crate::decorator;

pub fn execute(project: &mut Project, context_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut decorated_set: HashSet<String> = HashSet::new();
    decorate_recursive_file(project, context_name, &mut decorated_set)?;
    // altre cose da fare dopo
    Ok(())
}

fn decorate_recursive_file(
    project: &mut Project,
    context_name: &str,
    decorated_set: &mut HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !decorated_set.insert(context_name.to_string()) {
        // Already decorated, avoid circular loops
        return Ok(());
    }

    // Load context_name as Vec<Line>
    let context_path = project.get_context_path(context_name)?;
    let content = std::fs::read_to_string(&context_path)?;
    let mut lines: Vec<Line> = content.lines().map(|s| Line::from(s.to_string())).collect();

    // Execute decorate on context_name
    let modified = decorator::decorate_context_in_memory(&mut lines)?;

    if modified {
        let new_content = lines.iter().map(|line| line.to_string()).collect::<Vec<String>>().join("\n");
        std::fs::write(&context_path, new_content)?;
    }

    // Cycle through lines, follow @include for recursive decoration
    for line in lines {
        if let Some(included_context) = extract_include_directive(&line.text) {
            decorate_recursive_file(project, &included_context, decorated_set)?;
        }
    }

    Ok(())
}

fn extract_include_directive(line: &str) -> Option<String> {
    if line.trim_start().starts_with("@include") {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 1 {
            return Some(parts[1].to_string());
        }
    }
    None
}
