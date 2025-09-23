use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use vespe_project::api;
use vespe_project::error::ProjectError;
use vespe_project::{Agent, AgentType, Task};

// --- App State ---
#[derive(Clone)]
pub struct AppState {
    pub project_root: PathBuf,
}

// --- DTOs (Data Transfer Objects) ---

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

// --- Handlers ---

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

// --- Error Mapping ---
pub fn map_project_error_to_http_response(error: ProjectError) -> (StatusCode, String) {
    match error {
        ProjectError::TaskNotFound(msg) => (StatusCode::NOT_FOUND, msg),
        ProjectError::AgentNotFound(msg) => (StatusCode::NOT_FOUND, msg),
        ProjectError::ToolNotFound(msg) => (StatusCode::NOT_FOUND, msg),
        ProjectError::InvalidPath(path) => (StatusCode::BAD_REQUEST, format!("Invalid path: {}", path.display())),
        ProjectError::Io(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("I/O error: {}", e)),
        ProjectError::Json(e) => (StatusCode::BAD_REQUEST, format!("JSON parsing error: {}", e)),
        ProjectError::InvalidProjectConfig(msg) => (StatusCode::BAD_REQUEST, msg),
        ProjectError::InvalidStateTransition(from, to) => (StatusCode::BAD_REQUEST, format!("Invalid state transition: from {:?} to {:?}", from, to)),
        ProjectError::UnexpectedState(state) => (StatusCode::BAD_REQUEST, format!("Unexpected state: {:?}", state)),
        ProjectError::MissingRequiredFile(path) => (StatusCode::BAD_REQUEST, format!("Missing required file: {}", path.display())),
        ProjectError::DependencyCycle(msg) => (StatusCode::BAD_REQUEST, format!("Dependency cycle detected: {}", msg)),
        ProjectError::ContentHashError(path, msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Content hash error for {}: {}", path.display(), msg)),
        ProjectError::UidGenerationError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("UID generation error: {}", msg)),
        ProjectError::ProjectRootNotFound(path) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Project root not found: {}", path.display())),
    }
}

// --- Main Server Setup ---
#[tokio::main]
pub async fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "vespe_api_server=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let project_root = PathBuf::from(std::env::var("VESPE_PROJECT_ROOT")
        .expect("VESPE_PROJECT_ROOT environment variable must be set"));

    let app_state = AppState { project_root };

    let app = Router::new()
        // Task Endpoints
        .route("/tasks", post(create_task_handler))
        .route("/tasks", get(list_tasks_handler))
        // Agent Endpoints
        .route("/agents", post(create_agent_handler))
        .route("/agents", get(list_agents_handler))
        // ... other endpoints for tool, task show, define_objective, etc.
        .with_state(app_state); // Pass state to all handlers

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}