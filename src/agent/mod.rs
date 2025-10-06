use anyhow::Context;
use std::process::{Command, Stdio};

pub struct ShellAgentCall {
    command_template: String,
}

impl ShellAgentCall {
    pub fn new(command: String) -> Self {
        Self {
            command_template: command,
        }
    }

    pub fn call(&self, query: &str) -> anyhow::Result<String> {
        let mut command_parts = self.command_template.split_whitespace();
        let _original_program = command_parts
            .next()
            .context("Command template cannot be empty")?;
        let initial_args: Vec<&str> = command_parts.collect();

        let program = "npx";
        let mut final_args = vec!["gemini"];
        final_args.extend(initial_args);

        let mut command = {
            #[cfg(windows)]
            {
                let mut cmd = Command::new(program);
                cmd.args(&final_args);
                cmd.arg("--prompt").arg(query);
                cmd
            }
            #[cfg(not(windows))]
            {
                let mut cmd = Command::new(program);
                cmd.args(&final_args);
                cmd.arg("--prompt").arg(query);
                cmd
            }
        };

        command.stdout(Stdio::piped()).stderr(Stdio::piped());

        let child = command.spawn().with_context(|| {
            format!(
                "Failed to spawn command: '{}'. Is it in your PATH?",
                self.command_template
            )
        })?;

        let output = child.wait_with_output()?;

        if !output.status.success() {
            anyhow::bail!(
                "Command '{}' failed: {:?}",
                self.command_template,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
