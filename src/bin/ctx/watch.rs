use anyhow::Result;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;
use tracing::{debug, error, info};

pub fn watch(project: &Project, agent: &ShellAgentCall) -> Result<()> {
    println!("Starting watch mode...");

    // Initial execution on all context files
    initial_execute_all_contexts(project, agent)?;

    // Setup file watcher
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;
    let contexts_dir = project.path_res.contexts_root();
    watcher.watch(contexts_dir.as_ref(), RecursiveMode::Recursive)?;

    println!("Watching for changes in: {}", contexts_dir.display());
    println!("Press Ctrl-C to stop.");

    // Handle Ctrl-C
    let (ctrlc_tx, ctrlc_rx) = channel();
    ctrlc::set_handler(move || {
        ctrlc_tx
            .send(())
            .expect("Could not send signal on channel.")
    })
    .expect("Error setting Ctrl-C handler");

    loop {
        // Check for Ctrl-C
        if ctrlc_rx.try_recv().is_ok() {
            println!("Ctrl-C received. Stopping watch mode.");
            break;
        }

        match rx.recv_timeout(Duration::from_millis(100)) {
            // Poll for events
            Ok(event_result) => match event_result {
                Ok(event) => {
                    for path in event.paths {
                        if is_context_file(project, &path) {
                            let context_name = path_to_context_name(project, &path)?;
                            println!(
                                "Change detected in context file: {}. Re-executing...",
                                context_name
                            );
                            if let Err(e) = project.execute_context(&context_name, agent) {
                                eprintln!("Error executing context {}: {}", context_name, e);
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Watch error: {:?}", e),
            },
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // No event, continue loop
            }
            Err(e) => {
                eprintln!("Watch channel error: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}

fn initial_execute_all_contexts(project: &Project, agent: &ShellAgentCall) -> Result<()> {
    println!("Performing initial execution on all context files...");
    let contexts = project.list_contexts()?;
    for context in contexts {
        println!("  Executing initial context: {}", context.name);
        if let Err(e) = project.execute_context(&context.name, agent) {
            eprintln!("Error executing initial context {}: {}", context.name, e);
        }
    }
    Ok(())
}

fn is_context_file(project: &Project, path: &Path) -> bool {
    path.extension().map_or(false, |ext| ext == "md") && path.starts_with(project.contexts_root())
}

fn path_to_context_name(project: &Project, path: &Path) -> Result<String> {
    let relative_path = path.strip_prefix(project.contexts_root())?;
    let context_name = relative_path
        .with_extension("")
        .to_string_lossy()
        .to_string();
    Ok(context_name)
}
