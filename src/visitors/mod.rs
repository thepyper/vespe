


struct RunnerVisitor {

    composed_context: String,
    reconstructed_contexts: HashMap<String, String>,
    modified_contexts: HashSet<String>,

    // Internal state 
    context_stack: Vec<PathBuf>,
};

impl crate::ast::Visitor for RunnerVisitor {

    fn pre_visit_context(&mut self, context: &mut Context) -> Result<(), Box<dyn std::error::Error>> {
        self.context_stack.push(context.path.clone());
        Ok(())
    }

    fn post_visit_context(&mut self, _context: &mut Context) -> Result<(), Box<dyn std::error::Error>> {
        self.context_stack.pop();
        Ok(())
    }

    fn pre_visit_snippet(&mut self, _snippet: &mut Snippet) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn post_visit_snippet(&mut self, _snippet: &mut Snippet) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn visit_line(&mut self, line: &mut Line) -> Result<(), Box<dyn std::error::Error>> {
        match line.kind {
            crate::ast::LineKind::Include { ref context, ref parameters } => {
                // Include is handled by traversing it, nothing more to do 
            },
            crate::ast::LineKind::Inline { ref snippet, ref parameters } => {
                // Inline is handled by traversing it, nothing more to do 
            },
            crate::ast::LineKind::Answer { ref parameters } => {
                // Answer line, we need to call the agent
                
            },
            crate::ast::LineKind::Summary { ref context, ref parameters } => {
                // Summary is handled by traversing it, nothing more to do

            },
            crate::ast::LineKind::Text => {
                // Just a text line, append it to the composed context
                self.composed_context.push_str(&line.text);
                self.composed_context.push('\n');
            },
        }
        Ok(())
    }
}