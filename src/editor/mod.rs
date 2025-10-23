//! This module defines the interface for communicating with a text editor extension.

use anyhow::Result;
use std::path::Path;
use uuid::Uuid;

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
    fn notify_file_modified(&self, request_id: Uuid) -> Result<()>;
}

pub mod lockfile;
