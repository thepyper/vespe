use portable_pty::{CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::thread;
use std::time::{Duration, Instant};

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
    println!(r"CommandBuilder created for cmd.exe in H:\my\github\vespe.");

    let mut child = pair.slave.spawn_command(cmd)?;
    println!("Child process (cmd.exe) spawned.");

    // Read and write to the PTY
    let master = pair.master;
    let mut reader = master.try_clone_reader()?;
    let mut writer = master.take_writer()?;
    println!("PTY master reader and writer obtained.");

    // Give the shell some time to start up
    println!("Sleeping for 2 seconds to allow shell to start...");
    thread::sleep(Duration::from_secs(2));

    // Function to read output with a timeout
    fn read_output_with_timeout(
        reader: &mut dyn Read,
        timeout: Duration,
        label: &str,
    ) -> anyhow::Result<String> {
        let mut output = String::new();
        let mut buffer = [0; 1024];
        let start_time = Instant::now();

        println!("Attempting to read {} output for {:?}", label, timeout);

        while start_time.elapsed() < timeout {
            match reader.read(&mut buffer) {
                Ok(0) => {
                    // No data available right now, but not necessarily EOF.
                    // Continue waiting until timeout.
                    thread::sleep(Duration::from_millis(50));
                }
                Ok(n) => {
                    let s = String::from_utf8_lossy(&buffer[..n]);
                    output.push_str(&s);
                    println!("Read {} bytes for {}: {:?}", n, label, s);
                    // Reset timer if we got data, to wait for more
                    // start_time = Instant::now(); // This would require start_time to be mutable
                    // For simplicity, we'll just keep reading until the initial timeout
                    thread::sleep(Duration::from_millis(50)); // Small delay to prevent busy-waiting
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data available right now, but not necessarily EOF.
                    // Continue waiting until timeout.
                    thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    eprintln!("Error reading from PTY for {}: {}", label, e);
                    return Err(e.into());
                }
            }
        }
        println!("Finished reading {} output after {:?}", label, start_time.elapsed());
        Ok(output)
    }

    // Read initial output (e.g., shell prompt)
    let initial_output = read_output_with_timeout(&mut reader, Duration::from_secs(5), "initial")?;
    println!("Initial output:\n---\n{}\n---", initial_output);

    // Write a command to the shell
    let command_to_send = "echo Hello from PTY!\r\n";
    println!("Sending command: {:?}", command_to_send.trim());
    writer.write_all(command_to_send.as_bytes())?;
    writer.flush()?;
    println!("Command sent.");

    // Give the command some time to execute
    println!("Sleeping for 2 seconds to allow command to execute...");
    thread::sleep(Duration::from_secs(2));

    // Read the output of the command
    let command_output = read_output_with_timeout(&mut reader, Duration::from_secs(5), "command")?;
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