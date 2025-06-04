use control_flow::ticket::ProjectManager;
use tempfile::TempDir;
use std::path::PathBuf;

#[test]
fn test_project_manager_functionality() {
    let temp_dir = TempDir::new().unwrap();
    
    // Test the exact same flow that TUI uses
    let workspace_dir = temp_dir.path().join("control-flow-projects");
    println!("Using workspace directory: {:?}", workspace_dir);
    
    // Step 1: Create project manager (like TUI App::new)
    let mut project_manager = match ProjectManager::load_index(&workspace_dir) {
        Ok(manager) => {
            println!("Loaded existing project manager");
            manager
        },
        Err(e) => {
            println!("Could not load existing manager: {}, creating new one", e);
            ProjectManager::new(&workspace_dir).unwrap()
        }
    };
    
    println!("Initial project count: {}", project_manager.list_projects().len());
    
    // Step 2: Create a project (like TUI create_project flow)
    let result = project_manager.create_project("debug_test_project".to_string(), "A test project for debugging".to_string());
    match result {
        Ok(_) => println!("✅ Project created successfully"),
        Err(e) => println!("❌ Failed to create project: {}", e),
    }
    
    // Step 3: List projects (like TUI list_projects)
    let projects = project_manager.list_projects();
    println!("Projects after creation: {:?}", projects);
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0], "debug_test_project");
    
    // Step 4: Try to load the project (like TUI load_project)
    let load_result = project_manager.load_project("debug_test_project");
    match load_result {
        Ok(project) => {
            println!("✅ Project loaded successfully: {}", project.name);
            assert_eq!(project.name, "debug_test_project");
            assert_eq!(project.description, "A test project for debugging");
        },
        Err(e) => {
            println!("❌ Failed to load project: {}", e);
            panic!("Should be able to load the project we just created");
        }
    }
    
    // Step 5: Check filesystem
    let project_file = workspace_dir.join("debug_test_project.json");
    println!("Project file exists: {}", project_file.exists());
    assert!(project_file.exists());
    
    let index_file = workspace_dir.join("projects.json");
    println!("Index file exists: {}", index_file.exists());
    assert!(index_file.exists());
    
    if index_file.exists() {
        let index_content = std::fs::read_to_string(&index_file).unwrap();
        println!("Index file content: {}", index_content);
    }
    
    if project_file.exists() {
        let project_content = std::fs::read_to_string(&project_file).unwrap();
        println!("Project file content (first 200 chars): {}", &project_content[..project_content.len().min(200)]);
    }
}

#[test] 
fn test_tui_workspace_path() {
    // Test the exact workspace path that TUI uses
    let tui_workspace = PathBuf::from("./control-flow-projects");
    println!("TUI workspace path: {:?}", tui_workspace);
    println!("TUI workspace absolute: {:?}", std::fs::canonicalize(&tui_workspace).unwrap_or(tui_workspace.clone()));
    
    // Create it like TUI does
    std::fs::create_dir_all(&tui_workspace).unwrap();
    
    let mut manager = ProjectManager::new(&tui_workspace).unwrap();
    manager.create_project("tui_test".to_string(), "TUI test project".to_string()).unwrap();
    
    let projects = manager.list_projects();
    println!("Projects in TUI workspace: {:?}", projects);
    assert_eq!(projects.len(), 1);
    
    // Check if files exist in the expected location
    let project_file = tui_workspace.join("tui_test.json");
    let index_file = tui_workspace.join("projects.json");
    
    println!("TUI project file exists: {}", project_file.exists());
    println!("TUI index file exists: {}", index_file.exists());
    
    assert!(project_file.exists());
    assert!(index_file.exists());
}

#[test]
fn test_project_filename_sanitization_debug() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = ProjectManager::new(temp_dir.path()).unwrap();
    
    // Test project names with spaces (common in TUI)
    let test_name = "my todo app";
    println!("Creating project with name: '{}'", test_name);
    
    let result = manager.create_project(test_name.to_string(), "Test description".to_string());
    assert!(result.is_ok(), "Failed to create project: {:?}", result.err());
    
    let projects = manager.list_projects();
    println!("Projects after creation: {:?}", projects);
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0], test_name); // Display name should be unchanged
    
    // Check what file was actually created
    let expected_filename = test_name.replace(' ', "_") + ".json";
    let expected_path = temp_dir.path().join(&expected_filename);
    println!("Expected file path: {:?}", expected_path);
    println!("File exists: {}", expected_path.exists());
    assert!(expected_path.exists());
    
    // Verify we can load it back
    let loaded_project = manager.load_project(test_name).unwrap();
    assert_eq!(loaded_project.name, test_name);
}