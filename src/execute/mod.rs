use crate::project::{Project, ContextManager};
use crate::agent::ShellAgentCall;
use crate::ast::types::{Line, Anchor, AnchorTag, LineKind, TagKind};
use std::collections::{HashMap, BTreeMap};
use uuid::Uuid;
use anyhow::Result;

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

    decorate::decorate_recursive_file(project, &mut context_manager, context_name)?;
    inject::inject_recursive_inline(project, &mut context_manager, context_name)?;
    decorate::decorate_recursive_file(project, &mut context_manager, context_name)?;

    loop {
        let answered_a_question = answer::answer_first_question(project, &mut context_manager, context_name, agent)?;
        if !answered_a_question {
            break;
        }
    }

    context_manager.save_modified_contexts(project)?;

    Ok(())
}

enum Exe2Compitino {
    None,
    AnswerQuestion(uuid::Uuid),
    Summarize(uuid::Uuid),
}

pub fn execute2(project: &Project,
    context_name: &str,
    agent: &ShellAgentCall,
) -> anyhow::Result<()>  {

    let mut context_manager = ContextManager::new();
    let mut exe2_manager = Execute2Manager::new();

    loop {
        let compitino = _execute2(project, context_name, agent, &mut context_manager, &mut exe2_manager)?;

        match compitino {
            Exe2Compitino::None => break,
            Exe2Compitino::AnswerQuestion(id) => {
                // Handle answering question
            },
            Exe2Compitino::Summarize(id) => {
                // Handle summarizing
            },
        }
    }

    Ok(())
}

struct Execute2Manager 
{
    collect_content: Vec<Line>,
}

impl Execute2Manager {
    fn new() -> Execute2Manager {
        Execute2Manager {
            collect_content: Vec::new(),
        }
    }
}

struct AnchorIndex
{
	begin: HashMap<Uuid, usize>,
	end: HashMap<Uuid, usize>,
}

impl AnchorIndex 
{
	fn new(lines: &Vec<Line>) -> AnchorIndex {
		let a1 = HashMap::<Uuid, usize>::new();
		let a2 = HashMap::<Uuid, usize>::new();
		for (i, line) in lines.iter().enumerate() {
			if let Some(anchor) = line.get_anchor() {
				match anchor.tag {
					AnchorTag::Begin => { a1.insert(anchor.uid, i); },
					AnchorTag::End => { a2.insert(anchor.uid, i); },
					_ => {}
				}
			}
		}
		AnchorIndex {
			begin: a1,
			end: a2,
		}
	}
}

fn apply_patches(lines : &mut Vec<Line>, patches: BTreeMap::<usize, Vec<Line>>) -> Result<()> {

	// TODO apply patches in reverse order
	
	Ok(())
}



fn _execute2(project: &Project,
    context_name: &str,
    agent: &ShellAgentCall, context_manager: &mut ContextManager, exe2: &mut Execute2Manager,
) -> anyhow::Result<()>  {
	
    let mut lines = context_manager.load_context(project, context_name)?.clone();
	
	{
	let patches = BTreeMap::<usize, Vec<Line>>::new();

	// Check for missing tag anchors 
	for (i, line) in lines.iter().enumerate() {
		if let LineKind::Tagged{ tag, .. } = &line.kind {	
			let expected_begin_anchor_kind = match tag {
				TagKind::Inline => Some(AnchorKind::Inline),
				TagKind::Answer => Some(AnchorKind::Answer),
				TagKind::Summary => Some(AnchorKind::Summary),
				_ => None,
			};
			if let Some(expected_begin_anchor_kind) = expected_begin_anchor_kind {
				let is_anchor_ok = match line.anchor {
					None => false,
					Some(anchor) => if anchor.kind != expected_begin_anchor_kind { false } else { if let AnchorTag::Begin = anchor.tag { true} else { false }}
				};				if !is_anchor_ok {
					patches.insert(i, vec![Line { kind: line.kind, anchor: Some(Anchor { kind: expected_begin_anchor_kind, uid: Uuid::new_v4(), tag: AnchorTag::Begin }) }]);
				}
			}
		}
	}
	
	apply_patches(lines, patches);
	}
	
	{
	let patches = BTreeMap::<usize, Vec<Line>>::new();
	let anchor_index = AnchorIndex::new(lines);

	// Check for orphan end anchors
	for (anchor_end_uuid, i) in anchor_index.end {
		if !anchor_index.begin.contains(anchor_end_uuid) {
			// Orphan end anchor, remove it 
			patches.insert(i, Vec::new());
		}
	}
		
	// Check for orphan begin anchors
	for (anchor_begin_uuid, i) in anchor_index.begin {
		if !anchor_index.end.contains(anchor_begin_uuid) {
			// Orphan begin anchor, add end anchor just after it 
			let begin_anchor_line = lines.get(i).unwrap();
			patches.insert(i, vec![begin_anchor_line.clone(), Line{ kind: LineKind::Text("".to_string()), anchor: Some(Anchor{ kind: begin_anchor_line.anchor.as_ref().unwrap().kind, uid: *anchor_begin_uuid, tag: AnchorTag::End }) }]);
		}
	}	
	apply_patches(lines, patches);
	}

	
	
}