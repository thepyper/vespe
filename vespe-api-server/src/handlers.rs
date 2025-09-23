use axum::{extract::State, http::StatusCode, Json};
use std::path::PathBuf;

use vespe_project::api;
use vespe_project::error::ProjectError;
use vespe_project::{Agent, AgentType, Task};

use crate::models::{
    CreateAgentRequest, CreateAgentResponse, CreateTaskRequest, CreateTaskResponse,
    ListAgentsResponse, ListTasksResponse,
};
use crate::error::map_project_error_to_http_response;

// --- App State ---
#[derive(Clone)]
pub struct AppState {
    pub project_root: PathBuf,
}

pub async fn create_task_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Json<CreateTaskResponse>, (StatusCode, String)> {
    let result = api::create_task(
        &app_state.project_root,
        payload.parent_uid,
        payload.name,
        payload.created_by_agent_uid,
        payload.template_name,
    );

    match result {
        Ok(task) => Ok(Json(CreateTaskResponse {
            task_uid: task.uid,
            task_name: task.config.name,
        })),
        Err(e) => Err(map_project_error_to_http_response(e)),
    }
}

pub async fn list_tasks_handler(
    State(app_state): State<AppState>,
) -> Result<Json<ListTasksResponse>, (StatusCode, String)> {
    let result = api::list_all_tasks(&app_state.project_root);

    match result {
        Ok(tasks) => Ok(Json(ListTasksResponse { tasks })),
        Err(e) => Err(map_project_error_to_http_response(e)),
    }
}

pub async fn create_agent_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateAgentRequest>,
) -> Result<Json<CreateAgentResponse>, (StatusCode, String)> {
    let result = api::create_agent(
        &app_state.project_root,
        payload.agent_type,
        payload.name,
    );

    match result {
        Ok(agent) => Ok(Json(CreateAgentResponse {
            agent_uid: agent.uid,
            agent_name: agent.name,
        })),
        Err(e) => Err(map_project_error_to_http_response(e)),
    }
}

pub async fn list_agents_handler(
    State(app_state): State<AppState>,
) -> Result<Json<ListAgentsResponse>, (StatusCode, String)> {
    let result = api::list_agents(&app_state.project_root);

    match result {
        Ok(agents) => Ok(Json(ListAgentsResponse { agents })),
        Err(e) => Err(map_project_error_to_http_response(e)),
    }
}
