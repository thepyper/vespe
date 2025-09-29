use std::collections::HashMap;
use std::sync::Arc;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use crate::tool::Tool;
use crate::agent_protocol::AgentProtocol;
use crate::error::ProjectError;

/// Un trait generico per un registro che può memorizzare implementazioni di un tipo `T`.
pub trait Registry<T> {
    /// Registra un'implementazione di `T` con un dato nome.
    fn register(&mut self, name: String, item: T) -> Result<(), ProjectError>;
    /// Recupera un'implementazione di `T` tramite il suo nome.
    fn get(&self, name: &str) -> Option<&T>;
    /// Recupera un'implementazione mutabile di `T` tramite il suo nome.
    fn get_mut(&mut self, name: &str) -> Option<&mut T>;
    /// Restituisce un iteratore sui nomi e gli elementi registrati.
    fn iter(&self) -> Box<dyn Iterator<Item = (&String, &T)> + '_>;
}

// --- Implementazione di ToolRegistry ---

pub struct ToolRegistryInner {
    tools: HashMap<String, Tool>,
}

impl Registry<Tool> for ToolRegistryInner {
    fn register(&mut self, name: String, tool: Tool) -> Result<(), ProjectError> {
        if self.tools.contains_key(&name) {
            return Err(ProjectError::RegistryError(format!("Tool with name '{}' already registered.", name)));
        }
        self.tools.insert(name, tool);
        Ok(())
    }

    fn get(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }

    fn get_mut(&mut self, name: &str) -> Option<&mut Tool> {
        self.tools.get_mut(name)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = (&String, &Tool)> + '_> {
        Box::new(self.tools.iter())
    }
}

/// Singleton per il registro degli strumenti.
pub static TOOL_REGISTRY: Lazy<Arc<Mutex<ToolRegistryInner>>> = Lazy::new(|| {
    Arc::new(Mutex::new(ToolRegistryInner { tools: HashMap::new() }))
});

// --- Implementazione di AgentProtocolRegistry ---

pub struct AgentProtocolRegistryInner {
    protocols: HashMap<String, Box<dyn AgentProtocol>>,
}

impl Registry<Box<dyn AgentProtocol>> for AgentProtocolRegistryInner {
    fn register(&mut self, name: String, protocol: Box<dyn AgentProtocol>) -> Result<(), ProjectError> {
        if self.protocols.contains_key(&name) {
            return Err(ProjectError::RegistryError(format!("AgentProtocol with name '{}' already registered.", name)));
        }
        self.protocols.insert(name, protocol);
        Ok(())
    }

    fn get(&self, name: &str) -> Option<&Box<dyn AgentProtocol>> {
        self.protocols.get(name)
    }

    fn get_mut(&mut self, name: &str) -> Option<&mut Box<dyn AgentProtocol>> {
        self.protocols.get_mut(name)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = (&String, &Box<dyn AgentProtocol>)> + '_> {
        Box::new(self.protocols.iter())
    }
}

/// Singleton per il registro dei protocolli agente.
pub static AGENT_PROTOCOL_REGISTRY: Lazy<Arc<Mutex<AgentProtocolRegistryInner>>> = Lazy::new(|| {
    Arc::new(Mutex::new(AgentProtocolRegistryInner { protocols: HashMap::new() }))
});
