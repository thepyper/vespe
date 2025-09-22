use chrono::Utc;

use project::api;
use project::models::PersistentEvent;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Task API CLI Mock...");

    // Define a temporary base path for tasks
    let temp_dir = tempfile::tempdir()?;
    let base_path = temp_dir.path().join(".vespe").join("tasks");
    std::fs::create_dir_all(&base_path)?;

    println!("Using base path: {}", base_path.display());

    // 1. Create a new task
    println!("\n--- Creating Task ---");
    let task_name = "Implement User Authentication".to_string();
    let created_by = "cli_user".to_string();
    let template = "default".to_string();

    let task = api::create_task(&base_path, None, task_name.clone(), created_by.clone(), template.clone())?;
    println!("Created task: {} (UID: {})", task.config.name, task.uid);
    println!("Initial state: {:?}", task.status.current_state);

    // 2. Define Objective
    println!("\n--- Defining Objective ---");
    let objective_content = "The goal is to implement a secure user authentication system with OAuth2 support.".to_string();
    let task = api::define_objective(&base_path, &task.uid, objective_content.clone())?;
    println!("Objective defined for task {}. New state: {:?}", task.uid, task.status.current_state);

    // 3. Define Plan
    println!("\n--- Defining Plan ---");
    let plan_content = "1. Research OAuth2 providers. 2. Design database schema. 3. Implement API endpoints. 4. Write unit tests.".to_string();
    let task = api::define_plan(&base_path, &task.uid, plan_content.clone())?;
    println!("Plan defined for task {}. New state: {:?}", task.uid, task.status.current_state);

    // 4. Add Persistent Events
    println!("\n--- Adding Persistent Events ---");
    let event1 = PersistentEvent {
        timestamp: Utc::now(),
        event_type: "llm_thought".to_string(),
        agent_id: "manager_agent".to_string(),
        content: "Considering initial steps for auth implementation.".to_string(),
    };
    api::add_persistent_event(&base_path, &task.uid, event1.clone())?;
    println!("Added persistent event: {:?}", event1.event_type);

    let event2 = PersistentEvent {
        timestamp: Utc::now() + chrono::Duration::seconds(1),
        event_type: "tool_output".to_string(),
        agent_id: "dev_agent".to_string(),
        content: "OAuth2 research complete. Recommended providers: Google, GitHub.".to_string(),
    };
    api::add_persistent_event(&base_path, &task.uid, event2.clone())?;
    println!("Added persistent event: {:?}", event2.event_type);

    // 5. Add Result Files
    println!("\n--- Adding Result Files ---");
    api::add_result_file(&base_path, &task.uid, "design_doc.md", "# Auth Design\n\n... ".as_bytes().to_vec())?;
    println!("Added design_doc.md to result/.");

    api::add_result_file(&base_path, &task.uid, "api_spec.json", "{\"endpoints\":[]}".as_bytes().to_vec())?;
    println!("Added api_spec.json to result/.");

    // 6. Calculate Result Hash
    println!("\n--- Calculating Result Hash ---");
    let result_hash = api::calculate_result_hash(&base_path, &task.uid)?;
    println!("Result hash for task {}: {}", task.uid, result_hash);

    // 7. Load Task and Verify State
    println!("\n--- Loading Task and Verifying ---");
    let loaded_task = api::load_task(&base_path, &task.uid)?;
    println!("Loaded task: {} (UID: {})", loaded_task.config.name, loaded_task.uid);
    println!("Current state: {:?}", loaded_task.status.current_state);
    println!("Objective: {}", loaded_task.objective);
    println!("Plan: {:?}", loaded_task.plan);

    let all_events = api::get_all_persistent_events(&base_path, &task.uid)?;
    println!("Total persistent events: {}", all_events.len());

    println!("\nTask API CLI Mock finished successfully.");

    Ok(())
}