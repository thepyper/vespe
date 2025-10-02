use anyhow::{Result, Context};
use std::process::{Command, Stdio};
use std::io::Write;

pub trait AgentCall {
    fn call_llm(&self, prompt: String) -> Result<String>;
}

pub struct ShellAgentCall {
    command_template: String,
}

impl ShellAgentCall {
    pub fn new(command_template: String) -> Self {
        Self { command_template }
    }
}

impl AgentCall for ShellAgentCall {
    fn call_llm(&self, prompt: String) -> Result<String> {
        let mut command_parts = self.command_template.split_whitespace();
        let program = command_parts.next().context("Command template cannot be empty")?;
        let args: Vec<&str> = command_parts.collect();

        let mut command = {
            #[cfg(windows)]
            {
                let mut cmd = Command::new("cmd");
                cmd.arg("/C").arg(program).args(args);
                cmd
            }
            #[cfg(not(windows))]
            {
                Command::new(program).args(args)
            }
        };

        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command
            .spawn()
            .with_context(|| format!("Failed to spawn command: '{}'. Is it in your PATH?", self.command_template))?;

        child.stdin.as_mut().unwrap().write_all(prompt.as_bytes())?;
        let output = child.wait_with_output()?;

        if !output.status.success() {
            anyhow::bail!("Command '{}' failed: {:?}", self.command_template, String::from_utf8_lossy(&output.stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

pub struct MockAgentCall;

impl AgentCall for MockAgentCall {
    fn call_llm(&self, prompt: String) -> Result<String> {
        println!("MockAgentCall received prompt:\n{}", prompt);
        Ok("LLM Response for @answer".to_string())
    }
}
