//! This module defines the interface for communicating with a text editor extension.

use std::path::Path;
use uuid::Uuid;
use thiserror::Error;

use crate::error::Result;

#[derive(Error, Debug)]
pub enum EditorError {
    #[error("Failed to request file modification: {0}")]
    FileModificationRequestFailed(String),
    #[error("Failed to notify file modified: {0}")]
    FileModifiedNotificationFailed(String),
    #[error("Failed to create directory: {0}")]
    CreateDirFailed(#[from] std::io::Error),
    #[error("Failed to serialize JSON: {0}")]
    SerializeJsonFailed(#[from] serde_json::Error),
    #[error("Invalid file path: {0}")]
    InvalidFilePath(String),
    #[error("Failed to write file: {0}")]
    WriteFileFailed(#[from] std::io::Error),
    #[error("Failed to read file: {0}")]
    ReadFileFailed(#[from] std::io::Error),
    #[error("Failed to deserialize JSON: {0}")]
    DeserializeJsonFailed(#[from] serde_json::Error),
    #[error("Timeout waiting for editor response")]
    Timeout,
    #[error("Editor error: {0}")]
    EditorResponseError(String),
    #[error("Unexpected editor response")]
    UnexpectedEditorResponse,
}

/// Trait for communicating with a text editor extension.
pub trait EditorCommunicator {
    /// Requests the editor to prepare a file for modification.
    /// If the file is open, the editor should save it and ideally lock it to prevent external changes.
    ///
    /// # Arguments
    /// * `file_path` - The absolute path to the file to be modified.
    ///
    /// # Returns
    /// `Ok(Uuid)` with a request ID if the request was successful, `Err` otherwise.
    fn request_file_modification(&self, file_path: &Path) -> Result<Uuid>;

    /// Notifies the editor that a file has been modified by the program.
    /// If the file is open, the editor should reload it and unlock it.
    ///
    /// # Arguments
    /// * `file_path` - The absolute path to the file that was modified.
    /// * `request_id` - The ID of the original modification request.
    ///
    /// # Returns
    /// `Ok(())` if the notification was successful, `Err` otherwise.
    fn notify_file_modified(&self, file_path: &Path, request_id: Uuid) -> Result<()>;
}

pub mod lockfile;

pub struct DummyEditorCommunicator;

impl EditorCommunicator for DummyEditorCommunicator {
    fn request_file_modification(&self, _file_path: &Path) -> Result<Uuid> {
        // For the dummy communicator, we just return a new UUID without doing anything.
        Ok(Uuid::new_v4())
    }

    fn notify_file_modified(&self, _file_path: &Path, _request_id: Uuid) -> Result<()> {
        // For the dummy communicator, we do nothing.
        Ok(())
    }
}
