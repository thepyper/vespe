use std::collections::HashMap;
use std::sync::Arc;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use crate::tool::Tool;
use crate::agent_protocol::AgentProtocol;
use crate::error::ProjectError;

/// Un trait generico per un registro che può memorizzare implementazioni di un tipo `T`.
pub trait Registry<T> {
    /// Restituisce un riferimento immutabile alla mappa interna del registro.
    fn get_map(&self) -> &HashMap<String, T>;
    /// Restituisce un riferimento mutabile alla mappa interna del registro.
    fn get_map_mut(&mut self) -> &mut HashMap<String, T>;

    /// Registra un'implementazione di `T` con un dato nome.
    fn register(&mut self, name: String, item: T) -> Result<(), ProjectError> {
        let map = self.get_map_mut();
        if map.contains_key(&name) {
            return Err(ProjectError::RegistryError(format!("Item with name '{}' already registered.", name)));
        }
        map.insert(name, item);
        Ok(())
    }

    /// Recupera un'implementazione di `T` tramite il suo nome.
    fn get(&self, name: &str) -> Option<&T> {
        self.get_map().get(name)
    }

    /// Recupera un'implementazione mutabile di `T` tramite il suo nome.
    fn get_mut(&mut self, name: &str) -> Option<&mut T> {
        self.get_map_mut().get_mut(name)
    }

    /// Restituisce un iteratore sui nomi e gli elementi registrati.
    fn iter(&self) -> Box<dyn Iterator<Item = (&String, &T)> + '_> {
        Box::new(self.get_map().iter())
    }
}

// --- Implementazione di ToolRegistry ---

pub struct ToolRegistryInner {
    tools: HashMap<String, Tool>,
}

impl ToolRegistryInner {
    fn new() -> Self {
        ToolRegistryInner { tools: HashMap::new() }
    }
}

impl Registry<Tool> for ToolRegistryInner {
    fn get_map(&self) -> &HashMap<String, Tool> {
        &self.tools
    }

    fn get_map_mut(&mut self) -> &mut HashMap<String, Tool> {
        &mut self.tools
    }
}

/// Singleton per il registro degli strumenti.
pub static TOOL_REGISTRY: Lazy<Arc<Mutex<ToolRegistryInner>>> = Lazy::new(|| {
    Arc::new(Mutex::new(ToolRegistryInner::new()))
});

// --- Implementazione di AgentProtocolRegistry ---

pub struct AgentProtocolRegistryInner {
    protocols: HashMap<String, Box<dyn AgentProtocol>>,
}

impl AgentProtocolRegistryInner {
    fn new() -> Self {
        AgentProtocolRegistryInner { protocols: HashMap::new() }
    }
}

impl Registry<Box<dyn AgentProtocol>> for AgentProtocolRegistryInner {
    fn get_map(&self) -> &HashMap<String, Box<dyn AgentProtocol>> {
        &self.protocols
    }

    fn get_map_mut(&mut self) -> &mut HashMap<String, Box<dyn AgentProtocol>> {
        &mut self.protocols
    }
}

/// Singleton per il registro dei protocolli agente.
pub static AGENT_PROTOCOL_REGISTRY: Lazy<Arc<Mutex<AgentProtocolRegistryInner>>> = Lazy::new(|| {
    Arc::new(Mutex::new(AgentProtocolRegistryInner::new()))
});
