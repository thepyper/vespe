
pub mod states;

use crate::agent::ShellAgentCall;
use crate::project::Project;
use crate::semantic::{self, Context, Line, Patches};
use crate::execute::states::{AnswerState, AnswerStatus, InlineState, SummaryState};
use crate::utils::AnchorIndex;
use serde::{Deserialize, Serialize};
use tracing::{debug};


#[derive(Debug)]
enum Exe2Compitino {
    None,
	Continue,
    AnswerQuestion{ uid: uuid::Uuid, content: Vec<Line> },
    // Summarize{ uid: uuid::Uuid, content: Vec<Line> },
}

fn hash_content(_lines : &Vec<Line>) -> String {
	// TODO hash da lines
	unimplemented!()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnswerState2 {
    pub content_hash: String,
	pub reply_hash: String,
	pub reply: String,
	pub injected_hash: String,
}

impl Default for AnswerState2 {
    fn default() -> Self {
        AnswerState2 {
            content_hash: String::new(),
            reply_hash: String::new(),
            reply: String::new(),
            injected_hash: String::new(),
        }
    }
}

pub fn execute(
    project: &Project,
    context_name: &str,
    _agent: &ShellAgentCall,
) -> anyhow::Result<()> {
    debug!("Executing context: {}", context_name);
    
    let mut exe2_manager = Execute2Manager::new();

    loop {
        debug!("Starting _execute loop for context: {}", context_name);
        let compitino = _execute(
            project,
            context_name,
            &mut exe2_manager,
        )?;
        debug!("_execute returned: {:?}", compitino);
        match compitino {
            Exe2Compitino::None => {
                debug!("Exe2Compitino::None received, breaking loop.");
                break;
            },
			Exe2Compitino::Continue => {
                debug!("Exe2Compitino::Continue received, continuing loop.");
            },
            Exe2Compitino::AnswerQuestion{ uid: _uid, content } => {
                debug!("Exe2Compitino::AnswerQuestion received for UID: {:?}", _uid);

                let mut state = project.load_answer_state(&_uid)?;
                if state.status != AnswerStatus::NeedAnswer {
                    debug!("State status is not NeedAnswer, continuing loop.");
                    continue;
                }
                // TODO: dovrei fare in qulche ltro modo non con compitino... ricerca estensiva domande da rispondere?
                // limitata al context in corso? comunque...

				let reply = _agent.call(&state.query)?;
                
                state.status = AnswerStatus::NeedInjection;
                state.reply = reply;
                
                project.save_answer_state(&_uid, &state)?;

                debug!("AnswerQuestion processed for UID: {:?}", _uid);
            }
        }
    }
	
    debug!("Context execution finished for: {}", context_name);
    Ok(())
}

struct Execute2Manager {
    collect_content: Vec<Line>,
}

impl Execute2Manager {
    fn new() -> Execute2Manager {
        Execute2Manager {
            collect_content: Vec::new(),
        }
    }
}

fn decorate_with_new_anchors(
    project: &Project,
    context: &mut Context,
) -> anyhow::Result<()> {
    debug!("Decorating context with new anchors.");
  
    let mut patches = Patches::new();

    for (i, line) in context.lines.iter().enumerate() {
        match line {
            Line::InlineTag { snippet_name } => {
                debug!("Found InlineTag at line {}: {}", i, snippet_name);
                let uuid = uuid::Uuid::new_v4();
                let anchors = semantic::Line::new_inline_anchors(uuid);
                project.save_inline_state(&uuid, &InlineState::new(&snippet_name))?;
                patches.insert(
                    (i, i + 1), // Replace the current line (the tag line) with the anchor
                    anchors,
                );
                debug!("Inserted inline anchors for snippet: {}", snippet_name);
            }
            Line::AnswerTag => {
                debug!("Found AnswerTag at line {}", i);
                let uuid = uuid::Uuid::new_v4();
                let anchors = semantic::Line::new_answer_anchors(uuid);
                project.save_answer_state(&uuid, &AnswerState::new())?;
                patches.insert(
                    (i, i + 1), // Replace the current line (the tag line) with the anchor
                    anchors,
                );
                debug!("Inserted answer anchors.");
            }
            Line::SummaryTag { context_name } => {
                debug!("Found SummaryTag at line {}: {}", i, context_name);
                let uuid = uuid::Uuid::new_v4();
                let anchors = semantic::Line::new_summary_anchors(uuid);
                project.save_summary_state(&uuid, &SummaryState::new(&context_name))?;
                patches.insert(
                    (i, i + 1), // Replace the current line (the tag line) with the anchor
                    anchors,
                );
                debug!("Inserted summary anchors for context: {}", context_name);
            }
            _ => { /* Do nothing */ }
        }
    }

    if context.apply_patches(patches) {
        debug!("Patches applied in decorate_with_new_anchors.");
        debug!("Modified context is now\n***\n{}\n***\n", semantic::format_document(&context.lines));
    } else {
        debug!("No patches applied in decorate_with_new_anchors.");
    }
    Ok(())
    
}

/* TODO redo this
fn check_for_orphan_anchors(
    context_name: &str,
    context_manager: &mut ContextManager,
) -> anyhow::Result<()> {
    let mut lines = context_manager.get_context(context_name)?;
    let mut patches = BTreeMap::<(usize, usize), Vec<Line>>::new();
    let anchor_index = AnchorIndex::new(lines);

    // Check for orphan end anchors
    for (anchor_end_uuid, i) in &anchor_index.end {
        if !anchor_index.begin.contains_key(anchor_end_uuid) {
            // Orphan end anchor, remove it (or replace with a blank line if needed)
            patches.insert((*i, 1), vec![Line::Text("".to_string())]); // Replace with a blank text line
        }
    }

    // Check for orphan begin anchors
    for (anchor_begin_uuid, i) in &anchor_index.begin {
        if !anchor_index.end.contains_key(anchor_begin_uuid) {
            // Orphan begin anchor, add end anchor just after it
            if let Line::Anchor(begin_anchor) = &lines[*i] {
                patches.insert(
                    (*i + 1, 0),
                    vec![Line::Anchor(Anchor {
                        kind: begin_anchor.kind.clone(),
                        uid: *anchor_begin_uuid,
                        tag: AnchorTag::End,
                    })],
                );
            }
        }
    }

    if !patches.is_empty() {
        apply_patches(lines, patches)?;
        context_manager.mark_as_modified(context_name);
    }
    Ok(())
}
    */

fn apply_inline(
    project: &Project,
    context: &mut Context,
) -> anyhow::Result<Exe2Compitino> {
    debug!("Applying inline snippets.");

    let anchor_index = AnchorIndex::new(&context.lines);

    let mut patches = Patches::new();

    for (i, line) in context.lines.iter().enumerate() {
        if let Line::InlineBeginAnchor { uuid } = line {
            debug!("Found InlineBeginAnchor at line {} with UUID: {}", i, uuid);
            let state = project.load_inline_state(uuid)?;
            if !state.pasted {
                debug!("Snippet '{}' not yet pasted. Pasting now.", state.snippet_name);
                let j = anchor_index.get_end(&uuid).ok_or_else(|| anyhow::anyhow!("End anchor not found for UUID: {}", uuid))?;
                let snippet = project.load_snippet(&state.snippet_name)?;
                let enriched_snippet = semantic::enrich_syntax_document(project, &snippet.content)?;
                patches.insert(
                    (i + 1, j),
                    enriched_snippet,
                );
                
                // Update state to mark as pasted
                let mut new_state = state.clone();
                new_state.pasted = true;
                project.save_inline_state(uuid, &new_state)?;
                
                debug!("Snippet '{}' pasted and state updated.", state.snippet_name);
            }
        }
    }

    if context.apply_patches(patches) {
        debug!("Patches applied in apply_inline. Continuing execution.");
        debug!("Modified context is now\n***\n{}\n***\n", semantic::format_document(&context.lines));
        Ok(Exe2Compitino::Continue)
    } else {
        debug!("No patches applied in apply_inline. No more inlines to process.");
        Ok(Exe2Compitino::None)
    }
}

fn apply_answer_summary(
    project: &Project,
    context: &mut Context,
    exe2: &mut Execute2Manager,
) -> anyhow::Result<Exe2Compitino> {
    debug!("Applying answer and summary processing.");

    let anchor_index = AnchorIndex::new(&context.lines);

    let mut patches = Patches::new();

    let mut compitino = Exe2Compitino::None;

    for (i, line) in context.lines.iter().enumerate() {
        match line {
             Line::Text(x) => {
                debug!("Collecting text line: {}", x);
                exe2.collect_content.push(Line::Text(x.clone()))
            },
             Line::AnswerBeginAnchor { uuid } => {
                let state = project.load_answer_state(uuid)?;
                debug!("Found AnswerBeginAnchor at line {} with UUID: {} and status: {:?}", i, uuid, state.status);
                let j = anchor_index.get_end(&uuid).ok_or_else(|| anyhow::anyhow!("End anchor not found for UUID: {}", uuid))?;
                match state.status {
                    AnswerStatus::NeedContext => {
                        debug!("AnswerStatus::NeedContext: Requesting answer for UUID: {}", uuid);
                        // The tag line has been replaced by the anchor
                        let uid = uuid.clone();
                        let mut updated_state = state.clone();
                        updated_state.status = AnswerStatus::NeedAnswer;
                        project.save_answer_state(uuid, &updated_state)?;
                        compitino = Exe2Compitino::AnswerQuestion { uid: uid, content: exe2.collect_content.clone() };
                        break;
                    },
                    AnswerStatus::NeedAnswer => {
                        debug!("AnswerStatus::NeedAnswer: Waiting for answer for UUID: {}", uuid);
                        // Do nothing, wait for answer to be provided
                    },
                    AnswerStatus::NeedInjection => {
                        debug!("AnswerStatus::NeedInjection: Injecting answer for UUID: {}", uuid);
                        patches.insert(
                            (i + 1, j),
                            state.reply.lines().map(|s| Line::Text(s.to_string())).collect(),
                        );
                        let mut updated_state = state.clone();
                        updated_state.status = AnswerStatus::Completed;
                        project.save_answer_state(uuid, &updated_state)?;
                        debug!("Answer injected and state updated for UUID: {}", uuid);
                    },
                    AnswerStatus::Completed => {
                        debug!("AnswerStatus::Completed: Answer already processed for UUID: {}", uuid);
                        // Do nothing, already completed
                    },
                }

             },
             Line::IncludeTag { context_name } => {
                 debug!("Found IncludeTag for context: {}", context_name);
                 let included_compitino = _execute(project, &context_name, exe2)?;
                 match included_compitino {
                        Exe2Compitino::None => {
                            debug!("Included context '{}' processed without compitino.", context_name);
                            // Do nothing, continue processing
                        },
                        _ => {
                            debug!("Included context '{}' returned compitino: {:?}. Propagating.", context_name, included_compitino);
                            // Included needs attention, propagate it up
                            compitino = included_compitino;
                            break;
                        }
                 }
             },
             _ => { /* Do nothing */ },
            }
    };
 
    if context.apply_patches(patches) {
        debug!("Patches applied in fn apply_answer_summary. Continuing execution.");
        debug!("Modified context is now\n***\n{}\n***\n", semantic::format_document(&context.lines));
    }
    Ok(compitino) 
}

fn _execute(
    project: &Project,
    context_name: &str,
    exe2: &mut Execute2Manager,
) -> anyhow::Result<Exe2Compitino> {

    let mut compitino = Exe2Compitino::None;

    debug!("Starting _execute for context: {}", context_name);

    let mut context = Context::load(project, context_name)?;
    debug!("Context '{}' loaded.", context_name);

    if let Exe2Compitino::None = compitino {
        debug!("Calling decorate_with_new_anchors for context: {}", context_name);
        decorate_with_new_anchors(project, &mut context)?;
    }

    // TODO let orphans_checked = check_for_orphan_anchors(context_name, context_manager)?;

    if let Exe2Compitino::None = compitino {
        debug!("Calling apply_inline for context: {}", context_name);
        compitino = apply_inline(project, &mut context)?;
    }

    if let Exe2Compitino::None = compitino {
        debug!("Calling apply_answer_summary for context: {}", context_name);
        compitino = apply_answer_summary(project, &mut context, exe2)?;
    }

    debug!("Saving context: {}", context_name);
    context.save()?;

    debug!("_execute finished for context: {}", context_name);
    Ok(compitino)
}