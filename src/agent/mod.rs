use std::io::Write;
use std::process::{Command, Stdio};
use tracing::{debug, error};
use thiserror::Error;

use crate::project::Project;
use crate::error::{Result, Error as GeneralError};

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Command template cannot be empty")]
    EmptyCommandTemplate,
    #[error("Failed to spawn command '{command}': {source}")]
    CommandSpawnFailed { command: String, source: std::io::Error },
    #[error("Failed to write to stdin: {0}")]
    StdinWriteFailed(#[from] std::io::Error),
    #[error("Command '{command}' failed: {stderr}")]
    CommandFailed { command: String, stderr: String },
    #[error("Failed to get command output: {0}")]
    CommandOutputFailed(std::io::Error),
    #[error("Failed to convert command output to UTF-8: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}

pub struct ShellAgentCall<'a> {
    command_template: String,
    _project: &'a Project,
}

impl<'a> ShellAgentCall<'a> {
    pub fn new(command: String, project: &'a Project) -> Result<Self> {
        Ok(Self {
            command_template: command,
            _project: project,
        })
    }

    pub fn call(&self, query: &str) -> Result<String> {
        //debug!("ShellAgentCall received query: {}", query);
        let mut command_parts = self.command_template.split_whitespace();
        let program = command_parts
            .next()
            .ok_or(AgentError::EmptyCommandTemplate)?;
        let args: Vec<&str> = command_parts.collect();

        let mut command = {
            #[cfg(windows)]
            {
                let mut cmd = Command::new("cmd");
                cmd.arg("/C").arg(program).args(args.clone());
                cmd
            }
            #[cfg(not(windows))]
            {
                Command::new(program).args(args)
            }
        };

        debug!("Executing command: {:?} with args: {:?}", program, args);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command.spawn().map_err(|e| {
            AgentError::CommandSpawnFailed {
                command: self.command_template.clone(),
                source: e,
            }
        })?;

        debug!("Writing query to stdin.");
        child.stdin.as_mut().ok_or(AgentError::StdinWriteFailed(std::io::Error::new(std::io::ErrorKind::Other, "Failed to get stdin")))?
            .write_all(query.as_bytes())?;
        let output = child.wait_with_output().map_err(AgentError::CommandOutputFailed)?;

        debug!("Command finished with status: {:?}", output.status);
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            error!(
                "Command '{}' failed: {:?}",
                self.command_template,
                stderr
            );
            return Err(AgentError::CommandFailed {
                command: self.command_template.clone(),
                stderr,
            }.into());
        }

        debug!(
            "Command executed successfully. Output length: {}",
            output.stdout.len()
        );
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
