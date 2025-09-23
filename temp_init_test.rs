use project::utils::initialize_project_root;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let target_dir = PathBuf::from("temp_test_dir");
    println!("Calling initialize_project_root for: {}", target_dir.display());
    initialize_project_root(&target_dir)?;
    println!("Initialization successful.");
    Ok(())
}
