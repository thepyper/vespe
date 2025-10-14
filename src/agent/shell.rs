use anyhow::Context;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::thread;
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
        //debug!("ShellAgentCall received query: {}", query);
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

        let stdout = child.stdout.take().context("Failed to take stdout")?;
        let stderr = child.stderr.take().context("Failed to take stderr")?;

#[allow(unused_assignments)]
        let mut full_stdout = Vec::new();
#[allow(unused_assignments)]
        let mut full_stderr = Vec::new();

        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        let stdout_handle = thread::spawn(move || {
            let mut line = String::new();
            let mut reader = stdout_reader;
            let mut buffer = Vec::new();
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        debug!("Child stdout: {}", line.trim_end());
                        buffer.extend_from_slice(line.as_bytes());
                    }
                    Err(e) => {
                        error!("Error reading child stdout: {:?}", e);
                        break;
                    }
                }
            }
            buffer
        });

        let stderr_handle = thread::spawn(move || {
            let mut line = String::new();
            let mut reader = stderr_reader;
            let mut buffer = Vec::new();
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        debug!("Child stderr: {}", line.trim_end());
                        buffer.extend_from_slice(line.as_bytes());
                    }
                    Err(e) => {
                        error!("Error reading child stderr: {:?}", e);
                        break;
                    }
                }
            }
            buffer
        });

        let status = child.wait()?;

        full_stdout = stdout_handle.join().expect("Failed to join stdout thread");
        full_stderr = stderr_handle.join().expect("Failed to join stderr thread");

        debug!("Command finished with status: {:?}", status);
        if !status.success() {
            error!(
                "Command '{}' failed: {:?}",
                self.command_template,
                String::from_utf8_lossy(&full_stderr)
            );
            anyhow::bail!(
                "Command '{}' failed: {:?}",
                self.command_template,
                String::from_utf8_lossy(&full_stderr)
            );
        }

        debug!(
            "Command executed successfully. Output length: {}",
            full_stdout.len()
        );
        Ok(String::from_utf8_lossy(&full_stdout).to_string())
    }
}
