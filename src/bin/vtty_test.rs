use portable_pty::{CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    println!("Starting vtty_test...");

    // Create a new pseudo-terminal
    let pty_system = portable_pty::native_pty_system();
    println!("PTY system initialized.");

    let pair = pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    })?;
    println!("PTY pair opened.");

    // Spawn a command in the pseudo-terminal
    let mut cmd = CommandBuilder::new("cmd.exe"); // Use "bash" or "sh" on Unix-like systems
    cmd.cwd("H:\\my\\github\\vespe"); // Set the working directory
    println!("CommandBuilder created for cmd.exe in H:\my\github\vespe.");

    let mut child = pair.slave.spawn_command(cmd)?;
    println!("Child process (cmd.exe) spawned.");

    // Read and write to the PTY
    let master = pair.master;
    let mut reader = master.try_clone_reader()?;
    let mut writer = master.take_writer()?;
    println!("PTY master reader and writer obtained.");

    // Give the shell some time to start up
    println!("Sleeping for 1 second to allow shell to start...");
    thread::sleep(Duration::from_secs(1));

    // Read initial output (e.g., shell prompt)
    let mut buffer = [0; 1024];
    let mut initial_output = String::new();
    println!("Attempting to read initial output...");
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => {
                println!("Reader returned 0 bytes, likely EOF or no more data for now.");
                break; // No more data or EOF
            }
            Ok(n) => {
                let s = String::from_utf8_lossy(&buffer[..n]);
                initial_output.push_str(&s);
                println!("Read {} bytes: {:?}", n, s);
                // Give a small moment for more data to arrive, but don't block indefinitely
                thread::sleep(Duration::from_millis(50));
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                println!("Read would block, no more data for now.");
                break; // No more data available immediately
            }
            Err(e) => {
                eprintln!("Error reading from PTY: {}", e);
                return Err(e.into());
            }
        }
        // Add a timeout or a condition to break the loop if no data for a while
        // For simplicity, we'll just break after a few reads if it would block
        // or if we've read some initial data.
        if initial_output.len() > 0 && initial_output.ends_with('>') { // Heuristic for cmd.exe prompt
            println!("Detected cmd.exe prompt, stopping initial read.");
            break;
        }
        thread::sleep(Duration::from_millis(100)); // Small delay to prevent busy-waiting
    }
    println!("Initial output:\n---\n{}\n---", initial_output);

    // Write a command to the shell
    let command_to_send = "echo Hello from PTY!\r\n";
    println!("Sending command: {:?}", command_to_send.trim());
    writer.write_all(command_to_send.as_bytes())?;
    writer.flush()?;
    println!("Command sent.");

    // Give the command some time to execute
    println!("Sleeping for 1 second to allow command to execute...");
    thread::sleep(Duration::from_secs(1));

    // Read the output of the command
    let mut command_output = String::new();
    println!("Attempting to read command output...");
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => {
                println!("Reader returned 0 bytes, likely EOF or no more data for now.");
                break;
            }
            Ok(n) => {
                let s = String::from_utf8_lossy(&buffer[..n]);
                command_output.push_str(&s);
                println!("Read {} bytes: {:?}", n, s);
                thread::sleep(Duration::from_millis(50));
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                println!("Read would block, no more data for now.");
                break;
            }
            Err(e) => {
                eprintln!("Error reading from PTY: {}", e);
                return Err(e.into());
            }
        }
        if command_output.len() > 0 && command_output.ends_with('>') { // Heuristic for cmd.exe prompt
            println!("Detected cmd.exe prompt, stopping command output read.");
            break;
        }
        thread::sleep(Duration::from_millis(100)); // Small delay to prevent busy-waiting
    }
    println!("Output after command:\n---\n{}\n---", command_output);

    // Send an exit command
    println!("Sending 'exit' command...");
    writer.write_all(b"exit\r\n")?;
    writer.flush()?;
    println!("'exit' command sent.");

    // Wait for the child process to exit
    println!("Waiting for child process to exit...");
    let exit_status = child.wait()?;
    println!("Child process exited with: {:?}", exit_status);

    Ok(())
}
