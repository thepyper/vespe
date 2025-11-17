use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::thread;
use tracing::{debug, error};
use crate::error::Error;

pub fn shell_call(command_template: &str, input: &str) -> anyhow::Result<String> {
    let mut command_parts = command_template.split_whitespace();
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

    let mut child = command.spawn().map_err(|source| Error::CommandSpawnError {
        command: command_template.to_string(),
        source,
    })?;

    child.stdin.as_mut()
        .ok_or_else(|| Error::StdinWriteError {
            command: command_template.to_string(),
            source: std::io::Error::new(std::io::ErrorKind::Other, "Stdin not available"),
        })?
        .write_all(input.as_bytes())
        .map_err(|source| Error::StdinWriteError {
            command: command_template.to_string(),
            source,
        })?;

    let stdout = child.stdout.take().ok_or(Error::StdoutCaptureError {
        command: command_template.to_string(),
    })?;
    let stderr = child.stderr.take().ok_or(Error::StderrCaptureError {
        command: command_template.to_string(),
    })?;

    #[allow(unused_assignments)]
    let mut full_stdout = Vec::new();
    #[allow(unused_assignments)]
    let mut full_stderr = Vec::new();

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    let stdout_handle = thread::spawn({
        let command_template = command_template.to_string();
        move || {
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
                        return Err(Error::StdoutReadError {
                            command: command_template,
                            source: e,
                        });
                    }
                }
            }
            Ok(buffer)
        }
    });

    let stderr_handle = thread::spawn({
        let command_template = command_template.to_string();
        move || {
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
                        return Err(Error::StderrReadError {
                            command: command_template,
                            source: e,
                        });
                    }
                }
            }
            Ok(buffer)
        }
    });

    let status = child.wait()?;

    full_stdout = stdout_handle.join().map_err(|_| Error::StdoutThreadJoinError {
        command: command_template.to_string(),
    })??;
    full_stderr = stderr_handle.join().map_err(|_| Error::StderrThreadJoinError {
        command: command_template.to_string(),
    })??;

    debug!("Command finished with status: {:?}", status);
    if !status.success() {
        error!(
            "Command '{}' failed: {:?}",
            command_template,
            String::from_utf8_lossy(&full_stderr)
        );
        return Err(Error::CommandFailed {
            command: command_template.to_string(),
            exit_code: status.code().unwrap_or(-1), // Use -1 if exit code is not available
            stderr: String::from_utf8_lossy(&full_stderr).to_string(),
        }
        .into());
    }

    debug!(
        "Command executed successfully. Output length: {}",
        full_stdout.len()
    );
    Ok(String::from_utf8_lossy(&full_stdout).to_string())
}
