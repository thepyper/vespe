use crate::semantic;
use std::collections::HashMap;
use std::collections::BTreeMap;
use uuid::Uuid;
use anyhow::Result;
use std::path::PathBuf;

use crate::ast2::Range;
use crate::file;
use crate::path;

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

pub struct AnchorStateManager<'a> {
    file_access: &'a dyn file::FileAccessor, 
    path_res: &'a dyn path::PathResolver,
    command: crate::ast2::CommandKind,
    uuid: Uuid,
}

impl<'a> AnchorStateManager<'a> {
    pub fn new(file_access: &'a dyn file::FileAccessor, path_res: &'a dyn path::PathResolver, anchor: &crate::ast2::Anchor) -> Self {
        AnchorStateManager {
            file_access,
            path_res,
            command: anchor.command,
            uuid: anchor.uuid,
        }
    }
    fn get_state_path(&self) -> Result<PathBuf> {
        let meta_path = self.path_res.resolve_metadata(&self.command.to_string(), &self.uuid)?;
        let state_path = meta_path.join("state.json");
        Ok(state_path)
    }
    fn load_state<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        let state_path = self.get_state_path()?;
        let state = self.file_access.read_file(&state_path)?;
        let state: T = serde_json::from_str(&state)?;
        Ok(state)
    }
    fn save_state<T: serde::Serialize>(&self, state: &T, comment: Option<&str>) -> Result<()> {
        let state_path = self.get_state_path()?;
        let state_str = serde_json::to_string_pretty(state)?;
        self.file_access.write_file(&state_path, &state_str, comment)?;
        Ok(())
    }
}

pub struct Patches<'a> {
    document: &'a str,
    patches: BTreeMap<Range, String>,
}

impl<'a> Patches<'a> {
    pub fn new(document: &'a str) -> Self {
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
        Ok(self.document.to_string())
    }
}


