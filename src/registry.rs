use std::collections::HashMap;
use std::sync::Arc;
use once_cell::sync::Lazy;

use crate::tool::Tool;
use crate::agent_protocol::AgentProtocol;
use crate::agent_protocol_mcp::McpAgentProtocol;

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

pub struct AgentProtocolRegistryInner {
    protocols: HashMap<String, Arc<Box<dyn AgentProtocol + Send + Sync>>>,
}

impl Registry<Arc<Box<dyn AgentProtocol + Send + Sync>>> for AgentProtocolRegistryInner {
    fn get_map(&self) -> &HashMap<String, Arc<Box<dyn AgentProtocol + Send + Sync>>> {
        &self.protocols
    }
}

pub static AGENT_PROTOCOL_REGISTRY: Lazy<AgentProtocolRegistryInner> = Lazy::new(|| {
    let mut protocols = HashMap::new();
    protocols.insert("mcp".to_string(), Arc::new(Box::new(McpAgentProtocol) as Box<dyn AgentProtocol + Send + Sync>));
    AgentProtocolRegistryInner { protocols }
});
