use control_flow::ticket::ProjectManager;
use tempfile::TempDir;

#[test]
fn test_fresh_project_immediately_visible() {
    println!("🧪 Testing fresh project creation and immediate visibility...");
    
    // Create completely fresh environment
    let temp_dir = TempDir::new().unwrap();
    let workspace_dir = temp_dir.path().join("control-flow-projects");
    
    // Step 1: Initialize like TUI does
    let mut project_manager = match ProjectManager::load_index(&workspace_dir) {
        Ok(manager) => {
            println!("📂 Loaded existing project manager");
            manager
        },
        Err(_) => {
            println!("📂 Created new project manager");
            ProjectManager::new(&workspace_dir).unwrap()
        }
    };
    
    // Step 2: Verify no projects initially
    let initial_projects = project_manager.list_projects();
    println!("📋 Initial projects: {:?}", initial_projects);
    assert_eq!(initial_projects.len(), 0);
    
    // Step 3: Create a project (like TUI create workflow)
    println!("📝 Creating new project 'my_fresh_project'...");
    project_manager.create_project(
        "my_fresh_project".to_string(), 
        "A freshly created project".to_string()
    ).unwrap();
    
    // Step 4: Immediately check if it's visible (like TUI list_projects)
    let projects_after_creation = project_manager.list_projects();
    println!("📋 Projects after creation: {:?}", projects_after_creation);
    
    assert_eq!(projects_after_creation.len(), 1);
    assert_eq!(projects_after_creation[0], "my_fresh_project");
    
    // Step 5: Verify we can load it
    let loaded_project = project_manager.load_project("my_fresh_project").unwrap();
    println!("✅ Successfully loaded project: {}", loaded_project.name);
    assert_eq!(loaded_project.name, "my_fresh_project");
    assert_eq!(loaded_project.description, "A freshly created project");
    
    // Step 6: Simulate the exact items list that TUI would create
    let mut tui_items: Vec<String> = projects_after_creation.iter().map(|p| p.to_string()).collect();
    tui_items.push("← Back".to_string());
    
    println!("🖥️ TUI items that should appear:");
    for (i, item) in tui_items.iter().enumerate() {
        println!("   {}. {}", i, item);
    }
    
    assert_eq!(tui_items, vec!["my_fresh_project", "← Back"]);
    
    println!("🎉 Fresh project is immediately visible after creation!");
}