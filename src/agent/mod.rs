use anyhow::Context;
use std::io::Write;
use std::process::{Command, Stdio};
use tracing::{debug, error};

use crate::project::Project;

pub struct ShellAgentCall<'a> {
    command_template: String,
    _project: &'a Project,
}

impl<'a> ShellAgentCall<'a> {
    pub fn new(command: String, project: &'a Project) -> anyhow::Result<Self> {
        Ok(Self {
            command_template: command,
            _project: project,
        })
    }

    pub fn call(&self, query: &str) -> anyhow::Result<String> {
        debug!("ShellAgentCall received query: {}", query);
        let mut command_parts = self.command_template.split_whitespace();
        let program = command_parts
            .next()
            .context("Command template cannot be empty")?;
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

        let mut child = command.spawn().with_context(|| {
            format!(
                "Failed to spawn command: '{}'. Is it in your PATH?",
                self.command_template
            )
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
            anyhow::bail!(
                "Command '{}' failed: {:?}",
                self.command_template,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        debug!(
            "Command executed successfully. Output length: {}",
            output.stdout.len()
        );
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
