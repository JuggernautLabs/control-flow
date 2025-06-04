use control_flow::ticket::ProjectManager;
use tempfile::TempDir;

// Mock the TUI App state to debug what's happening
#[derive(Debug, Clone)]
pub enum AppState {
    MainMenu,
    ListProjects,
    OpenProject,
    DeleteProject,
}

struct MockApp {
    pub state: AppState,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub project_manager: ProjectManager,
    pub items: Vec<String>,
}

impl MockApp {
    pub fn new(workspace_dir: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let project_manager = ProjectManager::new(workspace_dir)?;
        
        let items = vec![
            "List projects".to_string(),
            "Create new project".to_string(),
            "Open project".to_string(),
            "Delete project".to_string(),
            "Exit".to_string(),
        ];

        Ok(MockApp {
            state: AppState::MainMenu,
            selected_index: 0,
            scroll_offset: 0,
            project_manager,
            items,
        })
    }

    fn list_projects(&mut self) {
        let projects = self.project_manager.list_projects();
        println!("🔍 list_projects called:");
        println!("  - Found {} projects: {:?}", projects.len(), projects);
        
        if projects.is_empty() {
            println!("  - Setting state to Error");
            self.state = AppState::MainMenu; // Simplified for test
        } else {
            println!("  - Setting items to projects + back button");
            self.items = projects.iter().map(|p| p.to_string()).collect();
            self.items.push("← Back".to_string());
            self.state = AppState::ListProjects;
            println!("  - New items: {:?}", self.items);
            println!("  - New state: {:?}", self.state);
        }
        self.selected_index = 0;
        self.scroll_offset = 0;
    }
}

#[test]
fn debug_tui_items_issue() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_dir = temp_dir.path().join("control-flow-projects");
    std::fs::create_dir_all(&workspace_dir).unwrap();
    
    let mut app = MockApp::new(&workspace_dir).unwrap();
    
    println!("📋 Initial state:");
    println!("  - State: {:?}", app.state);
    println!("  - Items: {:?}", app.items);
    
    // Create some projects first
    app.project_manager.create_project("project1".to_string(), "First project".to_string()).unwrap();
    app.project_manager.create_project("project2".to_string(), "Second project".to_string()).unwrap();
    
    println!("\n📋 After creating projects:");
    let projects = app.project_manager.list_projects();
    println!("  - Projects in manager: {:?}", projects);
    
    // Simulate user selecting "List projects" (index 0)
    println!("\n📋 User selects 'List projects' (index 0):");
    app.list_projects();
    
    println!("\n📋 Final state:");
    println!("  - State: {:?}", app.state);
    println!("  - Items: {:?}", app.items);
    println!("  - Selected index: {}", app.selected_index);
    
    // Verify the items are what we expect
    assert!(app.items.len() >= 2); // At least 2 projects
    assert!(app.items.contains(&"project1".to_string()));
    assert!(app.items.contains(&"project2".to_string()));
    assert!(app.items.contains(&"← Back".to_string()));
    
    println!("✅ Items are correctly populated with projects!");
}