use crate::ast2::{parse_document, Anchor, AnchorKind, CommandKind, Document, Range, Tag};
use anyhow::Result;

struct SystemContent {
    text: String,
}

struct UserContent {
    text: String,
}

struct AgentContent {
    author: String,
    text: String,
}

enum ContentItem {
    System(SystemContent),
    User(UserContent),
    Agent(AgentContent),
}

impl ContentItem {
    fn user(text: &str) -> Self {
        ContentItem::User(UserContent { text: text.into() })
    }
    fn system(text: &str) -> Self {
        ContentItem::System(SystemContent { text: text.into() })
    }
    fn agent(author: &str, text: &str) -> Self {
        ContentItem::Agent(AgentContent {
            author: author.into(),
            text: text.into(),
        })
    }
}

enum AnchorStatus {
    /// Just created, empty without any content nor state gathered
    JustCreated,
    /// Gathered info, need to process them
    NeedProcessing,
    /// Information has been processed, need to be injected in document
    NeedInjection,
    /// Completed, no further processing needed
    Completed,
}

pub fn execute(project: &Project, context_name: &str, commit: &Commit) {}

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

type State = serde_json::Value;

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

struct Executor {
    project: &Project,
    visited: HashSet<String>,
    prelude: Vec<ContentItem>,
    context: Vec<ContentItem>,
}

impl Executor {
    fn execute_loop(&self, context_name: &str) {
        let context_path = self.project.resolve_context(context_name);

        if self.visited.contains(context_path) {
            return;
        }

        while execute_step(context_path) {}
    }
    fn execute_step(&self, context_path: &Path) -> Result<bool> {
        // Read file, parse it, execute slow things that do not modify context
        let context = std::fs::read_to_string(context_path)?;
        let ast = crate::ast2::parse_document(context)?;
        let want_next_step_1 = pass_1(ast);

        // Lock file, re-read it (could be edited outside), parse it, execute fast things that may modify context and save it
        // TODO lock
        let context = std::fs::read_to_string(context_path)?;
        let ast = crate::ast2::parse_document(context)?;
        let (want_next_step_2, patches) = pass_2(context, ast);
        if patches.apply(context) {
            // save file
        }
        // TODO unlock

        Ok(want_next_step_1 | want_next_step_2)
    }

    fn pass_1(&self, ast: &Document) -> Result<bool> {
        let mut want_next_step = false;

        for item in ast.content {
            match item {
                Text(text) => self.pass_1_text(text)?,
                Tag(tag) => want_next_step |= self.pass_1_tag(tag)?,
                Anchor(anchor) => want_next_step |= self.pass_1_anchor(tag)?,
            }
        }

        Ok(want_next_step)
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
        let state = anchor.load_state(self.project)?;
        match (
            anchor.command,
            anchor.kind,
            anchor.parameters,
            anchor.arguments,
        ) {
            (CommandKind::Answer, AnchorKind::Begin, parameters, arguments) => {
                want_next_step |= self.pass_1_answer_begin_anchor(state, parameters, arguments)
            } // passa commit?
            (CommandKind::Derive, AnchorKind::Begin, parameters, arguments) => {
                want_next_step |= self.pass_1_derive_begin_anchor(state, parameters, arguments)
            } // passa commit?
            (CommandKind::Inline, AnchorKind::Begin, parameters, arguments) => {
                want_next_step |= self.pass_1_inline_begin_anchor(state, parameters, arguments)
            } // passa commit?
            (CommandKind::Repeat, AnchorKind::Begin, parameters, arguments) => {
                want_next_step |= self.pass_1_repeat_begin_anchor(state, parameters, arguments)
            } // passa commit?
            (CommandKind::Summarize, AnchorKind::Begin, parameters, arguments) => {
                want_next_step |= self.pass_1_summarize_begin_anchor(state, parameters, arguments)
            } // passa commit?
            _ => {}
        }
    }

    fn pass_1_answer_begin_anchor(
        &self,
        state: &State,
        parameters: &Parameters,
        arguments: &Arguments,
    ) -> Result<bool> {
        unimplemented!()
    }

    fn pass_1_derive_begin_anchor(
        &self,
        state: &State,
        parameters: &Parameters,
        arguments: &Arguments,
    ) -> Result<bool> {
        unimplemented!()
    }

    fn pass_1_inline_begin_anchor(
        &self,
        state: &State,
        parameters: &Parameters,
        arguments: &Arguments,
    ) -> Result<bool> {
        unimplemented!()
    }

    fn pass_1_repeat_begin_anchor(
        &self,
        state: &State,
        parameters: &Parameters,
        arguments: &Arguments,
    ) -> Result<bool> {
        unimplemented!()
    }

    fn pass_1_summarize_begin_anchor(
        &self,
        state: &State,
        parameters: &Parameters,
        arguments: &Arguments,
    ) -> Result<bool> {
        unimplemented!()
    }

    fn pass_2() {}
}
