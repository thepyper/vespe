use std::collections::HashMap;
use std::sync::Arc;
use once_cell::sync::Lazy;

use crate::tool::Tool;
use crate::agent_protocol::AgentProtocol;
use crate::error::ProjectError;

/// Un trait generico per un registro che può memorizzare implementazioni di un tipo `T`.
pub trait Registry<T> {
    /// Restituisce un riferimento immutabile alla mappa interna del registro.
    fn get_map(&self) -> &HashMap<String, T>;

    /// Recupera un'implementazione di `T` tramite il suo nome.
    fn get(&self, name: &str) -> Option<&T> {
        self.get_map().get(name)
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

impl Registry<Tool> for ToolRegistryInner {
    fn get_map(&self) -> &HashMap<String, Tool> {
        &self.tools
    }
}

/// Singleton per il registro degli strumenti.
pub static TOOL_REGISTRY: Lazy<ToolRegistryInner> = Lazy::new(|| {
    // Inizializzazione del registro degli strumenti. Tutti gli strumenti devono essere registrati qui.
    let mut tools = HashMap::new();
    // Esempio di registrazione di uno strumento fittizio:
    // tools.insert("dummy_tool_1".to_string(), Tool::create("dummy_tool_1".to_string(), "A dummy tool for testing.".to_string(), serde_json::json!({}), Path::new("/")).unwrap());
    ToolRegistryInner { tools }
});

// --- Implementazione di AgentProtocolRegistry ---

pub struct AgentProtocolRegistryInner {
    protocols: HashMap<String, Arc<Box<dyn AgentProtocol + Send + Sync>>>,
}

impl Registry<Arc<Box<dyn AgentProtocol + Send + Sync>>> for AgentProtocolRegistryInner {
    fn get_map(&self) -> &HashMap<String, Arc<Box<dyn AgentProtocol + Send + Sync>>> {
        &self.protocols
    }
}

/// Singleton per il registro dei protocolli agente.
pub static AGENT_PROTOCOL_REGISTRY: Lazy<AgentProtocolRegistryInner> = Lazy::new(|| {
    // Inizializzazione del registro dei protocolli agente. Tutti i protocolli devono essere registrati qui.
    let mut protocols = HashMap::new();
    // Esempio di registrazione di un protocollo fittizio:
    // protocols.insert("default_protocol".to_string(), Arc::new(Box::new(DefaultAgentProtocol::new())));
    AgentProtocolRegistryInner { protocols }
});
