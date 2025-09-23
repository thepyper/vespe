use vespe::cli::commands::{Cli, Commands};
// use vespe::project_root::{self, is_project_root}; // Commented out
// use vespe::statistics; // Commented out
use project::utils::{find_project_root, is_project_root}; // New import

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let current_dir = std::env::current_dir()?;
    let current_project_root = find_project_root(&current_dir); // Use new find_project_root

    // Handle Init command separately, as it doesn't require an existing project root
    // This block is commented out as Init command is now handled by vespe_cli
    /*
    if let Commands::Init { path } = &cli.command {
        let target_dir = if let Some(p) = path {
            p.clone()
        } else {
            current_dir.clone()
        };
        project_root::initialize_project_root(&target_dir)?;
        println!("Vespe project initialized at: {}", target_dir.display());
        return Ok(());
    }
    */

    let project_root = if let Some(path) = cli.project_root {
        path
    } else if let Some(root) = current_project_root {
        root
    } else {
        return Err(anyhow!("Project root not found. Please run 'vespe init' or specify --project-root."));
    };
    
    // Check if it actually is a root
    if !is_project_root(&project_root) { // Use new is_project_root
        return Err(anyhow!("Not a vespe project."));
    }

    // Initialize logging
    let log_dir = project_root.join(".vespe").join("log");
    std::fs::create_dir_all(&log_dir)?;

    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "vespe.log");
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(non_blocking_appender)
        .with_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()));

    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()));

    Registry::default()
        .with(file_layer)
        .with(stdout_layer)
        .init();

    info!("Project root at {}", project_root.display());

    // // Load statistics (Commented out)
    // let stats = Arc::new(Mutex::new(statistics::load_statistics(&project_root).await?));

    // Call the main run function from the library
    let result = vespe::run(project_root.clone(), cli.command /*, stats.clone()*/).await; // stats argument commented out

    // // Save statistics before exiting (Commented out)
    // statistics::save_statistics(&project_root, &*stats.lock().await).await?;

    result
}