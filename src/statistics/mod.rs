use anyhow::Result;
use std::path::{Path, PathBuf};
use tokio::fs;
use crate::statistics::models::UsageStatistics;

const STATS_FILE_NAME: &str = "statistics.json";

pub async fn load_statistics(project_root: &Path) -> Result<UsageStatistics> {
    let stats_path = project_root.join(".vespe").join(STATS_FILE_NAME);
    if stats_path.exists() {
        let content = fs::read_to_string(&stats_path).await?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(UsageStatistics::default())
    }
}

pub async fn save_statistics(project_root: &Path, stats: &UsageStatistics) -> Result<()> {
    let stats_path = project_root.join(".vespe").join(STATS_FILE_NAME);
    let content = serde_json::to_string_pretty(stats)?;
    fs::write(&stats_path, content).await?;
    Ok(())
}
