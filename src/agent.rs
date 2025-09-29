use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::memory::{Memory, Message, MessageContent};
use crate::error::ProjectError;
use crate::utils::{generate_uid, get_entity_path, read_json_file, write_json_file, write_file_content, read_file_content};
use crate::registry::{AGENT_PROTOCOL_REGISTRY, TOOL_REGISTRY, Registry};
use crate::agent_protocol::{AgentProtocol, ToolConfig};
use crate::llm_client::{create_llm_client, LLMClient};

// Default protocol name for agents
const DEFAULT_AGENT_PROTOCOL_NAME: &str = "default_protocol";

fn default_protocol_name() -> String {
    DEFAULT_AGENT_PROTOCOL_NAME.to_string()
}

// 1. METADATA COMUNI: Dati condivisi da tutti gli agenti.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentMetadata {
    pub uid: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub parent_uid: Option<String>,
    #[serde(default = "default_protocol_name")]
    pub protocol_name: String,
}

// 2. DETTAGLI SPECIFICI: L'enum che modella la differenza chiave.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AgentDetails {
    AI(AIConfig),
    Human(HumanConfig),
}

// 3. CONFIGURAZIONE AI: Campi specifici per l'AI.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AIConfig {
    pub role: String,
    pub llm_provider: LLMProviderConfig,
    pub allowed_tools: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum LLMProviderConfig {
    Ollama { model: String, endpoint: String },
    OpenAI { model: String, api_key_env: String },
}

// 4. CONFIGURAZIONE HUMAN: Campi specifici per l'umano.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct HumanConfig { /* ...email, permissions, etc. */ }

// 5. STATO DINAMICO DELL'AGENTE (semplificato)
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AgentState {
    pub last_seen_at: DateTime<Utc>,
}

// L'AGENTE COMPLETO: Oggetto costruito in memoria.
#[derive(Debug, Clone)]
pub struct Agent {
    pub metadata: AgentMetadata,
    pub details: AgentDetails,
    pub state: AgentState,
    pub memory: Memory,
    pub system_prompt: Option<String>,
}

impl Agent {
    pub fn create_ai(
        project_root: &Path,
        name: String,
        config: AIConfig,
        system_prompt: Option<String>,
        protocol_name: Option<String>,
    ) -> Result<Agent, ProjectError> {
        let uid = generate_uid("agt")?;
        let agents_base_path = project_root.join(".vespe").join("agents");
        let agent_path = get_entity_path(&agents_base_path, &uid)?;

        std::fs::create_dir_all(&agent_path).map_err(|e| ProjectError::Io(e))?;
        std::fs::create_dir_all(agent_path.join("memory")).map_err(|e| ProjectError::Io(e))?;

        if let Some(prompt) = system_prompt {
            write_file_content(&agent_path.join("system_prompt.hbs"), &prompt)?;
        }

        let now = Utc::now();

        let metadata = AgentMetadata {
            uid: uid.clone(),
            name: name.clone(),
            created_at: now,
            parent_uid: None,
            protocol_name: protocol_name.unwrap_or_else(default_protocol_name),
        };
        write_json_file(&agent_path.join("metadata.json"), &metadata)?;

        let details = AgentDetails::AI(config);
        write_json_file(&agent_path.join("details.json"), &details)?;

        let state = AgentState::default();
        write_json_file(&agent_path.join("state.json"), &state)?;

        Agent::load(project_root, &uid)
    }

    pub fn create_human(
        project_root: &Path,
        name: String,
        config: HumanConfig,
        protocol_name: Option<String>,
    ) -> Result<Agent, ProjectError> {
        let uid = generate_uid("usr")?;
        let agents_base_path = project_root.join(".vespe").join("agents");
        let agent_path = get_entity_path(&agents_base_path, &uid)?;

        std::fs::create_dir_all(&agent_path).map_err(|e| ProjectError::Io(e))?;
        std::fs::create_dir_all(agent_path.join("memory")).map_err(|e| ProjectError::Io(e))?;

        let now = Utc::now();

        let metadata = AgentMetadata {
            uid: uid.clone(),
            name: name.clone(),
            created_at: now,
            parent_uid: None,
            protocol_name: protocol_name.unwrap_or_else(default_protocol_name),
        };
        write_json_file(&agent_path.join("metadata.json"), &metadata)?;

        let details = AgentDetails::Human(config);
        write_json_file(&agent_path.join("details.json"), &details)?;

        let state = AgentState::default();
        write_json_file(&agent_path.join("state.json"), &state)?;

        Agent::load(project_root, &uid)
    }

    pub fn load(project_root: &Path, agent_uid: &str) -> Result<Self, ProjectError> {
        let agents_base_path = project_root.join(".vespe").join("agents");
        let agent_path = get_entity_path(&agents_base_path, agent_uid)?;

        if !agent_path.exists() {
            return Err(ProjectError::AgentNotFound(agent_uid.to_string()));
        }

        let metadata: AgentMetadata = read_json_file(&agent_path.join("metadata.json"))?;
        let details: AgentDetails = read_json_file(&agent_path.join("details.json"))?;
        let state: AgentState = read_json_file(&agent_path.join("state.json"))?;
        let memory = Memory::load(&agent_path.join("memory")).map_err(|e| ProjectError::Memory(e))?;

        let system_prompt_path = agent_path.join("system_prompt.hbs");
        let system_prompt = if system_prompt_path.exists() {
            Some(read_file_content(&system_prompt_path)?)
        } else {
            None
        };

        Ok(Agent {
            metadata,
            details,
            state,
            memory,
            system_prompt,
        })
    }

    /// Retrieves the AgentProtocol associated with this agent from the registry.
    pub async fn protocol(&self) -> Result<Arc<Box<dyn AgentProtocol + Send + Sync>>, ProjectError> {
        let registry = &AGENT_PROTOCOL_REGISTRY;
        let protocol_name = &self.metadata.protocol_name;

        registry.get(protocol_name)
            .map(|b| Arc::new(b.clone())) // Clone the Box and wrap in Arc
            .ok_or_else(|| ProjectError::AgentProtocolNotFound(protocol_name.clone()))
    }

    /// Sends a request to the LLM, handles formatting, querying, and parsing.
    pub async fn call_llm(
        &self,
        messages: Vec<Message>,
    ) -> Result<Vec<Message>, ProjectError> {
        let protocol = self.protocol().await?;

        let ai_config = match &self.details {
            AgentDetails::AI(config) => config,
            AgentDetails::Human(_) => return Err(ProjectError::InvalidOperation("Cannot call LLM for a Human agent.".to_string())),
        };

        // Retrieve allowed tools from the agent's configuration
        let allowed_tool_names = &ai_config.allowed_tools;
        let mut available_tools_for_protocol = Vec::new();
        let tool_registry = &TOOL_REGISTRY;

        for tool_name in allowed_tool_names {
            if let Some(tool_config) = tool_registry.get(tool_name) {
                available_tools_for_protocol.push(tool_config.config.clone()); // Assuming ToolConfig is what AgentProtocol expects
            } else {
                // Log a warning or return an error if an allowed tool is not found in the registry
                // For now, we'll just skip it, but a warning would be good.
                eprintln!("Warning: Allowed tool '{}' not found in TOOL_REGISTRY.", tool_name);
            }
        }

        // 1. Format messages using the agent's protocol
        let formatted_prompt = protocol.format_messages(messages, Some(available_tools_for_protocol)).await?;

        // 2. Get LLM client based on agent's configuration
        let llm_client = create_llm_client(&ai_config.llm_provider)?;

        // 3. Send query to LLM
        let raw_response = llm_client.send_query(formatted_prompt).await?;

        // 4. Parse LLM response using the agent's protocol
        let parsed_messages = protocol.parse_llm_output(raw_response).await?;

        // 5. Validate tool calls in parsed messages
        for message in &parsed_messages {
            if let MessageContent::ToolCall { tool_name, .. } = &message.content {
                if !allowed_tool_names.contains(tool_name) {
                    return Err(ProjectError::InvalidToolCall(format!("Agent attempted to call disallowed tool: '{}'.", tool_name)));
                }
            }
        }

        Ok(parsed_messages)
    }

    pub fn save_state(&self, project_root: &Path) -> Result<(), ProjectError> {
        let agents_base_path = project_root.join(".vespe").join("agents");
        let agent_path = get_entity_path(&agents_base_path, &self.metadata.uid)?;
        write_json_file(&agent_path.join("state.json"), &self.state)?;
        Ok(())
    }
}