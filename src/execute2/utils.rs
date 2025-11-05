use std::collections::HashMap;
use uuid::Uuid;

use crate::ast2::*;

pub struct AnchorIndex {
    begin: HashMap<Uuid, usize>, // uid -> content index
    end: HashMap<Uuid, usize>,   // uid -> content index
}

impl AnchorIndex {
    pub fn new(content: &[Content]) -> Self {
        let mut begin = HashMap::new();
        let mut end = HashMap::new();

        for (i, line) in content.iter().enumerate() {
            match line {
                Content::Anchor(anchor) => match anchor.kind {
                    AnchorKind::Begin => {
                        let _ = begin.insert(anchor.uuid, i);
                    }
                    AnchorKind::End => {
                        let _ = end.insert(anchor.uuid, i);
                    }
                },
                _ => {}
            }
        }

        Self { begin, end }
    }

    pub fn get_begin(&self, uid: &Uuid) -> Option<usize> {
        self.begin.get(uid).copied()
    }

    pub fn get_end(&self, uid: &Uuid) -> Option<usize> {
        self.end.get(uid).copied()
    }
}
