use serde::{Deserialize, Serialize};
use vespe_project::{Agent, AgentType, Task};

// Task DTOs
#[derive(Deserialize)]
pub struct CreateTaskRequest {
    pub parent_uid: Option<String>,
    pub name: String,
    pub created_by_agent_uid: String,
    pub template_name: String,
}

#[derive(Serialize)]
pub struct CreateTaskResponse {
    pub task_uid: String,
    pub task_name: String,
}

#[derive(Serialize)]
pub struct ListTasksResponse {
    pub tasks: Vec<Task>,
}

// Agent DTOs
#[derive(Deserialize)]
pub struct CreateAgentRequest {
    pub agent_type: AgentType,
    pub name: String,
}

#[derive(Serialize)]
pub struct CreateAgentResponse {
    pub agent_uid: String,
    pub agent_name: String,
}

#[derive(Serialize)]
pub struct ListAgentsResponse {
    pub agents: Vec<Agent>,
}
