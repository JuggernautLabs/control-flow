use control_flow::ticket::ProjectManager;
use tempfile::TempDir;

#[test]
fn test_project_creation_integration() {
    // Create a temporary workspace
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path();
    
    // Test the exact same code path that CLI uses
    let mut project_manager = match ProjectManager::load_index(&workspace_path) {
        Ok(manager) => manager,
        Err(_) => ProjectManager::new(&workspace_path).unwrap()
    };
    
    // Test creating a project with spaces (the failing case)
    let result = project_manager.create_project("todo app".to_string(), "making a todo app".to_string());
    
    match result {
        Ok(_) => {
            // Verify the project was created correctly
            let projects = project_manager.list_projects();
            assert_eq!(projects.len(), 1);
            assert_eq!(projects[0], "todo app");
            
            // Verify we can load the project
            let project = project_manager.load_project("todo app").unwrap();
            assert_eq!(project.name, "todo app");
            assert_eq!(project.description, "making a todo app");
            
            // Verify the files exist on disk
            assert!(workspace_path.join("todo_app.json").exists());
            assert!(workspace_path.join("projects.json").exists());
            
            println!("✅ Project creation test passed!");
        }
        Err(e) => {
            panic!("Project creation failed: {}", e);
        }
    }
}

#[test]
fn test_workspace_directory_creation() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path().join("non_existent_dir").join("control-flow-projects");
    
    // Workspace doesn't exist yet
    assert!(!workspace_path.exists());
    
    // This should create the workspace directory
    let mut manager = ProjectManager::new(&workspace_path).unwrap();
    
    // Verify workspace was created
    assert!(workspace_path.exists());
    
    // Test creating a project
    let result = manager.create_project("test project".to_string(), "test description".to_string());
    assert!(result.is_ok(), "Failed to create project: {:?}", result.err());
    
    // Verify project file exists
    assert!(workspace_path.join("test_project.json").exists());
}

#[test]
fn test_project_filename_sanitization() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = ProjectManager::new(temp_dir.path()).unwrap();
    
    // Test various problematic characters
    let test_cases = vec![
        ("project with spaces", "project_with_spaces.json"),
        ("project/with/slashes", "project_with_slashes.json"),
        ("project with/mixed problems", "project_with_mixed_problems.json"),
    ];
    
    for (project_name, expected_filename) in test_cases {
        let result = manager.create_project(project_name.to_string(), "test description".to_string());
        assert!(result.is_ok(), "Failed to create project '{}': {:?}", project_name, result.err());
        
        let expected_path = temp_dir.path().join(expected_filename);
        assert!(expected_path.exists(), "Expected file '{}' was not created", expected_filename);
        
        // Verify we can load the project back
        let loaded_project = manager.load_project(project_name).unwrap();
        assert_eq!(loaded_project.name, project_name);
    }
}

#[test]
fn test_multiple_projects() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = ProjectManager::new(temp_dir.path()).unwrap();
    
    // Create multiple projects
    let projects = vec![
        ("todo app", "A todo application"),
        ("marketing strategy", "Marketing plan for Q1"),
        ("website redesign", "New company website"),
    ];
    
    for (name, desc) in &projects {
        let result = manager.create_project(name.to_string(), desc.to_string());
        assert!(result.is_ok(), "Failed to create project '{}': {:?}", name, result.err());
    }
    
    // Verify all projects exist
    let project_list = manager.list_projects();
    assert_eq!(project_list.len(), 3);
    
    for (name, desc) in &projects {
        let name_string = name.to_string();
        assert!(project_list.contains(&&name_string));
        let project = manager.load_project(name).unwrap();
        assert_eq!(project.name, *name);
        assert_eq!(project.description, *desc);
    }
}

#[test]
fn test_refinement_request_integration() {
    use control_flow::ticket::{RefinementRequest, RefinementPriority};
    
    // Test structured refinement request creation
    let refinement = RefinementRequest {
        term: "responsive design".to_string(),
        context: "create a responsive design for mobile".to_string(),
        reason: "needs clarification on breakpoints and devices".to_string(),
        priority: RefinementPriority::High,
    };
    
    // Test serialization
    let json = serde_json::to_string(&refinement).unwrap();
    assert!(json.contains("responsive design"));
    assert!(json.contains("HIGH"));
    
    // Test deserialization back
    let deserialized: RefinementRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.term, "responsive design");
    assert_eq!(deserialized.context, "create a responsive design for mobile");
    
    // Test legacy string format compatibility
    let legacy_json = r#""mobile optimization""#;
    let legacy_refinement: RefinementRequest = serde_json::from_str(legacy_json).unwrap();
    assert_eq!(legacy_refinement.term, "mobile optimization");
    assert_eq!(legacy_refinement.context, "Legacy format - context not specified");
}