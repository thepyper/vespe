mod state;
mod content;

use state::*;
use content::*;

use crate::ast2::{parse_document, Anchor, AnchorKind, CommandKind, Document, Range, Tag};
use anyhow::Result;


pub fn execute(file_access: &file::FileAccessor, path_res: &path::PathResolver, context_name: &str, commit: &Commit) {

}

/*
impl Tag {
    pub fn get_argument_as_context(&self, i: usize, project: &Project) -> Result<PathBuf> {
        let context_name = self.arguments.arguments.get(i).ok_or_else(/* TODO errore */).value;
        let context_path = project.resolve_context(context_name);
        Ok(context_path)
    }
    pub fn validate_argument_as_context(&self, i: usize, project: &Project) -> Result<PathBuf> {
        let context_path = self.get_argument_as_context(i, project);
        match std::fs::exists(context_path) {
            true => Ok(context_path),
            false => Err(_), // TODO inesistente
        }
    }
}
*/

type State = serde_json::Value;

/*
impl Anchor {
    fn state_file_name(&self, project: &Project) -> Result<PathBuf> {
        let meta_path = project.resolve_metadata(self.kind, self.uuid)?;
        let state_file = meta_path.join("state.json")?;
        Ok(state_file)
    }
    pub fn load_state(&self, project: &Project) -> Result<State> {
        let state_file = self.state_file_name(project)?;
        // TODO load json
    }
    pub fn save_state(&self, project: &Project, state: &State) {
        let state_file = self.state_file_name(project)?;
        // TODO save json
    }
}
*/

struct Executor {
    file_access: &file::FileAccessor, 
    path_res: &path::PathResolver,
    visited: HashSet<String>,
    prelude: Vec<ContentItem>,
    context: Vec<ContentItem>,
}

impl Executor {
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
            anchor.parameters,
            anchor.arguments,
        ) {
            (CommandKind::Answer, AnchorKind::Begin, parameters, arguments) => {
                want_next_step |= self.pass_1_answer_begin_anchor(asm, parameters, arguments)
            } // passa commit?
            (CommandKind::Derive, AnchorKind::Begin, parameters, arguments) => {
                want_next_step |= self.pass_1_derive_begin_anchor(asm, parameters, arguments)
            } // passa commit?
            (CommandKind::Inline, AnchorKind::Begin, parameters, arguments) => {
                want_next_step |= self.pass_1_inline_begin_anchor(asm, parameters, arguments)
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
                true
            }
            AnchorStatus::NeedProcessing => {
                let context_path = self.path_res.resolve_context(state.context_name);
                state.context = self.file_access.read_file(context_path)?;
                state.status = AnchorStatus::NeedInjection;
                asm.save_state(state);
                true 
            }
            _ => {}
        }
    }

    fn pass_2(&self, context: &str, ast: &Document) {
        
        for item in ast.content {
            match item {
                Tag(tag) => {
                    if self.pass_2_tag(tag)? {
                        return Ok(true);
                    }
                }
                Anchor(anchor) => {
                    if self.pass_2_anchor(tag)? {
                        return Ok(true);
                    }
                }
                _ => {}
            }
        }

        Ok(false)
    }

    fn pass_2_tag(&self, tag: &Tag) -> Result<bool> {
        match tag.command {
            CommandKind::  
            _ => Ok(false),
        }
    }

    fn pass_2_anchor(&self, anchor: &Anchor) -> Result<bool> {
        let asm = utils::AnchorStateManager::new(self.file_access, self.path_res, anchor);
        match (
            anchor.command,
            anchor.kind,
            anchor.parameters,
            anchor.arguments,
        ) {
            (CommandKind::Answer, AnchorKind::Begin, parameters, arguments) => {
                want_next_step |= self.pass_2_answer_begin_anchor(asm, parameters, arguments)
            } // passa commit?
            (CommandKind::Derive, AnchorKind::Begin, parameters, arguments) => {
                want_next_step |= self.pass_2_derive_begin_anchor(asm, parameters, arguments)
            } // passa commit?
            (CommandKind::Inline, AnchorKind::Begin, parameters, arguments) => {
                want_next_step |= self.pass_2_inline_begin_anchor(asm, parameters, arguments)
            } // passa commit?          
            _ => {}
        }
    }

    fn pass_2_answer_begin_anchor(
        &self,
        asm: &utils::AnchorStateManager,
        parameters: &Parameters,
        arguments: &Arguments,
    ) -> Result<bool> {
        let mut state : AnswerState = asm.load_state();
        match state.status {
            AnchorStatus::
            _ => {}
        }
    }

    fn pass_2_derive_begin_anchor(
        &self,
        asm: &utils::AnchorStateManager,
        parameters: &Parameters,
        arguments: &Arguments,
    ) -> Result<bool> {
        let state : DeriveState = asm.load_state();
        match state.status {
            AnchorStatus::
            _ => {}
        }
    }

    fn pass_2_inline_begin_anchor(
        &self,
        asm: &utils::AnchorStateManager,
        parameters: &Parameters,
        arguments: &Arguments,
    ) -> Result<bool> {
        let state : InlineState = asm.load_state();
        match state.status {
            AnchorStatus::
            _ => {}
        }
    }

}
