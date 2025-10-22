
use state::*;
use content::*;

use crate::ast2::{parse_document, Anchor, AnchorKind, CommandKind, Document, Range, Tag};
use anyhow::Result;

pub fn execute_context(file_access: &file::FileAccessor, path_res: &path::PathResolver, context_name: &str) -> Result<()> {

    let exe = Executor::new(file_access, path_res);
    let _ = exe.execute_loop(context_name)?;
}

//type State = serde_json::Value;

struct Executor {
    file_access: &file::FileAccessor, 
    path_res: &path::PathResolver,
    visited: HashSet<String>,
    prelude: Vec<ContentItem>,
    context: Vec<ContentItem>,
}

impl Executor {
    fn new(file_access: &file::FileAccessor, path_res: &path::PathResolver) -> Self {
        Executor {
            file_access,
            path_res,
            visited: HashSet::new(),
            prelude: Vec::new(),
            context: Vec::new(),
        }
    }
    fn execute_loop(&self, context_name: &str) -> Result<Content> {
        let context_path = self.path_res.resolve_context(context_name);

        if self.visited.contains(context_path) {
            return;
        }

        while execute_step(context_path) {}
    }
    fn execute_step(&self, context_path: &Path) -> Result<bool> {
        // Read file, parse it, execute slow things that do not modify context
        let context = self.file_access.read_file(context_path)?;
        let ast = crate::ast2::parse_document(context)?;
        let want_next_step_1 = pass_1(ast);

        // Lock file, re-read it (could be edited outside), parse it, execute fast things that may modify context and save it
        self.file_access.lock_file(context_path)?;
        let context = self.file_access.read_file(context_path)?;
        let ast = crate::ast2::parse_document(context)?;
        let mut patches = utils::Patches::new(context);
        let want_next_step_2 = pass_2(ast, patches);
        if !patches.is_empty() {
            let context = patches.apply_patches();
            self.file_access.write_file(context_path, context, None)?; // TODO comment?
        }
        self.file_access.unlock_file(context_path)?;

        Ok(want_next_step_1 | want_next_step_2)
    }

    fn pass_1(&self, ast: &Document) -> Result<bool> {
        
        for item in ast.content {
            match item {
                Text(text) => self.pass_1_text(text)?,
                Tag(tag) => {
                    if self.pass_1_tag(tag)? {
                        return Ok(true);
                    }
                }
                Anchor(anchor) => {
                    if self.pass_1_anchor(tag)? {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    fn pass_1_text(&self, text: &Text) -> Result<()> {
        self.context.push(ContentItem::user(text.text));
    }

    fn pass_1_tag(&self, tag: &Tag) -> Result<bool> {
        match tag.command {
            CommandKind::Include => self.pass_1_include_tag(tag),
            _ => Ok(false),
        }
    }

    fn pass_1_include_tag() {
        let included_context = tag.validate_argument_as_context(0)?;
        self.execute_loop(included_context);
    }

    fn pass_1_anchor(&self, anchor: &Anchor) -> Result<bool> {
        let asm = utils::AnchorStateManager::new(self.file_access, self.path_res, anchor);
        match (
            anchor.command,
            anchor.kind,            
        ) {
            (CommandKind::Answer, AnchorKind::Begins) => {
                want_next_step |= self.pass_1_answer_begin_anchor(asm, anchor.parameters, anchor.arguments)
            } // passa commit?
            (CommandKind::Derive, AnchorKind::Begins) => {
                want_next_step |= self.pass_1_derive_begin_anchor(asm, anchor.parameters, anchor.arguments)
            } // passa commit?
            (CommandKind::Inline, AnchorKind::Begin) => {
                want_next_step |= self.pass_1_inline_begin_anchor(asm, anchor.parameters, anchor.arguments)
            } // passa commit?          
            _ => {}
        }
    }

    fn pass_1_answer_begin_anchor(
        &self,
        asm: &utils::AnchorStateManager,
        parameters: &Parameters,
        arguments: &Arguments,
    ) -> Result<bool> {
        let mut state : AnswerState = asm.load_state();
        match state.status {
            AnchorStatus::JustCreated => {
                state.query = self.context.clone();
                state.status = AnchorStatus::NeedProcessing;
                asm.save_state(state);
                true
            }
            AnchorStatus::NeedProcessing => {
                // TODO call llm 
                state.reply = "rispostone!! TODO ".into();
                state.status = AnchorStatus::NeedInjection;
                asm.save_state(state);
                true 
            }
            _ => {}
        }
    }

    fn pass_1_derive_begin_anchor(
        &self,
        asm: &utils::AnchorStateManager,
        parameters: &Parameters,
        arguments: &Arguments,
    ) -> Result<bool> {
        let state : DeriveState = asm.load_state();
        match state.status {
            AnchorStatus::JustCreated => {
                state.instruction_context_name = arguments.arguments.get(0)?; // TODO err?
                state.input_context_name = arguments.arguments.get(1); // TODO err?
                state.status = AnchorStatus::NeedProcessing;
                asm.save_state(state);
                true
            }
            AnchorStatus::NeedProcessing => {
                state.instruction_context = self.execute_loop(state.instruction_context_name);
                state.input_context = self.execute_loop(state.input_context_name);
                // call llm to derive                
                state.output = "rispostone!! TODO ".into();
                state.status = AnchorStatus::NeedInjection;
                asm.save_state(state);
                true 
            }
            _ => {}
        }
    }

    fn pass_1_inline_begin_anchor(
        &self,
        asm: &utils::AnchorStateManager,
        parameters: &Parameters,
        arguments: &Arguments,
    ) -> Result<bool> {
        let state : InlineState = asm.load_state();
        match state.status {
            AnchorStatus::JustCreated => {
                state.context_name = arguments.arguments.get(0)?; // TODO err?
                state.status = AnchorStatus::NeedProcessing;
                asm.save_state(state);
                Ok(true)
            }
            AnchorStatus::NeedProcessing => {
                let context_path = self.path_res.resolve_context(state.context_name);
                state.context = self.file_access.read_file(context_path)?;
                state.status = AnchorStatus::NeedInjection;
                asm.save_state(state);
                Ok(true)
            }
            _ => {}
        }
        Ok(false)
    }

    fn pass_2(&self, patches: &mut Patches, ast: &Document) {
        
        for item in ast.content {
            match item {
                Tag(tag) => {
                    if self.pass_2_tag(patches, tag)? {
                        return Ok(true);
                    }
                }
                Anchor(anchor) => {
                    if self.pass_2_anchor(patches, anchor)? {
                        return Ok(true);
                    }
                }
                _ => {}
            }
        }

        Ok(false)
    }

    fn pass_2_tag(&self, patches: &mut Patches, tag: &Tag) -> Result<bool> {
        match tag.command {
            CommandKind::Answer => self.pass_2_normal_tag<CommandKind::Answer, AnswerState>(patches, tag.parameters, tag.arguments, tag.range),            
            CommandKind::Derive => self.pass_2_normal_tag<CommandKind::Derive, DeriveState>(patches, tag.parameters, tag.arguments, tag.range),            
            CommandKind::Inline => self.pass_2_normal_tag<CommandKind::Inline, InlineState>(patches, tag.parameters, tag.arguments, tag.range),            
            _ => Ok(false),
        }
    }

    fn pass_2_normal_tag<S, T>(&self, patches: &mut Patches, parameters: &Parameters, arguments: &Arguments, range: &Range) -> Result<bool> {
        let (a0, a1) = Anchor::new_couple<S>(parameters, arguments);
        patches.add_patch(
            range,
            vec![a0.to_string(), a1.to_string()].join('\n'),
        );
        let ast = utils::AnchorStateManager::new(self.file_access, self.path_res, a0);
        ast.save_state(T::new());
    }
    
    fn pass_2_anchor(&self, patches: &mut Patches, anchor: &Anchor) -> Result<bool> {
        match anchor.kind {
            AnchorKind::Begin => self.pass_2_begin_anchor(patches, anchor),
            _ => Ok(false)
        }
    }

    fn pass_2_begin_anchor(&self, patches: &mut Patches, a0: &Anchor) -> Result<bool> {
        let a1 = // TODO find
        self.pass_2_anchors(patches, anchor, a1)
    }

    fn pass_2_anchors(&self, patches: &mut Patches, a0: &Anchor, a1: &Anchor) -> Result<bool> {
        let asm = utils::AnchorStateManager::new(self.file_access, self.path_res, a0);
        match a0.command => {
            CommandKind::Answer => self.pass_2_normal_begin_anchor<CommandKind::Answer, AnswerState>(patches, asm, a0.parameters, a0.arguments, a0.range, a1.range),    
            CommandKind::Derive => self.pass_2_normal_begin_anchor<CommandKind::Answer, AnswerState>(patches, asm, a0.parameters, a0.arguments, a0.range, a1.range),
            CommandKind::Inline => self.pass_2_normal_begin_anchor<CommandKind::Answer, AnswerState>(patches, asm, a0.parameters, a0.arguments, a0.range, a1.range),            
            _ => Ok(false),
        }                
    }

    fn pass_2_normal_begin_anchor<S, T>(
        &self,
        patches: &mut Patches,
        asm: &utils::AnchorStateManager,
        parameters: &Parameters,
        arguments: &Arguments,
        range_begin: &Range,
        range_end: &Range,
    ) -> Result<bool> {
        let mut state : AnswerState = asm.load_state();
        match state.status {
            AnchorStatus::NeedInjection => {
                let range = Range {
                    begin: range_begin.end,
                    end: range_end.begin,
                };
                patches.add_patch(range, state.output());
                state.status = AnchorStatus::Completed;
                asm.save_state(state);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
