use crate::semantic;
use std::collections::HashMap;
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::ast2::Range;

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

pub struct AnchorStateManager {
    file_access: &file::FileAccessor, 
    path_res: &path::PathResolver,
    command: crate::ast2::CommandKind,
    uuid: Uuid,
}

impl AnchorStateManager {
    pub fn new(file_access: &file::FileAccessor, path_res: &path::PathResolver, anchor: crate::ast2::Anchor&) -> Self {
        AnchorStateManager {
            file_access,
            path_res,
            command: anchor.command,
            uuid: anchor.uuid,
        }
    }
    fn get_state_path(&self) -> PathBuf {
        let meta_path = self.path_res.resolve_metadata(self.command.to_string(), self.uuid);
        let state_path = meta_path.join("state.json");
        state_path
    }
    fn load_state<T>(&self) -> Result<T> {
        let state_path = self.get_state_path();
        let state = self.file_access.read_file(state_path)?;
        let state: T = serde_json::from_str(state)?;
        Ok(state)
    }
    fn save_state<T>(&self, state: &T, comment: Option<&str>) -> Result<()> {
        let state_path = self.get_state_path();
        self.file_access.write_file(state_path, state, comment)?;
        Ok(())
    }
}

pub struct Patches {
    document: &str,
    patches: BTreeMap<Range, String>,
}

impl Patches {
    pub fn new(document: &str) -> Self {
        Patches {
            document,
            patches: BTreeMap::new(),
        }
    }
    pub fn add_patch(&mut self, range: &Range, replace: &str) {
        self.patches.insert(range.clone(), replace.to_string());
    }
    pub fn is_empty(&self) -> bool {
        self.patches.is_empty()
    }
    pub fn apply_patches(&self) -> Result<String> {
        // TODO apply
    }
}


