use super::super::project::Project;
use super::super::context::{Context, LineData};
use std::path::PathBuf;
use std::fs;

#[test]
fn test_summary_tag_parsing() {
    let content = "Hello\n@summary my_summary_context\nWorld";
    let file_path = PathBuf::from("test.md");
    let lines = Context::parse(content, file_path);

    assert_eq!(lines.len(), 3);
    assert!(matches!(lines[1].data, LineData::Summary { context_name } if context_name == "my_summary_context"));
}

// This test requires a mock LLM command and a temporary project setup.
// For now, we'll keep it commented out until we have a proper testing framework.
/*
#[test]
fn test_summary_generation_and_caching() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let project_root = temp_dir.path().to_path_buf();

    // Initialize a mock project
    let project = Project::init(&project_root)?;

    // Create a context file to be summarized
    let summary_context_path = project.contexts_dir()?.join("my_summary_context.md");
    fs::write(&summary_context_path, "This is the content to be summarized.")?;

    // Create a main context file that includes the summary tag
    let main_context_path = project.contexts_dir()?.join("main_context.md");
    fs::write(&main_context_path, "@summary my_summary_context")?;

    // Compose the main context
    let composed_lines = project.compose("main_context")?;

    // Assert that the summary is present
    assert_eq!(composed_lines.len(), 1);
    assert!(matches!(composed_lines[0].data, LineData::Text(ref text) if text.contains("(LLM Summary of: This is the content to be summarized.)")));

    // Check if the summary file was created
    let summary_cache_path = project.summaries_dir()?.join("my_summary_context.md.summary");
    assert!(summary_cache_path.exists());

    // Read cached summary and verify content and hash
    let cached_data: serde_json::Value = serde_json::from_str(&fs::read_to_string(&summary_cache_path)?)?;
    assert_eq!(cached_data["summary_content"].as_str().unwrap(), "(LLM Summary of: This is the content to be summarized.)");
    // You would also assert the hash here, but it requires calculating the hash of the original content.

    Ok(())
}
*/
