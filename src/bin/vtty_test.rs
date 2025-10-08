use portable_pty::{CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    // Create a new pseudo-terminal
    let pty_system = portable_pty::native_pty_system();

    let pair = pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    // Spawn a command in the pseudo-terminal
    let mut cmd = CommandBuilder::new("cmd.exe"); // Use "bash" or "sh" on Unix-like systems
    cmd.cwd("H:\\my\\github\\vespe"); // Set the working directory

    let mut child = pair.slave.spawn_command(cmd)?;

    // Read and write to the PTY
    let master = pair.master;
    let mut reader = master.try_clone_reader()?;
    let mut writer = master.take_writer()?;

    // Give the shell some time to start up
    thread::sleep(Duration::from_millis(500));

    // Read initial output (e.g., shell prompt)
    let mut output = String::new();
    reader.read_to_string(&mut output)?;
    println!("Initial output:\n{}", output);

    // Write a command to the shell
    let command_to_send = "echo Hello from PTY!\r\n";
    writer.write_all(command_to_send.as_bytes())?;
    writer.flush()?;
    println!("Sent command: {}", command_to_send.trim());

    // Give the command some time to execute
    thread::sleep(Duration::from_millis(500));

    // Read the output of the command
    output.clear();
    reader.read_to_string(&mut output)?;
    println!("Output after command:\n{}", output);

    // Send an exit command
    writer.write_all(b"exit\r\n")?;
    writer.flush()?;

    // Wait for the child process to exit
    let exit_status = child.wait()?;
    println!("Child process exited with: {:?}", exit_status);

    Ok(())
}
