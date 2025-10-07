use crate::agent::ShellAgentCall;
use crate::ast::types::{Anchor, AnchorKind, AnchorTag, Line, LineKind, TagKind};
use crate::project::{ContextManager, Project};
use anyhow::Result;
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;
use crate::execute::inject::InlineState;
use std::fs;
use serde_json;
use serde::{Deserialize, Serialize};


pub mod inject;



enum Exe2Compitino {
    None,
	Continue,
    AnswerQuestion{ uid: uuid::Uuid, content: Vec<Line> },
    Summarize{ uid: uuid::Uuid, content: Vec<Line> },
}

fn hash_content(lines : &Vec<Line>) -> String {
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
    agent: &ShellAgentCall,
) -> anyhow::Result<()> {
    let mut context_manager = ContextManager::new();
    let mut exe2_manager = Execute2Manager::new();

    loop {
        let compitino = _execute(
            project,
            context_name,
            agent,
            &mut context_manager,
            &mut exe2_manager,
        )?;

        match compitino {
            Exe2Compitino::None => break,
			Exe2Compitino::Continue => {},
            Exe2Compitino::AnswerQuestion{ uid, content } => {
                let content_str = format_document(content.clone()); // Clone content here
				let reply = agent.call(&content_str);
				
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
	
	context_manager.save_modified_contexts(project)?;

    Ok(())
}

fn format_document(lines: Vec<Line>) -> String {
    lines.into_iter().map(|line| {
        match line {
            Line::Text(s) => s,
            Line::Tagged { tag, arguments, .. } => format!("{} {}", tag.to_string(), arguments.join(" ")),
            Line::Anchor(anchor) => format!("{}", anchor),
        }
    }).collect::<Vec<String>>().join("\n")
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

struct AnchorIndex {
    begin: HashMap<Uuid, usize>,
    end: HashMap<Uuid, usize>,
}

impl AnchorIndex {
    fn new(lines: &Vec<Line>) -> AnchorIndex {
        let mut a1 = HashMap::<Uuid, usize>::new();
        let mut a2 = HashMap::<Uuid, usize>::new();
        for (i, line) in lines.iter().enumerate() {
            if let Line::Anchor(anchor) = line {
                match anchor.tag {
                    AnchorTag::Begin => {
                        a1.insert(anchor.uid, i);
                    }
                    AnchorTag::End => {
                        a2.insert(anchor.uid, i);
                    }
                    _ => {}
                }
            }
        }
        AnchorIndex { begin: a1, end: a2 }
    }

    fn get_begin_value(&self, uid: Uuid) -> usize {
        *self.begin.get(&uid).unwrap()
    }

    fn get_end_value(&self, uid: Uuid) -> usize {
        *self.end.get(&uid).unwrap()
    }
}

pub fn apply_patches(lines: &mut Vec<Line>, patches: BTreeMap<(usize, usize), Vec<Line>>) -> Result<()> {
    for ((i, n), patch) in patches.iter().rev() {
        lines.splice(*i..*i+n, patch.iter().cloned());
    }

    Ok(())
}

fn decorate_with_new_anchors(
    project: &Project,
    context_name: &str,
    context_manager: &mut ContextManager,
    lines: &mut Vec<Line>,
) -> anyhow::Result<()> {
    let mut patches = BTreeMap::<(usize, usize), Vec<Line>>::new();

    // Check for missing tag anchors
    for i in 0..lines.len() {
        if let Line::Tagged { tag, .. } = &lines[i] {
            let expected_begin_anchor_kind = match tag {
                TagKind::Inline => Some(AnchorKind::Inline),
                TagKind::Answer => Some(AnchorKind::Answer),
                TagKind::Summary => Some(AnchorKind::Summary),
                _ => None,
            };
            if let Some(expected_begin_anchor_kind) = expected_begin_anchor_kind {
                let mut is_anchor_ok = false;
                if i + 1 < lines.len() {
                    if let Line::Anchor(anchor) = &lines[i + 1] {
                        if anchor.kind == expected_begin_anchor_kind && anchor.tag == AnchorTag::Begin {
                            is_anchor_ok = true;
                        }
                    }
                }

                if !is_anchor_ok {
                    patches.insert(
                        (i, 1), // Replace the current line (the tag line) with the anchor
                        vec![Line::Anchor(Anchor {
                            kind: expected_begin_anchor_kind,
                            uid: Uuid::new_v4(),
                            tag: AnchorTag::Begin,
                        })],
                    );
                }
            }
        }
    }
    if !patches.is_empty() {
        apply_patches(lines, patches)?;
        context_manager.mark_as_modified(context_name);
    }
    Ok(())
}

fn check_for_orphan_anchors(
    context_name: &str,
    context_manager: &mut ContextManager,
    lines: &mut Vec<Line>,
) -> anyhow::Result<()> {
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

fn apply_inline(
    project: &Project,
    context_name: &str,
    context_manager: &mut ContextManager,
    lines: &mut Vec<Line>,
) -> anyhow::Result<Exe2Compitino> {
    let mut patches = BTreeMap::<(usize, usize), Vec<Line>>::new();
    let anchor_index = AnchorIndex::new(lines);

    // Apply inline tags if not done
    for i in 0..lines.len() {
        if let Line::Tagged { tag: TagKind::Inline, arguments, .. } = &lines[i] {
            // The tag line has been replaced by the anchor
            if let Line::Anchor(begin_anchor) = &lines[i] {
                if begin_anchor.tag == AnchorTag::Begin && begin_anchor.kind == AnchorKind::Inline {
                        let uid = begin_anchor.uid;
                        let anchor_metadata_dir = project.resolve_metadata(&AnchorKind::Inline.to_string(), &uid)?;
                        let state_file_path = anchor_metadata_dir.join("state.json");

                        let mut inline_state = InlineState::default();
                        if state_file_path.exists() {
                            let state_content = fs::read_to_string(&state_file_path)?;
                            inline_state = serde_json::from_str(&state_content)?;
                        }

                        if !inline_state.pasted {
                            let j = anchor_index.get_begin_value(uid);
                            let k = anchor_index.get_end_value(uid);
                            let snippet = project.load_snippet(arguments.first().unwrap().as_str())?;
                            patches.insert((j, k - j), snippet.content);

                            inline_state.pasted = true;
                            let updated_state_content = serde_json::to_string_pretty(&inline_state)?;
                            fs::write(&state_file_path, updated_state_content)?;
                        }
                    }
                }
            }
        }
    }

    if !patches.is_empty() {
        // Some inline applied, let's run all of this again
        apply_patches(lines, patches)?;
        context_manager.mark_as_modified(context_name);
        return Ok(Exe2Compitino::Continue);
    }
    Ok(Exe2Compitino::None)
}

fn apply_answer_summary(
    project: &Project,
    context_name: &str,
    agent: &ShellAgentCall,
    context_manager: &mut ContextManager,
    exe2: &mut Execute2Manager,
    lines: &mut Vec<Line>,
) -> anyhow::Result<Exe2Compitino> {
    let mut patches = BTreeMap::<(usize, usize), Vec<Line>>::new();
    let anchor_index = AnchorIndex::new(lines);

        for i in 0..lines.len() {
            match &lines[i] {
                Line::Text(_) => exe2.collect_content.push(lines[i].clone()),
                Line::Tagged { tag, arguments, .. } => {
                    match tag {
                        TagKind::Summary => {
                            let mut exe2_sub_manager = Execute2Manager::new();
                            // Execute content to summarize, can only summarize content that is completely executed
                            match _execute(project, arguments.first().unwrap().as_str(), agent, context_manager, &mut exe2_sub_manager) {
                                Ok(Exe2Compitino::None) => {
                                    // The tag line has been replaced by the anchor
                                    if let Line::Anchor(begin_anchor) = &lines[i] {
                                        if begin_anchor.tag == AnchorTag::Begin && begin_anchor.kind == AnchorKind::Summary {
                                            return Ok(Exe2Compitino::Summarize { uid: begin_anchor.uid, content: exe2_sub_manager.collect_content });
                                        }
                                    }
                                    return Ok(Exe2Compitino::None); // Should not happen if decorate_with_new_anchors works
                                }
                                x => { return x; }
                            }
                        }
                        TagKind::Answer => {
                            // The tag line has been replaced by the anchor
                            if let Line::Anchor(begin_anchor) = &lines[i] {
                                if begin_anchor.tag == AnchorTag::Begin && begin_anchor.kind == AnchorKind::Answer {
                                    let uid = begin_anchor.uid;
                                    let answer_state = AnswerState2::default(); // TODO carica da metadata
                                    if answer_state.content_hash.is_empty() {
                                        // Mai risposta la domanda, lancia compitino
                                        return Ok(Exe2Compitino::AnswerQuestion { uid: uid, content: exe2.collect_content.clone() });
                                    } else if answer_state.reply_hash.is_empty() {
                                        // Nessuna rispota ancora
                                    } else if answer_state.reply_hash != answer_state.injected_hash {
                                        // Disponibile una nuova risposta, iniettala
                                        let j = anchor_index.get_begin_value(uid);
                                        let k = anchor_index.get_end_value(uid);
                                        let reply_lines: Vec<Line> = answer_state.reply.lines().map(|s| Line::Text(s.to_string())).collect();
                                        patches.insert((j, k - j), reply_lines);
                                    }
                                }
                            }
                        }
                        _ => {},
                    }
                }
                Line::Anchor(_) => { /* Do nothing, anchors are handled by other logic */ }
            }
        }
    if !patches.is_empty() {
        apply_patches(lines, patches)?;
        context_manager.mark_as_modified(context_name);
    }
    Ok(Exe2Compitino::None)
}


fn _execute(
    project: &Project,
    context_name: &str,
    _agent: &ShellAgentCall,
    context_manager: &mut ContextManager,
    _exe2: &mut Execute2Manager,
) -> anyhow::Result<Exe2Compitino> {
    let mut lines = context_manager.load_context(project, context_name)?.clone();

    decorate_with_new_anchors(project, context_name, context_manager, &mut lines)?;
    check_for_orphan_anchors(context_name, context_manager, &mut lines)?;

    let inline_result = apply_inline(project, context_name, context_manager, &mut lines)?;
    if let Exe2Compitino::Continue = inline_result {
        return Ok(Exe2Compitino::Continue);
    }

    let answer_summary_result = apply_answer_summary(project, context_name, _agent, context_manager, _exe2, &mut lines)?;
    if let Exe2Compitino::Summarize { uid, content } = answer_summary_result {
        return Ok(Exe2Compitino::Summarize { uid, content });
    }
    if let Exe2Compitino::AnswerQuestion { uid, content } = answer_summary_result {
        return Ok(Exe2Compitino::AnswerQuestion { uid, content });
    }

    Ok(Exe2Compitino::None)
}