use axum::{extract::{State, Path}, http::StatusCode, Json};
use std::path::PathBuf;

use vespe_project::api;
use vespe_project::error::ProjectError;
use vespe_project::{Agent, AgentType, Task};

use super::models::{
    CreateAgentRequest, CreateAgentResponse, CreateTaskRequest, CreateTaskResponse,
    ListAgentsResponse, ListTasksResponse, LoadTaskResponse,
    DefineObjectiveRequest, DefineObjectiveResponse,
    DefinePlanRequest, DefinePlanResponse,
    AddPersistentEventRequest, AddPersistentEventResponse,
    GetAllPersistentEventsResponse, CalculateResultHashResponse,
    AddResultFileRequest, AddResultFileResponse,
    ReviewTaskRequest, ReviewTaskResponse,
    CreateToolRequest, CreateToolResponse, LoadToolResponse, ResolveToolResponse,
    ListAvailableToolsResponse, LoadProjectConfigResponse,
    SaveProjectConfigRequest, SaveProjectConfigResponse,
};
use vespe_project::ProjectConfig;
use vespe_project::PersistentEvent;
use chrono::Utc;
use super::error::map_project_error_to_http_response;

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

pub async fn load_task_handler(
    State(app_state): State<AppState>,
    Path(task_uid): Path<String>,
) -> Result<Json<LoadTaskResponse>, (StatusCode, String)> {
    let result = api::load_task(&app_state.project_root, &task_uid);

    match result {
        Ok(task) => Ok(Json(LoadTaskResponse { task })),
        Err(e) => Err(map_project_error_to_http_response(e)),
    }
}

pub async fn define_objective_handler(
    State(app_state): State<AppState>,
    Path(task_uid): Path<String>,
    Json(payload): Json<DefineObjectiveRequest>,
) -> Result<Json<DefineObjectiveResponse>, (StatusCode, String)> {
    let result = api::define_objective(
        &app_state.project_root,
        &task_uid,
        payload.objective_content,
    );

    match result {
        Ok(task) => Ok(Json(DefineObjectiveResponse {
            task_uid: task.uid,
            new_state: task.status.current_state.to_string(), // Assuming TaskState can be converted to String
        })),
        Err(e) => Err(map_project_error_to_http_response(e)),
    }
}

pub async fn define_plan_handler(
    State(app_state): State<AppState>,
    Path(task_uid): Path<String>,
    Json(payload): Json<DefinePlanRequest>,
) -> Result<Json<DefinePlanResponse>, (StatusCode, String)> {
    let result = api::define_plan(
        &app_state.project_root,
        &task_uid,
        payload.plan_content,
    );

    match result {
        Ok(task) => Ok(Json(DefinePlanResponse {
            task_uid: task.uid,
            new_state: task.status.current_state.to_string(), // Assuming TaskState can be converted to String
        })),
        Err(e) => Err(map_project_error_to_http_response(e)),
    }
}

pub async fn add_persistent_event_handler(
    State(app_state): State<AppState>,
    Path(task_uid): Path<String>,
    Json(payload): Json<AddPersistentEventRequest>,
) -> Result<Json<AddPersistentEventResponse>, (StatusCode, String)> {
    let event = PersistentEvent {
        timestamp: Utc::now(),
        event_type: payload.event_type,
        acting_agent_uid: payload.acting_agent_uid,
        content: payload.content,
    };

    let result = api::add_persistent_event(&app_state.project_root, &task_uid, event);

    match result {
        Ok(_) => Ok(Json(AddPersistentEventResponse {
            message: "Persistent event added successfully".to_string(),
        })),
        Err(e) => Err(map_project_error_to_http_response(e)),
    }
}

pub async fn get_all_persistent_events_handler(
    State(app_state): State<AppState>,
    Path(task_uid): Path<String>,
) -> Result<Json<GetAllPersistentEventsResponse>, (StatusCode, String)> {
    let result = api::get_all_persistent_events(&app_state.project_root, &task_uid);

    match result {
        Ok(events) => Ok(Json(GetAllPersistentEventsResponse { events })),
        Err(e) => Err(map_project_error_to_http_response(e)),
    }
}

pub async fn calculate_result_hash_handler(
    State(app_state): State<AppState>,
    Path(task_uid): Path<String>,
) -> Result<Json<CalculateResultHashResponse>, (StatusCode, String)> {
    let result = api::calculate_result_hash(&app_state.project_root, &task_uid);

    match result {
        Ok(hash) => Ok(Json(CalculateResultHashResponse { hash })),
        Err(e) => Err(map_project_error_to_http_response(e)),
    }
}

pub async fn add_result_file_handler(
    State(app_state): State<AppState>,
    Path(task_uid): Path<String>,
    Json(payload): Json<AddResultFileRequest>,
) -> Result<Json<AddResultFileResponse>, (StatusCode, String)> {
    let content_bytes = base64::decode(payload.content.as_bytes())
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid base64 content: {}", e)))?;

    let result = api::add_result_file(
        &app_state.project_root,
        &task_uid,
        &payload.filename,
        content_bytes,
    );

    match result {
        Ok(_) => Ok(Json(AddResultFileResponse {
            message: format!("File {} added successfully to task {}", payload.filename, task_uid),
        })),
        Err(e) => Err(map_project_error_to_http_response(e)),
    }
}

pub async fn review_task_handler(
    State(app_state): State<AppState>,
    Path(task_uid): Path<String>,
    Json(payload): Json<ReviewTaskRequest>,
) -> Result<Json<ReviewTaskResponse>, (StatusCode, String)> {
    let result = api::review_task(
        &app_state.project_root,
        &task_uid,
        payload.approved,
    );

    match result {
        Ok(task) => Ok(Json(ReviewTaskResponse {
            task_uid: task.uid,
            new_state: task.status.current_state.to_string(),
        })),
        Err(e) => Err(map_project_error_to_http_response(e)),
    }
}

pub async fn create_tool_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateToolRequest>,
) -> Result<Json<CreateToolResponse>, (StatusCode, String)> {
    let result = api::create_tool(
        &app_state.project_root,
        payload.name,
        payload.description,
        payload.schema,
        payload.implementation_details,
    );

    match result {
        Ok(tool) => Ok(Json(CreateToolResponse {
            tool_uid: tool.uid,
            tool_name: tool.config.name,
        })),
        Err(e) => Err(map_project_error_to_http_response(e)),
    }
}

pub async fn load_tool_handler(
    State(app_state): State<AppState>,
    Path(tool_uid): Path<String>,
) -> Result<Json<LoadToolResponse>, (StatusCode, String)> {
    // In vespe_project::api::load_tool, it expects a Path to the tool's root_path.
    // We need to construct that path from project_root and tool_uid.
    // This requires a helper function in vespe_project::utils or similar.
    // For now, we'll mock it or assume a direct load by UID is possible.
    // Based on vespe_project::api::load_tool, it takes `tool_path: &Path`.
    // We need to get the full path to the tool's directory.
    // `create_tool` uses `get_entity_path` to construct the tool_path.
    // So, we need to construct the tool_path here.

    let tools_base_path = app_state.project_root.join(".vespe").join("tools");
    let tool_path = tools_base_path.join(&tool_uid);

    let result = api::load_tool(&tool_path);

    match result {
        Ok(tool) => Ok(Json(LoadToolResponse { tool })),
        Err(e) => Err(map_project_error_to_http_response(e)),
    }
}

pub async fn resolve_tool_handler(
    State(app_state): State<AppState>,
    Path(tool_name): Path<String>,
) -> Result<Json<ResolveToolResponse>, (StatusCode, String)> {
    // vespe_project::api::resolve_tool requires a ProjectConfig, but for now we'll use default.
    let project_config = vespe_project::ProjectConfig::default();
    let result = api::resolve_tool(&app_state.project_root, &project_config, &tool_name);

    match result {
        Ok(tool) => Ok(Json(ResolveToolResponse { tool })),
        Err(e) => Err(map_project_error_to_http_response(e)),
    }
}

pub async fn list_available_tools_handler(
    State(app_state): State<AppState>,
) -> Result<Json<ListAvailableToolsResponse>, (StatusCode, String)> {
    let project_config = vespe_project::ProjectConfig::default();
    let result = api::list_available_tools(&app_state.project_root, &project_config);

    match result {
        Ok(tools) => Ok(Json(ListAvailableToolsResponse { tools })),
        Err(e) => Err(map_project_error_to_http_response(e)),
    }
}

pub async fn load_project_config_handler(
    State(app_state): State<AppState>,
) -> Result<Json<LoadProjectConfigResponse>, (StatusCode, String)> {
    let result = api::load_project_config(&app_state.project_root);

    match result {
        Ok(config) => Ok(Json(LoadProjectConfigResponse { config })),
        Err(e) => Err(map_project_error_to_http_response(e)),
    }
}

pub async fn save_project_config_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<SaveProjectConfigRequest>,
) -> Result<Json<SaveProjectConfigResponse>, (StatusCode, String)> {
    let result = api::save_project_config(&app_state.project_root, &payload.config);

    match result {
        Ok(_) => Ok(Json(SaveProjectConfigResponse {
            message: "Project configuration saved successfully".to_string(),
        })),
        Err(e) => Err(map_project_error_to_http_response(e)),
    }
}
