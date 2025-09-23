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

#[derive(Serialize)]
pub struct LoadTaskResponse {
    pub task: Task,
}

#[derive(Deserialize)]
pub struct DefineObjectiveRequest {
    pub objective_content: String,
}

#[derive(Serialize)]
pub struct DefineObjectiveResponse {
    pub task_uid: String,
    pub new_state: String,
}

#[derive(Deserialize)]
pub struct DefinePlanRequest {
    pub plan_content: String,
}

#[derive(Serialize)]
pub struct DefinePlanResponse {
    pub task_uid: String,
    pub new_state: String,
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

#[derive(Deserialize)]
pub struct AddPersistentEventRequest {
    pub event_type: String,
    pub acting_agent_uid: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct AddPersistentEventResponse {
    pub message: String,
}

#[derive(Serialize)]
pub struct GetAllPersistentEventsResponse {
    pub events: Vec<vespe_project::PersistentEvent>,
}

#[derive(Serialize)]
pub struct CalculateResultHashResponse {
    pub hash: String,
}

#[derive(Deserialize)]
pub struct AddResultFileRequest {
    pub filename: String,
    pub content: String, // Base64 encoded content for binary files, or plain string for text
}

#[derive(Serialize)]
pub struct AddResultFileResponse {
    pub message: String,
}

#[derive(Deserialize)]
pub struct ReviewTaskRequest {
    pub approved: bool,
}

#[derive(Serialize)]
pub struct ReviewTaskResponse {
    pub task_uid: String,
    pub new_state: String,
}
