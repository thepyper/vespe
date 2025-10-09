use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use std::env;
use std::thread::sleep;
use std::time::Duration;
use uuid::Uuid;

use super::EditorCommunicator;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON serialization/deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Timeout waiting for editor response")]
    Timeout,
    #[error("Editor error: {0}")]
    EditorError(String),
    #[error("Unexpected editor response")]
    UnexpectedResponse,
}

pub type Result<T> = std::result::Result<T, Error>;

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
    pub fn new(path: &Path) -> Result<Self> {
        let request_file: PathBuf = path.join("vespe_request.json");
        let response_file: PathBuf = path.join("vespe_response.json");

        if let Some(parent) = request_file.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = response_file.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&request_file, serde_json::to_string(&RequestState::None)?)?;
        fs::write(&response_file, serde_json::to_string(&ResponseState::None)?)?;

        env::set_var(
            "VESPE_REQUEST_FILE_PATH",
            request_file
                .to_str()
                .ok_or_else(|| Error::InvalidPath(request_file.display().to_string()))?,
        );
        env::set_var(
            "VESPE_RESPONSE_FILE_PATH",
            response_file
                .to_str()
                .ok_or_else(|| Error::InvalidPath(response_file.display().to_string()))?,
        );

        Ok(Self {
            request_file_path: request_file,
            response_file_path: response_file,
        })
    }

    fn _write_request(&self, state: RequestState) -> Result<()> {
        let json = serde_json::to_string_pretty(&state)?;
        fs::write(&self.request_file_path, json)?;
        Ok(())
    }

    fn _read_response(&self, expected_request_id: Uuid) -> Result<ResponseState> {
        let mut attempts = 0;
        loop {
            let content = fs::read_to_string(&self.response_file_path)?;
            let response: ResponseState = serde_json::from_str(&content)?;

            match &response {
                ResponseState::FileLocked { request_id, .. }
                | ResponseState::FileUnlocked { request_id, .. }
                | ResponseState::Error { request_id, .. } => {
                    if *request_id == expected_request_id {
                        fs::write(
                            &self.response_file_path,
                            serde_json::to_string(&ResponseState::None)?,
                        )?;
                        return Ok(response);
                    }
                }
                ResponseState::None => {} // Continue waiting
            }

            attempts += 1;
            if attempts > 60 {
                return Err(Error::Timeout);
            }
            sleep(Duration::from_secs(5));
        }
    }
}

impl EditorCommunicator for FileBasedEditorCommunicator {
    fn request_file_modification(&self, file_path: &Path) -> Result<Uuid> {
        let request_id = Uuid::new_v4();
        let request = RequestState::RequestModification {
            file_path: file_path.to_path_buf(),
            request_id,
        };
        self._write_request(request)?;

        let response = self._read_response(request_id)?;
        match response {
            ResponseState::FileLocked { .. } => Ok(request_id),
            ResponseState::Error { message, .. } => {
                Err(Error::EditorError(message))
            }
            _ => Err(Error::UnexpectedResponse),
        }
    }

    fn notify_file_modified(&self, file_path: &Path, request_id: Uuid) -> Result<()> {
        let request = RequestState::ModificationComplete {
            file_path: file_path.to_path_buf(),
            request_id,
        };
        self._write_request(request)?;

        let response = self._read_response(request_id)?;
        match response {
            ResponseState::FileUnlocked { .. } => Ok(()),
            ResponseState::Error { message, .. } => {
                Err(Error::EditorError(message))
            }
            _ => Err(Error::UnexpectedResponse),
        }
    }
}
