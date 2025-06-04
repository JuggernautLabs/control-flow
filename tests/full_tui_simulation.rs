use control_flow::ticket::ProjectManager;
use std::path::PathBuf;

// Simulate the exact TUI App behavior
#[derive(Debug, Clone)]
pub enum AppState {
    MainMenu,
    ListProjects,
    OpenProject,
    DeleteProject,
}

struct TuiSimulation {
    pub state: AppState,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub project_manager: ProjectManager,
    pub items: Vec<String>,
}

impl TuiSimulation {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let workspace_dir = PathBuf::from("./control-flow-projects");
        let project_manager = match ProjectManager::load_index(&workspace_dir) {
            Ok(manager) => manager,
            Err(_) => ProjectManager::new(&workspace_dir)?,
        };

        let items = vec![
            "List projects".to_string(),
            "Create new project".to_string(),
            "Open project".to_string(),
            "Delete project".to_string(),
            "Exit".to_string(),
        ];

        Ok(TuiSimulation {
            state: AppState::MainMenu,
            selected_index: 0,
            scroll_offset: 0,
            project_manager,
            items,
        })
    }

    // Exactly like the real TUI
    fn list_projects(&mut self) {
        let projects = self.project_manager.list_projects();
        if projects.is_empty() {
            println!("❌ No projects found - would show error state");
        } else {
            self.items = projects.iter().map(|p| p.to_string()).collect();
            self.items.push("← Back".to_string());
            self.state = AppState::ListProjects;
            println!("✅ Found {} projects, state changed to ListProjects", projects.len());
            println!("   Items: {:?}", self.items);
        }
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    fn handle_main_menu_selection(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.selected_index {
            0 => {
                println!("📋 User selected 'List projects' (index 0)");
                self.list_projects();
            },
            1 => println!("📝 User selected 'Create new project' (index 1)"),
            2 => println!("📂 User selected 'Open project' (index 2)"),
            3 => println!("🗑️ User selected 'Delete project' (index 3)"),
            4 => println!("🚪 User selected 'Exit' (index 4)"),
            _ => {}
        }
        Ok(())
    }

    fn simulate_key_press(&mut self, action: &str) -> Result<(), Box<dyn std::error::Error>> {
        match action {
            "enter" => {
                match &self.state {
                    AppState::MainMenu => self.handle_main_menu_selection()?,
                    AppState::ListProjects => {
                        println!("🔙 In ListProjects state, Enter pressed on index {}", self.selected_index);
                        if self.selected_index == self.items.len() - 1 {
                            println!("   Selected '← Back' - returning to main menu");
                            self.state = AppState::MainMenu;
                            self.items = vec![
                                "List projects".to_string(),
                                "Create new project".to_string(),
                                "Open project".to_string(),
                                "Delete project".to_string(),
                                "Exit".to_string(),
                            ];
                            self.selected_index = 0;
                        } else {
                            println!("   Selected project: {} - staying in list view", self.items[self.selected_index]);
                        }
                    },
                    _ => {}
                }
            },
            "down" => {
                if self.selected_index < self.items.len().saturating_sub(1) {
                    self.selected_index += 1;
                    println!("⬇️ Moved down to index {}", self.selected_index);
                }
            },
            "up" => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    println!("⬆️ Moved up to index {}", self.selected_index);
                }
            },
            _ => {}
        }
        Ok(())
    }

    fn print_current_screen(&self) {
        println!("\n🖥️ CURRENT SCREEN:");
        println!("   State: {:?}", self.state);
        println!("   Selected: {} of {}", self.selected_index, self.items.len());
        for (i, item) in self.items.iter().enumerate() {
            let marker = if i == self.selected_index { "► " } else { "  " };
            println!("   {}{}", marker, item);
        }
        println!("");
    }
}

#[test]
fn simulate_full_tui_interaction() {
    println!("🚀 Starting TUI simulation with real workspace...");
    
    let mut tui = TuiSimulation::new().unwrap();
    
    // Show initial screen
    println!("\n📱 === INITIAL MAIN MENU ===");
    tui.print_current_screen();
    
    // User presses Enter on "List projects"
    println!("👆 User presses Enter on 'List projects'...");
    tui.simulate_key_press("enter").unwrap();
    
    // Show what should appear
    println!("\n📱 === AFTER PRESSING ENTER ON 'LIST PROJECTS' ===");
    tui.print_current_screen();
    
    // Try navigating in the project list
    if matches!(tui.state, AppState::ListProjects) && tui.items.len() > 1 {
        println!("👆 User navigates down...");
        tui.simulate_key_press("down").unwrap();
        tui.print_current_screen();
        
        println!("👆 User presses Enter on selected project...");
        tui.simulate_key_press("enter").unwrap();
    }
    
    println!("✅ TUI simulation complete!");
}