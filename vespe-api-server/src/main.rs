use axum::{
    routing::{get, post, put},
    Router,
};
use std::{net::SocketAddr, path::PathBuf};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod models; // Declare the models module
mod error; // Declare the error module
mod handlers; // Declare the handlers module

use crate::handlers::{
    create_agent_handler, create_task_handler, list_agents_handler, list_tasks_handler, load_task_handler, define_objective_handler, define_plan_handler, add_persistent_event_handler, get_all_persistent_events_handler, calculate_result_hash_handler, add_result_file_handler, review_task_handler, create_tool_handler, load_tool_handler, resolve_tool_handler, list_available_tools_handler, load_project_config_handler, save_project_config_handler, AppState,
};

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
        .route("/tasks/:task_uid", get(load_task_handler))
        .route("/tasks/:task_uid/objective", post(define_objective_handler))
        .route("/tasks/:task_uid/plan", post(define_plan_handler))
        .route("/tasks/:task_uid/events", post(add_persistent_event_handler))
        .route("/tasks/:task_uid/events", get(get_all_persistent_events_handler))
        .route("/tasks/:task_uid/result/hash", get(calculate_result_hash_handler))
        .route("/tasks/:task_uid/result/files", post(add_result_file_handler))
        .route("/tasks/:task_uid/review", post(review_task_handler))
        // Tool Endpoints
        .route("/tools", post(create_tool_handler))
        .route("/tools/:tool_uid", get(load_tool_handler))
        .route("/tools/resolve/:tool_name", get(resolve_tool_handler))
        .route("/tools", get(list_available_tools_handler))
        // Project Endpoints
        .route("/project/config", get(load_project_config_handler))
        .route("/project/config", put(save_project_config_handler))
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
