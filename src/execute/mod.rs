use crate::project::Project;
use crate::agent::ShellAgentCall;

pub mod answer;
pub mod decorate;
pub mod inject;

pub fn execute(
    project: &mut Project,
    context_name: &str,
    _agent: &ShellAgentCall,
) -> anyhow::Result<()> {
    let modified_decorate = decorate::decorate_recursive_file(project, context_name)?;
    let modified_inject = inject::inject_recursive_inline(project, context_name)?;
    let modified_answer = answer::answer_questions(project, context_name, _agent)?;

    if modified_decorate || modified_inject || modified_answer {
        // If any modification happened, we might need to re-process
        // For now, just return Ok. Further iterations might require a loop.
    }

    Ok(())
}
