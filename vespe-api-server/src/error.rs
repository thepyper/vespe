use axum::http::StatusCode;
use vespe_project::error::ProjectError;

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
