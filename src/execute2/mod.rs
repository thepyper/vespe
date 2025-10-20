use anyhow::Result;
use crate::ast2::{Document, Range, parse_document};

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

pub fn execute(project: &Project, context_name: &str, commit: &Commit)
{

}



struct Executor {
    project: &Project,
    visited: HashSet<String>,
    prelude: Vec<ContentItem>,
    context: Vec<ContentItem>,
}

struct Marker {
    document: &str,
}

impl Marker {
    fn new(document: &str) -> Self {
        Marker {
            document,
        }
    }
    fn mark(&self, range: &Range) -> Result<String> {
        if !range.is_valid() {
            // TODO error
        } else if range.is_null() {
            return String::new();
        } else {
            return self.document[range.begin..range.end]; // TODO inclusiva o no?
        }
    }
}

impl Executor {
    fn execute_loop(&self, context_name: &str) {

        let context_path = self.project.resolve_context(context_name);

        if self.visited.contains(context_path)
            return;

        while execute_step(context_path) {

        }
    }
    fn execute_step(&self, context_path: &Path) -> Result<bool> {

        // Read file, parse it, execute slow things that do not modify context
        let context = std::fs::read_to_string(context_path)?;
        let ast = crate::ast2::parse_document(context)?;
        let want_next_step_1 = do_things_that_do_not_modify_context(context, ast);
        
        // Lock file, re-read it (could be edited outside), parse it, execute fast things that may modify context and save it
        // TODO lock
        let context = std::fs::read_to_string(context_path)?;
        let ast = crate::ast2::parse_document(context)?;
        let (want_next_step_2, patches) = do_fast_things_that_modify_context(context, ast);
        if patches.apply(context) {
            // save file
        }
        // TODO unlock

        Ok(want_next_step_1 | want_next_step_2)
    }

    fn do_things_that_do_not_modify_context(
        &self,
        document: &str,
        ast: &Document,
    ) -> Result<bool> {

        let mk = Marker::new(document);
        
        for item in ast.content {
            match item {
                Text(text) => {
                    self.context.push()
                }
            }
        }
    }
    fn do_fast_things_that_modify_context() {

    }
}
