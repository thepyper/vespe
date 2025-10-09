use std::io::Write;
use std::process::{Command, Stdio};
use tracing::{debug, error};
use thiserror::Error;

use crate::project::Project;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Command template cannot be empty")]
    EmptyCommandTemplate,
    #[error("Failed to spawn command '{0}'. Is it in your PATH?")]
    CommandSpawnError(String),
    #[error("Command '{0}' failed: {1}")]
    CommandFailed(String, String),
    #[error("Invalid command template: {0}")]
    InvalidCommandTemplate(String),
}

pub type Result<T> = std::result::Result<T, Error>;

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
        let mut command_parts = self.command_template.split_whitespace();
        let program = command_parts
            .next()
            .ok_or(Error::EmptyCommandTemplate)?;
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

        let mut child = command.spawn().map_err(|_| {
            Error::CommandSpawnError(self.command_template.clone())
        })?;

        debug!("Writing query to stdin.");
        child.stdin.as_mut().unwrap().write_all(query.as_bytes())?;
        let output = child.wait_with_output()?;

        debug!("Command finished with status: {:?}", output.status);
        if !output.status.success() {
            error!(
                "Command '{}' failed: {:?}",
                self.command_template,
                String::from_utf8_lossy(&output.stderr)
            );
            return Err(Error::CommandFailed(
                self.command_template.clone(),
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        debug!(
            "Command executed successfully. Output length: {}",
            output.stdout.len()
        );
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
