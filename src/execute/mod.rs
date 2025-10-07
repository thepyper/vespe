
//use crate::syntax::types::{Anchor, AnchorKind, AnchorTag, Line, TagKind};
use crate::project::Project;
use crate::semantic::{self, AnswerStatus, AnswerState, InlineState, Line, SummaryState};
use crate::syntax::parser::format_document;
use crate::utils::{AnchorIndex, Context, ContextManager, Patches};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use uuid::Uuid;


enum Exe2Compitino {
    None,
	Continue,
    AnswerQuestion{ uid: uuid::Uuid, content: Vec<Line> },
    Summarize{ uid: uuid::Uuid, content: Vec<Line> },
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
) -> anyhow::Result<()> {
    //let mut context_manager = ContextManager::new();
    
    let mut context_manager = ContextManager::new();
    let mut exe2_manager = Execute2Manager::new();

    //let mut lines = context_manager.load_context(project, context_name)?;

    loop {
        let compitino = _execute(
            project,
            context_name,
            &mut context_manager,
            &mut exe2_manager,
        )?;
        match compitino {
            Exe2Compitino::None => break,
			Exe2Compitino::Continue => {},
            Exe2Compitino::AnswerQuestion{ uid: _uid, content } => {
                let content_str = semantic::format_document(&content); // Clone content here
                // TODO: get reply from somewhere
				let reply: Result<String, anyhow::Error> = Ok("".to_string());
				
				let mut answer_state = AnswerState2::default();
				
				answer_state.content_hash = hash_content(&content);
				let actual_reply = reply?;
				answer_state.reply        = actual_reply.clone();
				answer_state.reply_hash   = hash_content(&actual_reply.lines().map(|s| Line::Text(s.to_string())).collect());
				
				// TODO save answer_state 
            }
            Exe2Compitino::Summarize{..} => {
                // TODO summarize the data with llm, save data into summary meta file, so on next _execute2 call content will be patched into context 
				// must save hash of content as well for future comparison
            }
        }
    }
	
	    context_manager.save_modified_contexts();
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
  
    let mut patches = Patches::new();

    for (i, line) in context.lines.iter().enumerate() {
        match line {
            Line::InlineTag { snippet_name } => {
                let anchors = semantic::Line::new_inline_anchors(InlineState::new(snippet_name));
                for anchor in &anchors {
                    anchor.save_state(project)?;
                }
                patches.insert(
                    (i, i + 1), // Replace the current line (the tag line) with the anchor
                    anchors,
                );
            }
            Line::AnswerTag => {
                let anchors = semantic::Line::new_answer_anchors(AnswerState::new());
                for anchor in &anchors {
                    anchor.save_state(project)?;
                }
                patches.insert(
                    (i, i + 1), // Replace the current line (the tag line) with the anchor
                    anchors,
                );
            }
            Line::SummaryTag { context_name } => {
                let anchors = semantic::Line::new_summary_anchors(SummaryState::new(context_name));
                for anchor in &anchors {
                    anchor.save_state(project)?;
                }
                patches.insert(
                    (i, i + 1), // Replace the current line (the tag line) with the anchor
                    anchors,
                );
            }
            _ => { /* Do nothing */ }
        }
    }

    context.apply_patches(patches);
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

    let anchor_index = AnchorIndex::new(&context.lines);

    let mut patches = Patches::new();

    for (i, line) in context.lines.iter().enumerate() {
        if let Line::InlineBeginAnchor { uuid, state } = line {
            if !state.pasted {
                let j = anchor_index.get_end(uuid).ok_or_else(|| anyhow::anyhow!("End anchor not found for UUID: {}", uuid))?;
                let snippet = project.load_snippet(&state.snippet_name)?;
                let enriched_snippet = semantic::enrich_syntax_document(project, &snippet.content)?;
                patches.insert(
                    (i + 1, j),
                    enriched_snippet,
                );
                
                // Update state to mark as pasted
                let mut new_line = line.clone();
                if let Line::InlineBeginAnchor { state, .. } = &mut new_line {
                    state.pasted = true;
                }
                
                patches.insert((i, i + 1), vec![new_line]); // Replace the begin anchor with updated state
            }
        }
    }

    if context.apply_patches(patches) {
        Ok(Exe2Compitino::Continue)
    } else {
        Ok(Exe2Compitino::None)
    }
}
fn apply_answer_summary(
    project: &Project,
    context: &mut Context,
    exe2: &mut Execute2Manager,
) -> anyhow::Result<Exe2Compitino> {

        let anchor_index = AnchorIndex::new(&context.lines);

    let mut patches = Patches::new();

    let mut compitino = Exe2Compitino::None;

    for (i, line) in context.lines.iter().enumerate() {
        match line {
             Line::Text(x) => exe2.collect_content.push(Line::Text(x.clone())),
             Line::AnswerBeginAnchor { uuid, state } => { 
                let j = anchor_index.get_end(uuid).ok_or_else(|| anyhow::anyhow!("End anchor not found for UUID: {}", uuid))?;
                match state.status {
                    AnswerStatus::NeedContext => {
                        // The tag line has been replaced by the anchor
                        let uid = *uuid;
                        let mut new_line = line.clone();
                        if let Line::AnswerBeginAnchor { state, .. } = &mut new_line {
                            state.status = AnswerStatus::NeedAnswer;
                        }
                        patches.insert((i, i + 1), vec![new_line]); // Replace the begin anchor with updated state
                        compitino = Exe2Compitino::AnswerQuestion { uid: uid, content: exe2.collect_content.clone() };
                        break;
                    },
                    AnswerStatus::NeedAnswer => {
                        // Do nothing, wait for answer to be provided
                    },
                    AnswerStatus::NeedInjection => {
                        patches.insert(
                            (i + 1, j),
                            state.reply.lines().map(|s| Line::Text(s.to_string())).collect(),
                        );
                        let mut new_line = line.clone();
                        if let Line::AnswerBeginAnchor { state, .. } = &mut new_line {
                            state.status = AnswerStatus::Completed;
                        }
                        patches.insert((i, i + 1), vec![new_line]); // Replace the begin anchor with updated state
                    },
                    AnswerStatus::Completed => {
                        // Do nothing, already completed
                    },
                }

             },
             Line::IncludeTag { context_name } => {
                 let included_compitino = _execute(project, &context_name, context_manager, exe2)?;
                 match included_compitino {
                        Exe2Compitino::None => {
                            // Do nothing, continue processing
                        },
                        _ => {
                            // Included needs attention, propagate it up
                            compitino = included_compitino;
                            break;
                        }
                 }
             },
             _ => { /* Do nothing */ },
            }
    };
 
    Ok(compitino) 
}

fn _execute(
    project: &Project,
    context_name: &str,
    context_manager: &mut ContextManager,
    exe2: &mut Execute2Manager,
) -> anyhow::Result<Exe2Compitino> {

    let context = Context::load(project, context_name)?;

    decorate_with_new_anchors(project, context)?;

    // TODO let orphans_checked = check_for_orphan_anchors(context_name, context_manager)?;

    match apply_inline(project, context)? {
        Exe2Compitino::None => {},
        compitino => return Ok(compitino),
    }

    match apply_answer_summary(project, context, exe2)? {
        Exe2Compitino::None => {},
        compitino => return Ok(compitino),  
    }

    Ok(Exe2Compitino::None)
}