use crate::semantic;
use std::collections::HashMap;
use uuid::Uuid;

pub struct AnchorIndex {
    begin: HashMap<Uuid, usize>, // uid -> line index
    end: HashMap<Uuid, usize>,   // uid -> line index
}

impl AnchorIndex {
    pub fn new(lines: &[semantic::Line]) -> Self {
        let mut begin = HashMap::new();
        let mut end = HashMap::new();

        for (i, line) in lines.iter().enumerate() {
            if line.is_begin_anchor() {
                begin.insert(line.get_uid(), i);
            } else if line.is_end_anchor() {
                end.insert(line.get_uid(), i);
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