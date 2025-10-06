use crate::project::{Project, ContextManager};
use crate::agent::ShellAgentCall;

pub mod answer;
pub mod decorate;
pub mod inject;

pub fn execute(
    project: &Project,
    context_name: &str,
    agent: &ShellAgentCall,
) -> anyhow::Result<()> {
    let mut context_manager = ContextManager::new();

    // Load the initial context
    context_manager.load_context(project, context_name)?;

    let modified_decorate = decorate::decorate_recursive_file(project, &mut context_manager, context_name)?;
    let modified_inject = inject::inject_recursive_inline(project, &mut context_manager, context_name)?;
    let modified_answer = answer::answer_questions(project, &mut context_manager, context_name, agent)?;

    if modified_decorate || modified_inject || modified_answer {
        // If any modification happened, we might need to re-process
        // For now, just return Ok. Further iterations might require a loop.
    }

    context_manager.save_modified_contexts(project)?;

    Ok(())
}
