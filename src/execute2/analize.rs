
use super::{tag_answer::AnswerState, tag_inline::InlineState, tag_task::TaskState, Result};
use crate::ast2::{parse_document, Anchor, CommandKind, Content};
use crate::file::FileAccessor;
use crate::path::PathResolver;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Enum that aggregates the different types of state specific for each anchor.
#[derive(Debug)]
pub enum AnchorState {
    Answer(AnswerState),
    Inline(InlineState),
    Task(TaskState),
}

/// Contains the complete analysis of a single anchor.
#[derive(Debug)]
pub struct AnchorAnalysis {
    pub anchor: Anchor,
    pub state: AnchorState,
}

/// Final result of the analysis of a context.
#[derive(Debug)]
pub struct ContextAnalysis {
    pub anchors: HashMap<Uuid, AnchorAnalysis>,
}

pub fn analyze_context(
    file_access: Arc<dyn FileAccessor>,
    path_res: Arc<dyn PathResolver>,
    context_name: &str,
) -> Result<ContextAnalysis> {
    let analyzer = Analyzer {
        file_access,
        path_res,
    };
    analyzer.run(context_name)
}

struct Analyzer {
    file_access: Arc<dyn FileAccessor>,
    path_res: Arc<dyn PathResolver>,
}

impl Analyzer {
    fn run(&self, context_name: &str) -> Result<ContextAnalysis> {
        let path = self.path_res.resolve_input_file(context_name)?;
        let content = self.file_access.read_file(&path)?;
        let doc = parse_document(&content)?;

        let mut analysis = ContextAnalysis {
            anchors: HashMap::new(),
        };

        for node in doc.content {
            if let Content::Anchor(anchor) = node {
                if let Some(state) = self.extract_anchor_state(&anchor)? {
                    let anchor_analysis = AnchorAnalysis {
                        anchor: anchor.clone(),
                        state,
                    };
                    analysis.anchors.insert(anchor.uuid, anchor_analysis);
                }
            }
        }

        Ok(analysis)
    }

    fn extract_anchor_state(&self, anchor: &Anchor) -> Result<Option<AnchorState>> {
        match anchor.command {
            CommandKind::Answer => {
                let state = load_state::<AnswerState>(anchor)?;
                Ok(Some(AnchorState::Answer(state)))
            }
            CommandKind::Inline => {
                let state = load_state::<InlineState>(anchor)?;
                Ok(Some(AnchorState::Inline(state)))
            }
            CommandKind::Task => {
                let state = load_state::<TaskState>(anchor)?;
                Ok(Some(AnchorState::Task(state)))
            }
            _ => Ok(None),
        }
    }
}

fn load_state<T: for<'de> Deserialize<'de> + Default>(anchor: &Anchor) -> Result<T> {
    if let Some(state_json) = anchor.parameters.get("state") {
        let state = serde_json::from_value(state_json.into())?;
        Ok(state)
    } else {
        Ok(T::default())
    }
}
