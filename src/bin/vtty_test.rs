use anyhow::{Context, Result};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{self, Read, Write};
use std::sync::mpsc;
use std::thread;

fn main() -> Result<()> {
    // Use the native pty implementation for the system
    let pty_system = native_pty_system();

    // Create a new pty
    let pty_pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .context("Failed to open PTY")?;

    // Spawn a Windows shell (e.g., cmd.exe or powershell.exe) into the pty
    // On Windows, you might need to specify the full path or ensure it's in PATH.
    // For cmd.exe, it's usually in C:\Windows\System32\cmd.exe
    // For powershell.exe, it's usually in C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe
    let mut cmd = CommandBuilder::new("powershell.exe");
    cmd.arg("-Command");
    cmd.arg("Write-Host 'Hello from PTY child process!'; exit;");
    // If you want to use PowerShell, uncomment the line below and comment out the cmd.exe line
    // let mut cmd = CommandBuilder::new("powershell.exe");
    cmd.cwd(r"H:\my\github\vespe"); // Set the working directory

    let mut child = pty_pair
        .slave
        .spawn_command(cmd)
        .context("Failed to spawn command in PTY")?;

    // Drop the slave PTY handle in the main thread as it's owned by the child process
    drop(pty_pair.slave);

    // Create channels for communication between threads
    let (tx_input, rx_input) = mpsc::channel(); // For sending input to the PTY
    let (tx_output, rx_output) = mpsc::channel(); // For receiving output from the PTY

    // Get reader and writer for the master PTY
    let mut master_reader = pty_pair
        .master
        .try_clone_reader()
        .context("Failed to clone PTY master reader")?;
    let mut master_writer = pty_pair
        .master
        .take_writer()
        .context("Failed to take PTY master writer")?;

    // Spawn a thread to read PTY output and send it to the main thread
    thread::spawn(move || {
        let mut buffer = [0u8; 1024];
        loop {
            match master_reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    if tx_output.send(buffer[..n].to_vec()).is_err() {
                        break; // Receiver disconnected
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from PTY: {}", e);
                    break;
                }
            }
        }
    });

    // Spawn a thread to write input from the main thread to the PTY
    thread::spawn(move || {
        loop {
            match rx_input.recv() {
                Ok(data) => {
                    let data: Vec<u8> = data;
                    if let Err(e) = master_writer.write_all(&data) {
                        eprintln!("Error writing to PTY: {}", e);
                        break;
                    }
                    if let Err(e) = master_writer.flush() {
                        eprintln!("Error flushing PTY writer: {}", e);
                        break;
                    }
                }
                Err(_) => break, // Sender disconnected
            }
        }
    });

    println!("Interactive shell started. Type 'exit' to quit.");

    // Main loop: read user input and send to PTY, print PTY output
    loop {
        // Read output from the PTY and print it
        while let Ok(output_data) = rx_output.try_recv() {
            io::stdout()
                .write_all(&output_data)
                .context("Failed to write PTY output to stdout")?;
            io::stdout().flush().context("Failed to flush stdout")?;
        }

        // Read user input from stdin
        let mut input_line = String::new();
        io::stdin()
            .read_line(&mut input_line)
            .context("Failed to read line from stdin")?;

        // Check for exit command
        if input_line.trim().eq_ignore_ascii_case("exit") {
            break;
        }

        // Send user input to the PTY
        tx_input
            .send(input_line.as_bytes().to_vec())
            .context("Failed to send input to PTY thread")?;
    }

    // Wait for the child process to exit
    let exit_status = child.wait().context("Failed to wait for child process")?;
    println!("Shell exited with status: {:?}", exit_status);

    Ok(())
}
