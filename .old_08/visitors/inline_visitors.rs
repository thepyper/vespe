use crate::ast::types::{AnchorData, AnchorDataValue, AnchorKind, Context, Line, LineKind, Snippet};
use crate::ast::visitor::Visitor;
use uuid::Uuid;

pub struct InlineBeginDecorator;

impl Visitor for InlineBeginDecorator {
    fn visit_context_lines(&mut self, context: &mut Context) {
        for line in &mut context.lines {
            self.process_line(line);
        }
    }

    fn visit_snippet_lines(&mut self, snippet: &mut Snippet) {
        for line in &mut snippet.lines {
            self.process_line(line);
        }
    }
}

impl InlineBeginDecorator {
    fn process_line(&mut self, line: &mut Line) {
        if let LineKind::Inline { .. } = &line.kind {
            let has_begin_anchor = line.anchor.as_ref().map_or(false, |anchor_data| {
                anchor_data.kind == AnchorKind::Inline && anchor_data.data == Some(AnchorDataValue::Begin)
            });

            if !has_begin_anchor {
                let new_uuid = Uuid::new_v4();
                let new_anchor = AnchorData {
                    kind: AnchorKind::Inline,
                    uid: new_uuid,
                    data: Some(AnchorDataValue::Begin),
                };
                let anchor_string = new_anchor.to_string(); // Use the Display impl for AnchorData
                line.text = format!("{} {}", anchor_string, line.text);
                line.anchor = Some(new_anchor);
            }
        }
    }
}
