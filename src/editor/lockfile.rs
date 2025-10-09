use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use std::env;
use std::thread::sleep;
use std::time::Duration;
use uuid::Uuid;
use thiserror::Error;
use crate::editor::EditorError;

use super::EditorCommunicator;

#[derive(Error, Debug)]
pub enum LockfileError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON serialization/deserialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Invalid file path")]
    InvalidFilePath,
    #[error("Timeout waiting for editor response")]
    Timeout,
    #[error("Editor error: {0}")]
    Editor(String),
    #[error("Unexpected editor response")]
    UnexpectedResponse,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum RequestState {
    /// Request to modify a file. The editor should save and lock it.
    RequestModification {
        file_path: PathBuf,
        request_id: Uuid,
    },
    /// Notification that the program has finished modifying the file. The editor should reload and unlock it.
    ModificationComplete {
        file_path: PathBuf,
        request_id: Uuid,
    },
    /// No active request.
    None,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ResponseState {
    /// Editor has saved and locked the file, ready for modification.
    FileLocked {
        file_path: PathBuf,
        request_id: Uuid,
    },
    /// Editor has reloaded and unlocked the file.
    FileUnlocked {
        file_path: PathBuf,
        request_id: Uuid,
    },
    /// Editor is busy or encountered an error.
    Error { message: String, request_id: Uuid },
    /// No active response.
    None,
}

pub struct FileBasedEditorCommunicator {
    request_file_path: PathBuf,
    response_file_path: PathBuf,
}

impl FileBasedEditorCommunicator {
    pub fn new(path: &Path) -> Result<Self, LockfileError> {
        let request_file: PathBuf = path.join("vespe_request.json");
        let response_file: PathBuf = path.join("vespe_response.json");

        // Ensure the parent directories exist
        if let Some(parent) = request_file.parent() {
            fs::create_dir_all(parent).map_err(LockfileError::Io)?;
        }
        if let Some(parent) = response_file.parent() {
            fs::create_dir_all(parent).map_err(LockfileError::Io)?;
        }

        // Initialize request and response files with None state
        fs::write(
            &request_file,
            serde_json::to_string(&RequestState::None).map_err(LockfileError::SerdeJson)?,
        )
        .map_err(LockfileError::Io)?;
        fs::write(
            &response_file,
            serde_json::to_string(&ResponseState::None).map_err(LockfileError::SerdeJson)?,
        )
        .map_err(LockfileError::Io)?;

        // Set environment variables for the VSCode extension to pick up
        env::set_var(
            "VESPE_REQUEST_FILE_PATH",
            request_file
                .to_str()
                .ok_or(LockfileError::InvalidFilePath)?,
        );
        env::set_var(
            "VESPE_RESPONSE_FILE_PATH",
            response_file
                .to_str()
                .ok_or(LockfileError::InvalidFilePath)?,
        );

        Ok(Self {
            request_file_path: request_file,
            response_file_path: response_file,
        })
    }

    fn _write_request(&self, state: RequestState) -> Result<(), LockfileError> {
        let json = serde_json::to_string_pretty(&state).map_err(LockfileError::SerdeJson)?;
        fs::write(&self.request_file_path, json).map_err(LockfileError::Io)?;
        Ok(())
    }

    fn _read_response(&self, expected_request_id: Uuid) -> Result<ResponseState, LockfileError> {
        // TODO: Implement actual filesystem event watching and polling
        // For now, a simple polling mechanism
        let mut attempts = 0;
        loop {
            let content = fs::read_to_string(&self.response_file_path).map_err(LockfileError::Io)?;
            let response: ResponseState = serde_json::from_str(&content).map_err(LockfileError::SerdeJson)?;

            match &response {
                ResponseState::FileLocked { request_id, .. }
                | ResponseState::FileUnlocked { request_id, .. }
                | ResponseState::Error { request_id, .. } => {
                    if *request_id == expected_request_id {
                        // Reset the response file to None after reading
                        fs::write(
                            &self.response_file_path,
                            serde_json::to_string(&ResponseState::None).map_err(LockfileError::SerdeJson)?,
                        )
                        .map_err(LockfileError::Io)?;
                        return Ok(response);
                    }
                }
                ResponseState::None => {} // Continue waiting
            }

            attempts += 1;
            if attempts > 60 {
                // Timeout after 5 minutes (60 * 5 seconds)
                return Err(LockfileError::Timeout);
            }
            sleep(Duration::from_secs(5));
        }
    }
}

impl EditorCommunicator for FileBasedEditorCommunicator {
    fn request_file_modification(&self, file_path: &Path) -> Result<Uuid, EditorError> {
        let request_id = Uuid::new_v4();
        let request = RequestState::RequestModification {
            file_path: file_path.to_path_buf(),
            request_id,
        };
        self._write_request(request)?;

        // Wait for the editor's response
        let response = self._read_response(request_id)?;
        match response {
            ResponseState::FileLocked { .. } => Ok(request_id),
            ResponseState::Error { message, .. } => Err(LockfileError::Editor(message).into()),
            _ => Err(LockfileError::UnexpectedResponse.into()),
        }
    }

    fn notify_file_modified(&self, file_path: &Path, request_id: Uuid) -> Result<(), EditorError> {
        let request = RequestState::ModificationComplete {
            file_path: file_path.to_path_buf(),
            request_id,
        };
        self._write_request(request)?;

        // Wait for the editor's response
        let response = self._read_response(request_id)?;
        match response {
            ResponseState::FileUnlocked { .. } => Ok(()),
            ResponseState::Error { message, .. } => Err(LockfileError::Editor(message).into()),
            _ => Err(LockfileError::UnexpectedResponse.into()),
        }
    }
}
