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
    text::{Line, Span},
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
    TicketFieldAction(String, TicketId, TicketField), // project name, ticket id, selected field
    TicketSearch(String, TicketId, SearchState), // project name, ticket id, search state
    CreateProject,
    CreateTicket(String), // project name
    QuickRefine(String, TicketId), // project name, ticket id
    Input(InputState),
    Loading(String), // loading message
    Error(String),   // error message
}

#[derive(Debug, Clone)]
pub enum TicketField {
    Title,
    RawInput,
    Status,
    Priority,
    Complexity,
    Terms(String), // specific term key
    ValidationMethod(usize), // index
    OpenQuestion(usize), // index
    EngineQuestion(usize), // index
    RefinementRequest(usize), // index
    Dependencies,
    Dependents,
}

#[derive(Debug, Clone)]
pub struct SearchState {
    pub query: String,
    pub matches: Vec<SearchMatch>,
    pub current_match: usize,
}

#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub field: TicketField,
    pub text: String,
    pub match_start: usize,
    pub match_end: usize,
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
    pub ticket_fields: Vec<TicketField>, // Available fields in ticket details view
    pub field_actions: Vec<String>, // Available actions for selected field
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
            ticket_fields: Vec::new(),
            field_actions: Vec::new(),
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
                AppState::TicketSearch(project_name, ticket_id, _) => {
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
                } else if matches!(self.state, AppState::TicketSearch(_, _, _)) {
                    // Exit search mode and return to ticket details view
                    if let AppState::TicketSearch(project_name, ticket_id, _) = &self.state {
                        let project_name = project_name.clone();
                        let ticket_id = ticket_id.clone();
                        self.show_ticket_details_view(project_name, ticket_id)?;
                    }
                } else {
                    self.go_back();
                }
            }
            KeyCode::Char('/') => {
                // Enter search mode if in ticket details view
                if let AppState::TicketDetailsView(project_name, ticket_id) = &self.state {
                    let project_name = project_name.clone();
                    let ticket_id = ticket_id.clone();
                    self.start_ticket_search(project_name, ticket_id)?;
                }
            }
            KeyCode::Up => self.move_up(),
            KeyCode::Down => self.move_down(),
            KeyCode::Enter => self.handle_enter()?,
            KeyCode::Char(c) => {
                if let AppState::Input(ref mut input_state) = self.state {
                    input_state.input.push(c);
                } else if let AppState::TicketSearch(_, _, ref mut search_state) = self.state {
                    if c != '/' {  // Don't add the initial '/' character
                        search_state.query.push(c);
                        self.update_search_matches()?;
                    }
                } else if let AppState::TicketDetailsView(project_name, ticket_id) = &self.state {
                    // Handle number keys for action execution
                    if c.is_ascii_digit() && c != '0' {
                        let action_index = (c as usize) - ('1' as usize);
                        if self.selected_index < self.ticket_fields.len() {
                            let field = self.ticket_fields[self.selected_index].clone();
                            let project_name = project_name.clone();
                            let ticket_id = ticket_id.clone();
                            self.execute_field_action(project_name, ticket_id, field, action_index)?;
                        }
                    }
                }
            }
            KeyCode::Backspace => {
                if let AppState::Input(ref mut input_state) = self.state {
                    input_state.input.pop();
                } else if let AppState::TicketSearch(_, _, ref mut search_state) = self.state {
                    search_state.query.pop();
                    self.update_search_matches()?;
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
            AppState::TicketSearch(project_name, ticket_id, search_state) => {
                let project_name = project_name.clone();
                let ticket_id = ticket_id.clone();
                let search_state = search_state.clone();
                self.handle_ticket_search_selection(project_name, ticket_id, search_state)?;
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
            if let Some(node) = project.graph.get_ticket(&ticket_id) {
                let ticket = &node.ticket;
                
                // Build the list of available fields to navigate
                let mut fields = Vec::new();
                let mut items = Vec::new();
                
                // Basic ticket info
                fields.push(TicketField::Title);
                items.push(format!("📋 Title: {}", ticket.original_ticket.title));
                
                fields.push(TicketField::RawInput);
                items.push("📝 Raw Input".to_string());
                
                fields.push(TicketField::Status);
                items.push(format!("📊 Status: {:?}", ticket.decomposed_ticket.metadata.status));
                
                fields.push(TicketField::Priority);
                items.push(format!("🎯 Priority: {:?}", ticket.decomposed_ticket.metadata.priority));
                
                fields.push(TicketField::Complexity);
                items.push(format!("⚡ Complexity: {:?}", ticket.decomposed_ticket.metadata.estimated_complexity));
                
                // Terms
                for (term, _) in &ticket.decomposed_ticket.terms {
                    fields.push(TicketField::Terms(term.clone()));
                    items.push(format!("📚 Term: {}", term));
                }
                
                // Validation methods
                for (i, _) in ticket.decomposed_ticket.validation_method.iter().enumerate() {
                    fields.push(TicketField::ValidationMethod(i));
                    items.push(format!("✅ Validation Method #{}", i + 1));
                }
                
                // Open questions
                for (i, _) in ticket.decomposed_ticket.open_questions.iter().enumerate() {
                    fields.push(TicketField::OpenQuestion(i));
                    items.push(format!("❓ Open Question #{}", i + 1));
                }
                
                // Engine questions
                for (i, _) in ticket.decomposed_ticket.engine_questions.iter().enumerate() {
                    fields.push(TicketField::EngineQuestion(i));
                    items.push(format!("🔧 Engine Question #{}", i + 1));
                }
                
                // Refinement requests
                for (i, request) in ticket.decomposed_ticket.terms_needing_refinement.iter().enumerate() {
                    fields.push(TicketField::RefinementRequest(i));
                    let priority_emoji = match request.priority {
                        control_flow::ticket::RefinementPriority::Critical => "🔥",
                        control_flow::ticket::RefinementPriority::High => "🟥",
                        control_flow::ticket::RefinementPriority::Medium => "🟨",
                        control_flow::ticket::RefinementPriority::Low => "🟩",
                    };
                    items.push(format!("🔍 {} Refinement: {}", priority_emoji, request.term));
                }
                
                // Dependencies and dependents
                if !node.dependencies.is_empty() {
                    fields.push(TicketField::Dependencies);
                    items.push(format!("🔗 Dependencies ({})", node.dependencies.len()));
                }
                
                if !node.dependents.is_empty() {
                    fields.push(TicketField::Dependents);
                    items.push(format!("⬆️ Dependents ({})", node.dependents.len()));
                }
                
                // Add back option
                items.push("← Back to ticket menu".to_string());
                
                self.ticket_fields = fields;
                self.items = items;
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
        if self.selected_index == self.items.len() - 1 {
            // "← Back to ticket menu" selected (always last item)
            self.show_ticket_detail(project_name, ticket_id)?;
        } else if self.selected_index < self.ticket_fields.len() {
            // A field was selected - execute the first action for that field (most common action)
            let field = self.ticket_fields[self.selected_index].clone();
            self.execute_field_action(project_name, ticket_id, field, 0)?;
        }
        Ok(())
    }

    fn execute_field_action(&mut self, _project_name: String, ticket_id: TicketId, field: TicketField, action_index: usize) -> Result<(), Box<dyn std::error::Error>> {
        // Get the available actions for this field
        let (actions, _) = get_field_actions_with_content(self, &ticket_id, &field);
        
        if action_index < actions.len() {
            let action = &actions[action_index];
            let field_name = self.get_field_display_name(&field);
            
            // For now, show loading message with the action being executed
            self.transition_to_state(AppState::Loading(format!("Executing: {} on {}", action, field_name)));
            
            // TODO: Implement actual action execution based on action type
            // Examples:
            // - "View full content" -> show detailed view
            // - "Create refinement ticket" -> trigger ticket creation workflow
            // - "Edit definition" -> open edit mode
            // - "Answer question" -> open answer input
            // - etc.
        } else {
            self.transition_to_state(AppState::Error("Invalid action selected".to_string()));
        }
        
        Ok(())
    }


    fn get_field_display_name(&self, field: &TicketField) -> String {
        match field {
            TicketField::Title => "Title".to_string(),
            TicketField::RawInput => "Raw Input".to_string(),
            TicketField::Status => "Status".to_string(),
            TicketField::Priority => "Priority".to_string(),
            TicketField::Complexity => "Complexity".to_string(),
            TicketField::Terms(term) => format!("Term: {}", term),
            TicketField::ValidationMethod(i) => format!("Validation #{}", i + 1),
            TicketField::OpenQuestion(i) => format!("Question #{}", i + 1),
            TicketField::EngineQuestion(i) => format!("Engine Q #{}", i + 1),
            TicketField::RefinementRequest(i) => format!("Refinement #{}", i + 1),
            TicketField::Dependencies => "Dependencies".to_string(),
            TicketField::Dependents => "Dependents".to_string(),
        }
    }


    fn start_ticket_search(&mut self, project_name: String, ticket_id: TicketId) -> Result<(), Box<dyn std::error::Error>> {
        let search_state = SearchState {
            query: String::new(),
            matches: Vec::new(),
            current_match: 0,
        };
        
        self.state = AppState::TicketSearch(project_name, ticket_id, search_state);
        Ok(())
    }

    fn update_search_matches(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let AppState::TicketSearch(_, ref ticket_id, ref mut search_state) = &mut self.state {
            if search_state.query.is_empty() {
                search_state.matches.clear();
                search_state.current_match = 0;
                return Ok(());
            }
            
            // Search through ticket content
            search_state.matches.clear();
            
            if let Some(project) = &self.current_project {
                if let Some(node) = project.graph.get_ticket(ticket_id) {
                    let ticket = &node.ticket;
                    let query = search_state.query.to_lowercase();
                    
                    // Search in title
                    if ticket.original_ticket.title.to_lowercase().contains(&query) {
                        search_state.matches.push(SearchMatch {
                            field: TicketField::Title,
                            text: ticket.original_ticket.title.clone(),
                            match_start: ticket.original_ticket.title.to_lowercase().find(&query).unwrap_or(0),
                            match_end: ticket.original_ticket.title.to_lowercase().find(&query).unwrap_or(0) + query.len(),
                        });
                    }
                    
                    // Search in raw input
                    if ticket.original_ticket.raw_input.to_lowercase().contains(&query) {
                        search_state.matches.push(SearchMatch {
                            field: TicketField::RawInput,
                            text: ticket.original_ticket.raw_input.clone(),
                            match_start: ticket.original_ticket.raw_input.to_lowercase().find(&query).unwrap_or(0),
                            match_end: ticket.original_ticket.raw_input.to_lowercase().find(&query).unwrap_or(0) + query.len(),
                        });
                    }
                    
                    // Search in terms
                    for (term, definition) in &ticket.decomposed_ticket.terms {
                        if term.to_lowercase().contains(&query) || definition.to_lowercase().contains(&query) {
                            search_state.matches.push(SearchMatch {
                                field: TicketField::Terms(term.clone()),
                                text: format!("{}: {}", term, definition),
                                match_start: 0, // Simplified for now
                                match_end: query.len(),
                            });
                        }
                    }
                    
                    // Reset current match if out of bounds
                    if search_state.current_match >= search_state.matches.len() {
                        search_state.current_match = 0;
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_ticket_search_selection(&mut self, project_name: String, ticket_id: TicketId, mut search_state: SearchState) -> Result<(), Box<dyn std::error::Error>> {
        if !search_state.matches.is_empty() {
            // Move to next match
            search_state.current_match = (search_state.current_match + 1) % search_state.matches.len();
            self.state = AppState::TicketSearch(project_name, ticket_id, search_state);
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
            AppState::TicketSearch(project_name, ticket_id, _) => {
                let project_name = project_name.clone();
                let ticket_id = ticket_id.clone();
                self.show_ticket_details_view(project_name, ticket_id).ok();
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
        AppState::TicketFieldAction(name, ticket_id, _) => &format!("⚡ Actions: Ticket {} in: {}", ticket_id, name),
        AppState::TicketSearch(name, ticket_id, _) => &format!("🔍 Search: Ticket {} in: {}", ticket_id, name),
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
        AppState::TicketSearch(project_name, ticket_id, search_state) => {
            render_ticket_search(frame, chunks[1], app, project_name, ticket_id, search_state)
        },
        _ => render_menu(frame, chunks[1], app),
    }

    // Footer with instructions
    let instructions = match &app.state {
        AppState::Input(_) => "Enter: Submit | Esc: Cancel | Backspace: Delete",
        AppState::TicketDetailsView(_, _) => "↑↓: Navigate | 1-9: Execute action | Enter: Default action | /: Search | Esc: Back",
        AppState::TicketSearch(_, _, _) => "Type: Search | Enter: Next match | Esc: Exit search",
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
            Constraint::Length(30),  // Navigation
        ])
        .split(area);

    // Render ticket details with highlighting for selected field
    if let Some(project) = &app.current_project {
        if let Some(node) = project.graph.get_ticket(ticket_id) {
            let ticket = &node.ticket;
            
            // Build the detailed content with highlighting for the selected field
            let lines = build_ticket_lines_with_highlight(ticket, node, ticket_id, app.selected_index, &app.ticket_fields);
            
            let paragraph = Paragraph::new(lines)
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
    
    // Render contextual actions for selected field in bottom pane
    render_field_actions_menu(frame, chunks[1], app, ticket_id);
}

fn build_ticket_lines_with_highlight(
    ticket: &control_flow::ticket::TicketDecomposition,
    node: &control_flow::ticket::TicketNode,
    _ticket_id: &TicketId,
    selected_index: usize,
    ticket_fields: &[TicketField]
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut field_index = 0;
    
    // Helper to create a line with potential highlighting
    let create_field_line = |text: String, field_index: usize, field_type: &TicketField| -> Line<'static> {
        if field_index == selected_index && field_index < ticket_fields.len() {
            // Check if this is the selected field
            if std::mem::discriminant(&ticket_fields[field_index]) == std::mem::discriminant(field_type) {
                // Create highlighted line with white background
                Line::from(vec![Span::styled(
                    text,
                    Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD)
                )])
            } else {
                Line::from(text)
            }
        } else {
            Line::from(text)
        }
    };
    
    // Basic ticket info
    lines.push(create_field_line(format!("📋 Title: {}", ticket.original_ticket.title), field_index, &TicketField::Title));
    field_index += 1;
    
    lines.push(create_field_line("📝 Raw Input".to_string(), field_index, &TicketField::RawInput));
    field_index += 1;
    
    lines.push(create_field_line(format!("📊 Status: {:?}", ticket.decomposed_ticket.metadata.status), field_index, &TicketField::Status));
    field_index += 1;
    
    lines.push(create_field_line(format!("🎯 Priority: {:?}", ticket.decomposed_ticket.metadata.priority), field_index, &TicketField::Priority));
    field_index += 1;
    
    lines.push(create_field_line(format!("⚡ Complexity: {:?}", ticket.decomposed_ticket.metadata.estimated_complexity), field_index, &TicketField::Complexity));
    field_index += 1;
    
    // Add empty line separator
    lines.push(Line::from(""));
    
    // Terms
    for (term, definition) in &ticket.decomposed_ticket.terms {
        lines.push(create_field_line(format!("📚 Term: {} = {}", term, definition), field_index, &TicketField::Terms(term.clone())));
        field_index += 1;
    }
    
    if !ticket.decomposed_ticket.terms.is_empty() {
        lines.push(Line::from(""));
    }
    
    // Validation methods
    for (i, method) in ticket.decomposed_ticket.validation_method.iter().enumerate() {
        lines.push(create_field_line(format!("✅ Validation #{}: {}", i + 1, method), field_index, &TicketField::ValidationMethod(i)));
        field_index += 1;
    }
    
    if !ticket.decomposed_ticket.validation_method.is_empty() {
        lines.push(Line::from(""));
    }
    
    // Open questions
    for (i, question) in ticket.decomposed_ticket.open_questions.iter().enumerate() {
        lines.push(create_field_line(format!("❓ Open Question #{}: {}", i + 1, question), field_index, &TicketField::OpenQuestion(i)));
        field_index += 1;
    }
    
    if !ticket.decomposed_ticket.open_questions.is_empty() {
        lines.push(Line::from(""));
    }
    
    // Engine questions
    for (i, question) in ticket.decomposed_ticket.engine_questions.iter().enumerate() {
        lines.push(create_field_line(format!("🔧 Engine Question #{}: {}", i + 1, question), field_index, &TicketField::EngineQuestion(i)));
        field_index += 1;
    }
    
    if !ticket.decomposed_ticket.engine_questions.is_empty() {
        lines.push(Line::from(""));
    }
    
    // Refinement requests
    for (i, request) in ticket.decomposed_ticket.terms_needing_refinement.iter().enumerate() {
        let priority_emoji = match request.priority {
            control_flow::ticket::RefinementPriority::Critical => "🔥",
            control_flow::ticket::RefinementPriority::High => "🟥",
            control_flow::ticket::RefinementPriority::Medium => "🟨",
            control_flow::ticket::RefinementPriority::Low => "🟩",
        };
        lines.push(create_field_line(
            format!("🔍 {} Refinement #{}: {} - {}", priority_emoji, i + 1, request.term, request.reason),
            field_index,
            &TicketField::RefinementRequest(i)
        ));
        field_index += 1;
    }
    
    if !ticket.decomposed_ticket.terms_needing_refinement.is_empty() {
        lines.push(Line::from(""));
    }
    
    // Dependencies and dependents
    if !node.dependencies.is_empty() {
        lines.push(create_field_line(format!("🔗 Dependencies ({})", node.dependencies.len()), field_index, &TicketField::Dependencies));
        field_index += 1;
    }
    
    if !node.dependents.is_empty() {
        lines.push(create_field_line(format!("⬆️ Dependents ({})", node.dependents.len()), field_index, &TicketField::Dependents));
        field_index += 1;
    }
    
    lines
}

fn render_field_actions_menu(frame: &mut Frame, area: Rect, app: &App, ticket_id: &TicketId) {
    // Get the currently selected field
    if app.selected_index < app.ticket_fields.len() {
        let selected_field = &app.ticket_fields[app.selected_index];
        
        // Generate contextual actions for the selected field
        let (actions, field_context) = get_field_actions_with_content(app, ticket_id, selected_field);
        
        // Create action items for display
        let action_items: Vec<ListItem> = actions.iter().enumerate().map(|(i, action)| {
            let style = Style::default().fg(Color::White);
            ListItem::new(format!("{}. {}", i + 1, action)).style(style)
        }).collect();
        
        // Create the action list
        let actions_list = List::new(action_items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(format!("Actions for: {}", field_context)))
            .style(Style::default().fg(Color::Green));
            
        frame.render_widget(actions_list, area);
    } else {
        // Show back option or general navigation
        let back_item = vec![ListItem::new("← Back to ticket menu")];
        let back_list = List::new(back_item)
            .block(Block::default().borders(Borders::ALL).title("Navigation"))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(back_list, area);
    }
}

fn get_field_actions_with_content(app: &App, ticket_id: &TicketId, field: &TicketField) -> (Vec<String>, String) {
    if let Some(project) = &app.current_project {
        if let Some(node) = project.graph.get_ticket(ticket_id) {
            let ticket = &node.ticket;
            
            match field {
                TicketField::Title => {
                    let context = format!("Title: {}", ticket.original_ticket.title);
                    let actions = vec![
                        "View full title".to_string(),
                        "Edit title".to_string(),
                        "Copy to clipboard".to_string(),
                    ];
                    (actions, context)
                },
                TicketField::RawInput => {
                    let preview = ticket.original_ticket.raw_input.chars().take(50).collect::<String>();
                    let context = format!("Raw Input: {}...", preview);
                    let actions = vec![
                        "View full content".to_string(),
                        "Edit raw input".to_string(),
                        "Re-decompose ticket".to_string(),
                        "Copy to clipboard".to_string(),
                    ];
                    (actions, context)
                },
                TicketField::Status => {
                    let context = format!("Status: {:?}", ticket.decomposed_ticket.metadata.status);
                    let actions = vec![
                        "Change status".to_string(),
                        "View status history".to_string(),
                        "Copy status".to_string(),
                    ];
                    (actions, context)
                },
                TicketField::Priority => {
                    let context = format!("Priority: {:?}", ticket.decomposed_ticket.metadata.priority);
                    let actions = vec![
                        "Change priority".to_string(),
                        "View priority reasoning".to_string(),
                        "Copy priority".to_string(),
                    ];
                    (actions, context)
                },
                TicketField::Complexity => {
                    let context = format!("Complexity: {:?}", ticket.decomposed_ticket.metadata.estimated_complexity);
                    let actions = vec![
                        "Re-estimate complexity".to_string(),
                        "View complexity breakdown".to_string(),
                        "Copy complexity".to_string(),
                    ];
                    (actions, context)
                },
                TicketField::Terms(term_key) => {
                    if let Some(definition) = ticket.decomposed_ticket.terms.get(term_key) {
                        let preview = definition.chars().take(40).collect::<String>();
                        let context = format!("Term: {} = {}...", term_key, preview);
                        let actions = vec![
                            "View full definition".to_string(),
                            "Edit definition".to_string(),
                            "Create refinement ticket".to_string(),
                            "Find related terms".to_string(),
                            "Copy term and definition".to_string(),
                        ];
                        (actions, context)
                    } else {
                        (vec!["Term not found".to_string()], format!("Term: {}", term_key))
                    }
                },
                TicketField::ValidationMethod(index) => {
                    if let Some(method) = ticket.decomposed_ticket.validation_method.get(*index) {
                        let preview = method.chars().take(40).collect::<String>();
                        let context = format!("Validation #{}: {}...", index + 1, preview);
                        let actions = vec![
                            "View full method".to_string(),
                            "Execute validation".to_string(),
                            "Edit method".to_string(),
                            "Create validation ticket".to_string(),
                            "Copy method".to_string(),
                        ];
                        (actions, context)
                    } else {
                        (vec!["Method not found".to_string()], format!("Validation #{}", index + 1))
                    }
                },
                TicketField::OpenQuestion(index) => {
                    if let Some(question) = ticket.decomposed_ticket.open_questions.get(*index) {
                        let preview = question.chars().take(40).collect::<String>();
                        let context = format!("Open Question #{}: {}...", index + 1, preview);
                        let actions = vec![
                            "View full question".to_string(),
                            "Answer question".to_string(),
                            "Create research ticket".to_string(),
                            "Mark as resolved".to_string(),
                            "Copy question".to_string(),
                        ];
                        (actions, context)
                    } else {
                        (vec!["Question not found".to_string()], format!("Open Question #{}", index + 1))
                    }
                },
                TicketField::EngineQuestion(index) => {
                    if let Some(question) = ticket.decomposed_ticket.engine_questions.get(*index) {
                        let preview = question.chars().take(40).collect::<String>();
                        let context = format!("Engine Question #{}: {}...", index + 1, preview);
                        let actions = vec![
                            "View full question".to_string(),
                            "Provide answer".to_string(),
                            "Create investigation ticket".to_string(),
                            "Escalate to expert".to_string(),
                            "Copy question".to_string(),
                        ];
                        (actions, context)
                    } else {
                        (vec!["Question not found".to_string()], format!("Engine Question #{}", index + 1))
                    }
                },
                TicketField::RefinementRequest(index) => {
                    if let Some(request) = ticket.decomposed_ticket.terms_needing_refinement.get(*index) {
                        let priority_emoji = match request.priority {
                            control_flow::ticket::RefinementPriority::Critical => "🔥",
                            control_flow::ticket::RefinementPriority::High => "🟥",
                            control_flow::ticket::RefinementPriority::Medium => "🟨",
                            control_flow::ticket::RefinementPriority::Low => "🟩",
                        };
                        let context = format!("{} Refinement #{}: {} - {}", priority_emoji, index + 1, request.term, request.reason);
                        let actions = vec![
                            "View full request".to_string(),
                            "Create refinement ticket".to_string(),
                            "Change priority".to_string(),
                            "Mark as resolved".to_string(),
                            "Research term".to_string(),
                            "Copy request".to_string(),
                        ];
                        (actions, context)
                    } else {
                        (vec!["Request not found".to_string()], format!("Refinement #{}", index + 1))
                    }
                },
                TicketField::Dependencies => {
                    let context = format!("Dependencies ({})", node.dependencies.len());
                    let actions = vec![
                        "View all dependencies".to_string(),
                        "Add new dependency".to_string(),
                        "Remove dependency".to_string(),
                        "Navigate to dependency".to_string(),
                        "Export dependency list".to_string(),
                    ];
                    (actions, context)
                },
                TicketField::Dependents => {
                    let context = format!("Dependents ({})", node.dependents.len());
                    let actions = vec![
                        "View all dependents".to_string(),
                        "Navigate to dependent".to_string(),
                        "View dependency graph".to_string(),
                        "Export dependents list".to_string(),
                    ];
                    (actions, context)
                },
            }
        } else {
            (vec!["Ticket not found".to_string()], "Unknown".to_string())
        }
    } else {
        (vec!["No project loaded".to_string()], "Unknown".to_string())
    }
}

fn render_ticket_search(frame: &mut Frame, area: Rect, _app: &App, _project_name: &str, _ticket_id: &TicketId, search_state: &SearchState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Search input
            Constraint::Length(3),  // Match counter
            Constraint::Min(0),     // Search results
        ])
        .split(area);

    // Search input
    let search_input = Paragraph::new(format!("/{}", search_state.query))
        .block(Block::default().borders(Borders::ALL).title("Search"))
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(search_input, chunks[0]);

    // Match counter
    let match_info = if search_state.matches.is_empty() {
        "No matches found".to_string()
    } else {
        format!("Match {} of {} | Enter: Next match", 
               search_state.current_match + 1, 
               search_state.matches.len())
    };
    let match_counter = Paragraph::new(match_info)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Green));
    frame.render_widget(match_counter, chunks[1]);

    // Search results
    if !search_state.matches.is_empty() {
        let results: Vec<String> = search_state.matches.iter().enumerate().map(|(i, search_match)| {
            let prefix = if i == search_state.current_match { "► " } else { "  " };
            let field_name = match &search_match.field {
                TicketField::Title => "Title",
                TicketField::RawInput => "Raw Input",
                TicketField::Terms(term) => &format!("Term: {}", term),
                _ => "Other",
            };
            format!("{}{}: {}", prefix, field_name, 
                   search_match.text.chars().take(80).collect::<String>())
        }).collect();

        let results_list = List::new(results.iter().map(|s| ListItem::new(s.as_str())).collect::<Vec<_>>())
            .block(Block::default().borders(Borders::ALL).title("Search Results"))
            .style(Style::default().fg(Color::White));
        
        frame.render_widget(results_list, chunks[2]);
    } else {
        let no_results = Paragraph::new("Type to search through ticket content...")
            .block(Block::default().borders(Borders::ALL).title("Search Results"))
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(no_results, chunks[2]);
    }
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