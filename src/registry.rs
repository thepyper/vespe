use std::collections::HashMap;
use std::sync::Arc;
use once_cell::sync::Lazy;

use crate::tool::Tool;

pub trait Registry<T> {
    fn get_map(&self) -> &HashMap<String, T>;

    fn get(&self, name: &str) -> Option<&T> {
        self.get_map().get(name)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = (&String, &T)> + '_> {
        Box::new(self.get_map().iter())
    }
}

pub struct ToolRegistryInner {
    tools: HashMap<String, Tool>,
}

impl Registry<Tool> for ToolRegistryInner {
    fn get_map(&self) -> &HashMap<String, Tool> {
        &self.tools
    }
}

pub static TOOL_REGISTRY: Lazy<ToolRegistryInner> = Lazy::new(|| {
    let tools = HashMap::new();
    ToolRegistryInner { tools }
});
