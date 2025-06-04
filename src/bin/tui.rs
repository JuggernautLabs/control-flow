use control_flow::ticket::{ProjectManager, Project, TicketId};
use control_flow::ticket_service::TicketService;
use client_implementations::claude::ClaudeClient;
use client_implementations::client::RetryConfig;
use std::env;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;

#[derive(Debug, Clone)]
pub enum AppState {
    MainMenu,
    ListProjects,    // viewing project list
    OpenProject,     // selecting project to open
    DeleteProject,   // selecting project to delete
    ProjectMenu(String), // project name
    TicketList(String),  // project name
    TicketDetail(String, TicketId), // project name, ticket id - menu view
    TicketDetailsView(String, TicketId), // project name, ticket id - detailed content view
    CreateProject,
    CreateTicket(String), // project name
    QuickRefine(String, TicketId), // project name, ticket id
    Input(InputState),
    Loading(String), // loading message
    Error(String),   // error message
}

#[derive(Debug, Clone)]
pub struct InputState {
    pub title: String,
    pub prompt: String,
    pub input: String,
    pub return_state: Box<AppState>,
}

pub struct App {
    pub state: AppState,
    pub previous_state: Option<AppState>, // Track previous state for context preservation
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub project_manager: ProjectManager,
    pub current_project: Option<Project>,
    pub ticket_service: TicketService<ClaudeClient>,
    pub items: Vec<String>, // Current menu items
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();
        
        let api_key = env::var("ANTHROPIC_API_KEY")
            .expect("ANTHROPIC_API_KEY environment variable must be set");
        
        let claude_client = ClaudeClient::new(api_key);
        let retry_config = RetryConfig::default();
        let ticket_service = TicketService::new(claude_client, retry_config);
        
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

        Ok(App {
            state: AppState::MainMenu,
            previous_state: None,
            selected_index: 0,
            scroll_offset: 0,
            project_manager,
            current_project: None,
            ticket_service,
            items,
            should_quit: false,
        })
    }

    // Helper method to transition states while preserving context
    fn transition_to_state(&mut self, new_state: AppState) {
        // Only preserve context for meaningful states (not Loading/Error)
        if !matches!(self.state, AppState::Loading(_) | AppState::Error(_)) {
            self.previous_state = Some(self.state.clone());
        }
        self.state = new_state;
    }
    
    // Helper method to return to previous context or fallback
    fn return_to_context(&mut self) {
        if let Some(previous) = self.previous_state.take() {
            match &previous {
                AppState::TicketDetail(project_name, ticket_id) => {
                    let project_name = project_name.clone();
                    let ticket_id = ticket_id.clone();
                    self.show_ticket_detail(project_name, ticket_id).ok();
                },
                AppState::TicketDetailsView(project_name, ticket_id) => {
                    let project_name = project_name.clone();
                    let ticket_id = ticket_id.clone();
                    self.show_ticket_details_view(project_name, ticket_id).ok();
                },
                AppState::TicketList(project_name) => {
                    let project_name = project_name.clone();
                    self.show_ticket_list(project_name).ok();
                },
                AppState::ProjectMenu(project_name) => {
                    let project_name = project_name.clone();
                    self.state = AppState::ProjectMenu(project_name);
                    self.update_project_menu_items();
                },
                _ => {
                    self.state = previous;
                }
            }
        } else {
            self.go_back();
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) -> Result<(), Box<dyn std::error::Error>> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                if matches!(self.state, AppState::MainMenu) {
                    self.should_quit = true;
                } else if matches!(self.state, AppState::Loading(_) | AppState::Error(_)) {
                    // Return to previous context for Loading/Error states
                    self.return_to_context();
                } else {
                    self.go_back();
                }
            }
            KeyCode::Up => self.move_up(),
            KeyCode::Down => self.move_down(),
            KeyCode::Enter => self.handle_enter()?,
            KeyCode::Char(c) => {
                if let AppState::Input(ref mut input_state) = self.state {
                    input_state.input.push(c);
                }
            }
            KeyCode::Backspace => {
                if let AppState::Input(ref mut input_state) = self.state {
                    input_state.input.pop();
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.adjust_scroll();
        }
    }

    fn move_down(&mut self) {
        if self.selected_index < self.items.len().saturating_sub(1) {
            self.selected_index += 1;
            self.adjust_scroll();
        }
    }

    fn adjust_scroll(&mut self) {
        const VISIBLE_ITEMS: usize = 10; // Number of items visible at once
        
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + VISIBLE_ITEMS {
            self.scroll_offset = self.selected_index + 1 - VISIBLE_ITEMS;
        }
    }

    fn handle_enter(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.state {
            AppState::MainMenu => self.handle_main_menu_selection()?,
            AppState::ListProjects => self.handle_list_projects_selection()?,
            AppState::OpenProject => self.handle_open_project_selection()?,
            AppState::DeleteProject => self.handle_delete_project_selection()?,
            AppState::ProjectMenu(project_name) => {
                let project_name = project_name.clone();
                self.handle_project_menu_selection(project_name)?;
            },
            AppState::TicketList(project_name) => {
                let project_name = project_name.clone();
                self.handle_ticket_list_selection(project_name)?;
            },
            AppState::TicketDetail(project_name, ticket_id) => {
                let project_name = project_name.clone();
                let ticket_id = ticket_id.clone();
                self.handle_ticket_detail_selection(project_name, ticket_id)?;
            },
            AppState::TicketDetailsView(project_name, ticket_id) => {
                let project_name = project_name.clone();
                let ticket_id = ticket_id.clone();
                self.handle_ticket_details_view_selection(project_name, ticket_id)?;
            },
            AppState::QuickRefine(project_name, ticket_id) => {
                let project_name = project_name.clone();
                let ticket_id = ticket_id.clone();
                self.handle_quick_refine_selection(project_name, ticket_id)?;
            },
            AppState::Input(_) => self.handle_input_submit()?,
            _ => {}
        }
        Ok(())
    }

    fn handle_main_menu_selection(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.selected_index {
            0 => self.list_projects(),
            1 => self.start_create_project(),
            2 => self.show_open_project_menu(),
            3 => self.show_delete_project_menu(),
            4 => self.should_quit = true,
            _ => {}
        }
        Ok(())
    }

    fn handle_list_projects_selection(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // In ListProjects state, only handle "← Back" selection
        // For project selections, we just stay in the list (it's a read-only view)
        if self.selected_index == self.items.len() - 1 {
            // "← Back" selected (it's always the last item)
            self.state = AppState::MainMenu;
            self.update_main_menu_items();
        }
        // If a project is selected (index < items.len() - 1), do nothing - just stay in list view
        Ok(())
    }

    fn handle_open_project_selection(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let projects = self.project_manager.list_projects();
        if self.selected_index < projects.len() {
            let project_name = projects[self.selected_index].clone();
            self.state = AppState::ProjectMenu(project_name);
            self.update_project_menu_items();
        } else if self.selected_index == projects.len() {
            // "← Back" selected
            self.state = AppState::MainMenu;
            self.update_main_menu_items();
        }
        Ok(())
    }

    fn handle_delete_project_selection(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let projects = self.project_manager.list_projects();
        if self.selected_index < projects.len() {
            let project_name = projects[self.selected_index].clone();
            // Delete the project
            match self.project_manager.delete_project(&project_name) {
                Ok(_) => {
                    self.state = AppState::Loading(format!("Deleted project: {}", project_name));
                },
                Err(e) => {
                    self.transition_to_state(AppState::Error(format!("Failed to delete project: {}", e)));
                }
            }
        } else if self.selected_index == projects.len() {
            // "← Back" selected
            self.state = AppState::MainMenu;
            self.update_main_menu_items();
        }
        Ok(())
    }

    fn handle_project_menu_selection(&mut self, project_name: String) -> Result<(), Box<dyn std::error::Error>> {
        match self.selected_index {
            0 => self.show_ticket_list(project_name)?,
            1 => self.start_create_ticket(project_name),
            2 => self.save_current_project()?,
            3 => {
                self.state = AppState::MainMenu;
                self.update_main_menu_items();
            },
            _ => {}
        }
        Ok(())
    }

    fn handle_ticket_detail_selection(&mut self, project_name: String, ticket_id: TicketId) -> Result<(), Box<dyn std::error::Error>> {
        match self.selected_index {
            0 => {
                // View ticket details - show the actual ticket details
                self.show_ticket_details_view(project_name, ticket_id)?;
            },
            1 => {
                // View refinement requests - show loading then return to ticket detail
                self.transition_to_state(AppState::Loading("Viewing refinement requests...".to_string()));
            },
            2 => {
                // Quick refine - show all terms (this changes context)
                self.show_quick_refine(project_name, ticket_id)?;
            },
            3 => {
                // View dependencies - show loading then return to ticket detail
                self.transition_to_state(AppState::Loading("Viewing dependencies...".to_string()));
            },
            4 => {
                // View dependents - show loading then return to ticket detail
                self.transition_to_state(AppState::Loading("Viewing dependents...".to_string()));
            },
            5 => {
                // Back
                self.show_ticket_list(project_name)?;
            },
            _ => {}
        }
        Ok(())
    }

    fn show_ticket_details_view(&mut self, project_name: String, ticket_id: TicketId) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.current_project {
            if let Some(_node) = project.graph.get_ticket(&ticket_id) {
                // Show basic navigation options for the details view
                self.items = vec![
                    "← Back to ticket menu".to_string(),
                ];
                
                self.state = AppState::TicketDetailsView(project_name, ticket_id);
                self.selected_index = 0;
                self.scroll_offset = 0;
            } else {
                self.transition_to_state(AppState::Error("Ticket not found.".to_string()));
            }
        } else {
            self.transition_to_state(AppState::Error("No project loaded.".to_string()));
        }
        Ok(())
    }

    fn handle_ticket_details_view_selection(&mut self, project_name: String, ticket_id: TicketId) -> Result<(), Box<dyn std::error::Error>> {
        match self.selected_index {
            0 => {
                // Back to ticket menu
                self.show_ticket_detail(project_name, ticket_id)?;
            },
            _ => {}
        }
        Ok(())
    }

    fn show_quick_refine(&mut self, project_name: String, ticket_id: TicketId) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.current_project {
            if let Some(node) = project.graph.get_ticket(&ticket_id) {
                if node.ticket.decomposed_ticket.terms_needing_refinement.is_empty() {
                    self.transition_to_state(AppState::Error("No terms need refinement in this ticket.".to_string()));
                    return Ok(());
                }

                // Create items showing priority indicators and terms
                self.items = node.ticket.decomposed_ticket.terms_needing_refinement
                    .iter()
                    .enumerate()
                    .map(|(i, request)| {
                        let priority_emoji = match request.priority {
                            control_flow::ticket::RefinementPriority::Critical => "🔥",
                            control_flow::ticket::RefinementPriority::High => "🟥",
                            control_flow::ticket::RefinementPriority::Medium => "🟨",
                            control_flow::ticket::RefinementPriority::Low => "🟩",
                        };
                        format!("{}. {} {} - {}", i + 1, priority_emoji, request.term, request.reason)
                    })
                    .collect();
                
                self.items.push("← Back".to_string());
                self.state = AppState::QuickRefine(project_name, ticket_id);
                self.selected_index = 0;
                self.scroll_offset = 0;
            }
        }
        Ok(())
    }

    fn handle_ticket_list_selection(&mut self, project_name: String) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.current_project {
            let root_tickets = project.get_root_tickets();
            if self.selected_index < root_tickets.len() {
                let ticket_id = root_tickets[self.selected_index].clone();
                self.show_ticket_detail(project_name, ticket_id)?;
            }
        }
        Ok(())
    }

    fn handle_quick_refine_selection(&mut self, project_name: String, ticket_id: TicketId) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.current_project {
            if let Some(node) = project.graph.get_ticket(&ticket_id) {
                if self.selected_index < node.ticket.decomposed_ticket.terms_needing_refinement.len() {
                    let refinement_request = &node.ticket.decomposed_ticket.terms_needing_refinement[self.selected_index];
                    self.transition_to_state(AppState::Loading(format!("Creating refinement ticket for '{}'...", refinement_request.term)));
                    // TODO: In a real implementation, this would:
                    // 1. Trigger the async refinement process
                    // 2. Create a new ticket for the refinement
                    // 3. Navigate to that new ticket's detail view (becomes new context)
                    // For now, we'll simulate a creation that goes to a new ticket view
                    // In practice, you'd call something like:
                    // let new_ticket_id = self.create_refinement_ticket(refinement_request).await?;
                    // self.show_ticket_detail(project_name, new_ticket_id)?;
                } else if self.selected_index == node.ticket.decomposed_ticket.terms_needing_refinement.len() {
                    // "← Back" selected
                    self.show_ticket_detail(project_name, ticket_id)?;
                }
            }
        }
        Ok(())
    }

    fn handle_input_submit(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let AppState::Input(input_state) = &self.state.clone() {
            let input_text = input_state.input.clone();
            let return_state = input_state.return_state.as_ref().clone();
            
            match return_state {
                AppState::CreateProject => {
                    if input_state.title.contains("name") {
                        // Creating project with the input as name
                        self.create_project_with_name(input_text)?;
                    } else {
                        // This would be the description input
                        self.finalize_project_creation(input_text)?;
                    }
                },
                AppState::CreateTicket(project_name) => {
                    self.create_ticket_with_description(project_name, input_text)?;
                },
                _ => {
                    self.state = return_state;
                }
            }
        }
        Ok(())
    }

    fn go_back(&mut self) {
        match &self.state {
            AppState::Loading(_) | AppState::Error(_) => {
                // These should use return_to_context instead
                self.return_to_context();
                return;
            },
            AppState::ListProjects | AppState::OpenProject | AppState::DeleteProject => {
                self.state = AppState::MainMenu;
                self.update_main_menu_items();
            },
            AppState::ProjectMenu(_) => {
                self.state = AppState::MainMenu;
                self.update_main_menu_items();
            },
            AppState::TicketList(project_name) => {
                let project_name = project_name.clone();
                self.state = AppState::ProjectMenu(project_name);
                self.update_project_menu_items();
            },
            AppState::TicketDetail(project_name, _) => {
                let project_name = project_name.clone();
                self.show_ticket_list(project_name).ok();
            },
            AppState::TicketDetailsView(project_name, ticket_id) => {
                let project_name = project_name.clone();
                let ticket_id = ticket_id.clone();
                self.show_ticket_detail(project_name, ticket_id).ok();
            },
            AppState::QuickRefine(project_name, ticket_id) => {
                let project_name = project_name.clone();
                let ticket_id = ticket_id.clone();
                self.show_ticket_detail(project_name, ticket_id).ok();
            },
            _ => {
                self.state = AppState::MainMenu;
                self.update_main_menu_items();
            }
        }
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    fn list_projects(&mut self) {
        let projects = self.project_manager.list_projects();
        if projects.is_empty() {
            self.transition_to_state(AppState::Error("No projects found.".to_string()));
        } else {
            self.items = projects.iter().map(|p| p.to_string()).collect();
            self.items.push("← Back".to_string());
            self.state = AppState::ListProjects;
        }
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    fn start_create_project(&mut self) {
        self.state = AppState::Input(InputState {
            title: "Create New Project".to_string(),
            prompt: "Enter project name:".to_string(),
            input: String::new(),
            return_state: Box::new(AppState::CreateProject),
        });
    }

    fn show_open_project_menu(&mut self) {
        let projects = self.project_manager.list_projects();
        if projects.is_empty() {
            self.transition_to_state(AppState::Error("No projects found.".to_string()));
        } else {
            self.items = projects.iter().map(|p| p.to_string()).collect();
            self.items.push("← Back".to_string());
            self.state = AppState::OpenProject;
        }
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    fn show_delete_project_menu(&mut self) {
        let projects = self.project_manager.list_projects();
        if projects.is_empty() {
            self.transition_to_state(AppState::Error("No projects found.".to_string()));
        } else {
            self.items = projects.iter().map(|p| format!("Delete: {}", p)).collect();
            self.items.push("← Back".to_string());
            self.state = AppState::DeleteProject;
        }
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    fn create_project_with_name(&mut self, name: String) -> Result<(), Box<dyn std::error::Error>> {
        // Store the name and ask for description
        self.state = AppState::Input(InputState {
            title: format!("Create Project: {}", name),
            prompt: "Enter project description:".to_string(),
            input: String::new(),
            return_state: Box::new(AppState::CreateProject),
        });
        Ok(())
    }

    fn finalize_project_creation(&mut self, _description: String) -> Result<(), Box<dyn std::error::Error>> {
        // This is a simplified version - in reality we'd need to store the name from the previous step
        self.state = AppState::MainMenu;
        self.update_main_menu_items();
        Ok(())
    }

    fn show_ticket_list(&mut self, project_name: String) -> Result<(), Box<dyn std::error::Error>> {
        self.current_project = Some(self.project_manager.load_project(&project_name)?);
        
        if let Some(project) = &self.current_project {
            let root_tickets = project.get_root_tickets();
            if root_tickets.is_empty() {
                self.items = vec!["No tickets found. Create a new ticket.".to_string(), "← Back".to_string()];
            } else {
                self.items = root_tickets.iter().enumerate().map(|(i, ticket_id)| {
                    if let Some(node) = project.graph.get_ticket(ticket_id) {
                        format!("{}. {} ({})", i + 1, node.ticket.original_ticket.title, ticket_id)
                    } else {
                        format!("{}. Unknown ticket ({})", i + 1, ticket_id)
                    }
                }).collect();
                self.items.push("← Back".to_string());
            }
        }
        
        self.state = AppState::TicketList(project_name);
        self.selected_index = 0;
        self.scroll_offset = 0;
        Ok(())
    }

    fn show_ticket_detail(&mut self, project_name: String, ticket_id: TicketId) -> Result<(), Box<dyn std::error::Error>> {
        // Update items based on current ticket status
        if let Some(project) = &self.current_project {
            if let Some(node) = project.graph.get_ticket(&ticket_id) {
                let refinement_count = node.ticket.decomposed_ticket.terms_needing_refinement.len();
                let dependencies_count = node.dependencies.len();
                let dependents_count = node.dependents.len();
                
                self.items = vec![
                    "View ticket details".to_string(),
                    format!("View refinement requests ({})", refinement_count),
                    format!("Quick refine - show all terms ({})", refinement_count),
                    format!("View dependencies ({})", dependencies_count),
                    format!("View dependents ({})", dependents_count),
                    "← Back".to_string(),
                ];
            } else {
                self.items = vec![
                    "View ticket details".to_string(),
                    "View refinement requests".to_string(),
                    "Quick refine - show all terms".to_string(),
                    "View dependencies".to_string(),
                    "View dependents".to_string(),
                    "← Back".to_string(),
                ];
            }
        }
        
        self.state = AppState::TicketDetail(project_name, ticket_id);
        self.selected_index = 0;
        self.scroll_offset = 0;
        Ok(())
    }

    fn start_create_ticket(&mut self, project_name: String) {
        self.state = AppState::Input(InputState {
            title: "Create New Ticket".to_string(),
            prompt: "Enter ticket description:".to_string(),
            input: String::new(),
            return_state: Box::new(AppState::CreateTicket(project_name)),
        });
    }

    fn create_ticket_with_description(&mut self, project_name: String, _description: String) -> Result<(), Box<dyn std::error::Error>> {
        self.transition_to_state(AppState::Loading("Creating ticket...".to_string()));
        // TODO: In a real implementation, this would:
        // 1. Trigger the async ticket creation
        // 2. Create a new ticket
        // 3. Navigate to that new ticket's detail view (becomes new context)
        // For now, we'll go back to the ticket list
        // In practice, you'd call something like:
        // let new_ticket_id = self.create_ticket_async(description).await?;
        // self.show_ticket_detail(project_name, new_ticket_id)?;
        self.show_ticket_list(project_name)
    }

    fn save_current_project(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.current_project {
            self.project_manager.save_project(project)?;
        }
        Ok(())
    }

    fn update_main_menu_items(&mut self) {
        self.items = vec![
            "List projects".to_string(),
            "Create new project".to_string(),
            "Open project".to_string(),
            "Delete project".to_string(),
            "Exit".to_string(),
        ];
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    fn update_project_menu_items(&mut self) {
        self.items = vec![
            "View tickets".to_string(),
            "Create new ticket".to_string(),
            "Save project".to_string(),
            "← Back to main menu".to_string(),
        ];
        self.selected_index = 0;
        self.scroll_offset = 0;
    }
}

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
        ])
        .split(frame.size());

    // Header
    let title = match &app.state {
        AppState::MainMenu => "🎫 Control Flow - Main Menu",
        AppState::ListProjects => "📋 All Projects",
        AppState::OpenProject => "📂 Open Project",
        AppState::DeleteProject => "🗑️ Delete Project",
        AppState::ProjectMenu(name) => &format!("📁 Project: {}", name),
        AppState::TicketList(name) => &format!("🎫 Tickets in: {}", name),
        AppState::TicketDetail(name, ticket_id) => &format!("🎫 Ticket {} in: {}", ticket_id, name),
        AppState::TicketDetailsView(name, ticket_id) => &format!("📄 Details: Ticket {} in: {}", ticket_id, name),
        AppState::CreateProject => "📝 Create New Project",
        AppState::CreateTicket(_) => "📝 Create New Ticket",
        AppState::QuickRefine(_, _) => "🔍 Quick Refine",
        AppState::Input(input_state) => &input_state.title,
        AppState::Loading(msg) => msg,
        AppState::Error(_) => "❌ Error",
    };

    let header = Paragraph::new(title)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(header, chunks[0]);

    // Main content
    match &app.state {
        AppState::Input(input_state) => render_input(frame, chunks[1], input_state),
        AppState::Loading(msg) => render_loading(frame, chunks[1], msg),
        AppState::Error(msg) => render_error(frame, chunks[1], msg),
        AppState::TicketDetailsView(project_name, ticket_id) => {
            render_ticket_details(frame, chunks[1], app, project_name, ticket_id)
        },
        _ => render_menu(frame, chunks[1], app),
    }

    // Footer with instructions
    let instructions = match &app.state {
        AppState::Input(_) => "Enter: Submit | Esc: Cancel | Backspace: Delete",
        _ => "↑↓: Navigate | Enter: Select | Esc/Q: Back/Quit",
    };
    
    let footer = Paragraph::new(instructions)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(footer, chunks[2]);
}

fn render_menu(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app.items.iter().enumerate().map(|(i, item)| {
        let style = if i == app.selected_index {
            Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        
        ListItem::new(item.as_str()).style(style)
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Options"))
        .highlight_style(Style::default().fg(Color::Black).bg(Color::White));

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_index));
    
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_input(frame: &mut Frame, area: Rect, input_state: &InputState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Prompt
            Constraint::Length(3), // Input field
            Constraint::Min(0),    // Spacer
        ])
        .split(area);

    let prompt = Paragraph::new(input_state.prompt.as_str())
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(prompt, chunks[0]);

    let input = Paragraph::new(input_state.input.as_str())
        .block(Block::default().borders(Borders::ALL).title("Input"))
        .style(Style::default().fg(Color::White));
    frame.render_widget(input, chunks[1]);
}

fn render_loading(frame: &mut Frame, area: Rect, message: &str) {
    let loading = Paragraph::new(format!("⏳ {}", message))
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Yellow))
        .wrap(Wrap { trim: true });
    frame.render_widget(loading, area);
}

fn render_error(frame: &mut Frame, area: Rect, message: &str) {
    let error = Paragraph::new(format!("❌ {}\n\nPress Esc to go back.", message))
        .block(Block::default().borders(Borders::ALL).title("Error"))
        .style(Style::default().fg(Color::Red))
        .wrap(Wrap { trim: true });
    frame.render_widget(error, area);
}

fn render_ticket_details(frame: &mut Frame, area: Rect, app: &App, _project_name: &str, ticket_id: &TicketId) {
    // Split area into ticket details and navigation
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),     // Ticket details
            Constraint::Length(5),  // Navigation
        ])
        .split(area);

    // Render ticket details
    if let Some(project) = &app.current_project {
        if let Some(node) = project.graph.get_ticket(ticket_id) {
            let ticket = &node.ticket;
            
            // Create detailed text content
            let mut details = vec![
                format!("📋 Title: {}", ticket.original_ticket.title),
                format!("🆔 ID: {}", ticket_id),
                format!("📝 Raw Input: {}", ticket.original_ticket.raw_input),
                "".to_string(),
                "🎯 Decomposed Ticket:".to_string(),
                format!("  Status: {:?}", ticket.decomposed_ticket.metadata.status),
                format!("  Priority: {:?}", ticket.decomposed_ticket.metadata.priority),
                format!("  Complexity: {:?}", ticket.decomposed_ticket.metadata.estimated_complexity),
                "".to_string(),
            ];
            
            // Add terms
            if !ticket.decomposed_ticket.terms.is_empty() {
                details.push("📚 Terms:".to_string());
                for (term, definition) in &ticket.decomposed_ticket.terms {
                    details.push(format!("  • {}: {}", term, definition));
                }
                details.push("".to_string());
            }
            
            // Add validation method
            if !ticket.decomposed_ticket.validation_method.is_empty() {
                details.push("✅ Validation Method:".to_string());
                for (i, method) in ticket.decomposed_ticket.validation_method.iter().enumerate() {
                    details.push(format!("  {}. {}", i + 1, method));
                }
                details.push("".to_string());
            }
            
            // Add open questions
            if !ticket.decomposed_ticket.open_questions.is_empty() {
                details.push("❓ Open Questions:".to_string());
                for (i, question) in ticket.decomposed_ticket.open_questions.iter().enumerate() {
                    details.push(format!("  {}. {}", i + 1, question));
                }
                details.push("".to_string());
            }
            
            // Add engine questions
            if !ticket.decomposed_ticket.engine_questions.is_empty() {
                details.push("❓ Engine Questions:".to_string());
                for (i, question) in ticket.decomposed_ticket.engine_questions.iter().enumerate() {
                    details.push(format!("  {}. {}", i + 1, question));
                }
                details.push("".to_string());
            }
            
            // Add terms needing refinement
            if !ticket.decomposed_ticket.terms_needing_refinement.is_empty() {
                details.push("🔍 Terms Needing Refinement:".to_string());
                for (i, request) in ticket.decomposed_ticket.terms_needing_refinement.iter().enumerate() {
                    let priority_emoji = match request.priority {
                        control_flow::ticket::RefinementPriority::Critical => "🔥",
                        control_flow::ticket::RefinementPriority::High => "🟥",
                        control_flow::ticket::RefinementPriority::Medium => "🟨",
                        control_flow::ticket::RefinementPriority::Low => "🟩",
                    };
                    details.push(format!("  {}. {} {} - {}", i + 1, priority_emoji, request.term, request.reason));
                }
                details.push("".to_string());
            }
            
            // Add dependencies info
            if !node.dependencies.is_empty() {
                details.push(format!("🔗 Dependencies: {} ticket(s)", node.dependencies.len()));
            }
            if !node.dependents.is_empty() {
                details.push(format!("⬆️ Dependents: {} ticket(s)", node.dependents.len()));
            }
            
            let content = details.join("\n");
            let paragraph = Paragraph::new(content)
                .block(Block::default().borders(Borders::ALL).title("Ticket Details"))
                .style(Style::default().fg(Color::White))
                .wrap(Wrap { trim: true });
            frame.render_widget(paragraph, chunks[0]);
        } else {
            let error = Paragraph::new("❌ Ticket not found")
                .block(Block::default().borders(Borders::ALL).title("Error"))
                .style(Style::default().fg(Color::Red));
            frame.render_widget(error, chunks[0]);
        }
    } else {
        let error = Paragraph::new("❌ No project loaded")
            .block(Block::default().borders(Borders::ALL).title("Error"))
            .style(Style::default().fg(Color::Red));
        frame.render_widget(error, chunks[0]);
    }
    
    // Render navigation menu at bottom
    render_menu(frame, chunks[1], app);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "control_flow=info,client_implementations=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    info!("Starting Control Flow TUI");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new()?;

    // Main loop
    loop {
        terminal.draw(|f| render(f, &app))?;

        if app.should_quit {
            break;
        }

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.handle_key(key.code)?;
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    info!("Control Flow TUI shutdown");
    Ok(())
}