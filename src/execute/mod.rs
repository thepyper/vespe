use crate::project::Project;
use crate::agent::ShellAgentCall;

mod decorate;
mod inject;

pub fn execute(
    project: &mut Project,
    context_name: &str,
    _agent: &ShellAgentCall,
) -> anyhow::Result<()> {
    // carica context context_name come Vec<Line>

    decorate::decorate_recursive_file(project, context_name)?;
    inject::inject_recursive_inline(project, context_name)?;

    // altre cose da fare dopo

    Ok(())
}
