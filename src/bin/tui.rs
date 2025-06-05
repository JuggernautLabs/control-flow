use control_flow::ticket::{ProjectManager, Project, TicketId, RefinementRequest, RefinementContext, RefinementPriority, TicketDecomposition};
use control_flow::ticket_service::TicketService;
use control_flow::refinement_service::RefinementService;
use client_implementations::claude::ClaudeClient;
use client_implementations::client::RetryConfig;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::sync::mpsc;
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

// Log entry structure for TUI display
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub target: Option<String>,
}

// Thread-safe log collector
#[derive(Debug, Clone)]
pub struct LogCollector {
    entries: Arc<Mutex<VecDeque<LogEntry>>>,
    max_entries: usize,
}

impl LogCollector {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(Mutex::new(VecDeque::new())),
            max_entries,
        }
    }

    pub fn add_entry(&self, entry: LogEntry) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.push_back(entry);
            if entries.len() > self.max_entries {
                entries.pop_front();
            }
        }
    }

    pub fn get_entries(&self) -> Vec<LogEntry> {
        if let Ok(entries) = self.entries.lock() {
            entries.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.clear();
        }
    }
}

// Custom tracing layer for TUI
pub struct TuiLayer {
    collector: LogCollector,
}

impl TuiLayer {
    pub fn new(collector: LogCollector) -> Self {
        Self { collector }
    }
}

impl<S> tracing_subscriber::layer::Layer<S> for TuiLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();
        let level = metadata.level().to_string();
        let target = metadata.target().to_string();

        // Extract the message from the event
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        let entry = LogEntry {
            timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
            level,
            message: visitor.message,
            target: if target.is_empty() { None } else { Some(target) },
        };

        self.collector.add_entry(entry);
    }
}

// Visitor to extract message from tracing events
#[derive(Default)]
struct MessageVisitor {
    message: String,
}

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }
}

#[derive(Debug)]
pub enum BackgroundTaskResult {
    RefinementComplete {
        project_name: String,
        parent_ticket_id: TicketId,
        term: String,
        ticket: TicketDecomposition,
    },
    TicketComplete {
        project_name: String,
        ticket: TicketDecomposition,
    },
    TaskError {
        error: String,
    },
}

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
    TicketFieldEdit(String, TicketId, FieldEditState), // project name, ticket id, edit state
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
pub struct FieldEditState {
    pub field: TicketField,
    pub edit_type: EditType,
    pub current_value: String,
    pub original_value: String,
    pub is_modified: bool,
}

#[derive(Debug, Clone)]
pub enum EditType {
    TextEdit,        // Free text editing
    StatusSelect,    // Status dropdown selection
    PrioritySelect,  // Priority dropdown selection
    ComplexitySelect, // Complexity dropdown selection
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
    pub refinement_service: RefinementService<ClaudeClient>,
    pub items: Vec<String>, // Current menu items
    pub ticket_fields: Vec<TicketField>, // Available fields in ticket details view
    pub field_actions: Vec<String>, // Available actions for selected field
    pub should_quit: bool,
    pub progress_counter: usize, // For animated progress indicators
    pub log_collector: LogCollector, // For capturing and displaying logs
    pub show_logs: bool, // Toggle for log pane visibility
    pub task_receiver: mpsc::Receiver<BackgroundTaskResult>, // Receive background task results
    pub task_sender: mpsc::Sender<BackgroundTaskResult>, // Send background task results
    pub pending_project_name: Option<String>, // Store project name during creation
}

impl App {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();
        
        let api_key = env::var("ANTHROPIC_API_KEY")
            .map_err(|_| "ANTHROPIC_API_KEY environment variable must be set. Please run: export ANTHROPIC_API_KEY=your_api_key")?;
        
        let claude_client = ClaudeClient::new(api_key.clone());
        let retry_config = RetryConfig::default();
        let ticket_service = TicketService::new(claude_client.clone(), retry_config.clone());
        let refinement_service = RefinementService::new(claude_client, retry_config);
        
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

        let (task_sender, task_receiver) = mpsc::channel();

        Ok(App {
            state: AppState::MainMenu,
            previous_state: None,
            selected_index: 0,
            scroll_offset: 0,
            project_manager,
            current_project: None,
            ticket_service,
            refinement_service,
            items,
            ticket_fields: Vec::new(),
            field_actions: Vec::new(),
            should_quit: false,
            progress_counter: 0,
            log_collector: LogCollector::new(100), // Keep last 100 log entries
            show_logs: false, // Initially hidden, can be toggled with a key
            task_receiver,
            task_sender,
            pending_project_name: None,
        })
    }

    pub fn new_with_log_collector(log_collector: LogCollector) -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();
        
        let api_key = env::var("ANTHROPIC_API_KEY")
            .map_err(|_| "ANTHROPIC_API_KEY environment variable must be set. Please run: export ANTHROPIC_API_KEY=your_api_key")?;
        
        let claude_client = ClaudeClient::new(api_key.clone());
        let retry_config = RetryConfig::default();
        let ticket_service = TicketService::new(claude_client.clone(), retry_config.clone());
        let refinement_service = RefinementService::new(claude_client, retry_config);
        
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

        let (task_sender, task_receiver) = mpsc::channel();

        Ok(App {
            state: AppState::MainMenu,
            previous_state: None,
            selected_index: 0,
            scroll_offset: 0,
            project_manager,
            current_project: None,
            ticket_service,
            refinement_service,
            items,
            ticket_fields: Vec::new(),
            field_actions: Vec::new(),
            should_quit: false,
            progress_counter: 0,
            log_collector, // Use the provided log collector
            show_logs: false, // Initially hidden, can be toggled with a key
            task_receiver,
            task_sender,
            pending_project_name: None,
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
                AppState::TicketFieldEdit(project_name, ticket_id, _) => {
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
                AppState::CreateProject | AppState::MainMenu => {
                    // After project creation or from main menu, go back to main menu
                    self.state = AppState::MainMenu;
                    self.update_main_menu_items();
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
        // Handle input states first (highest priority - don't interfere with typing)
        match &mut self.state {
            AppState::Input(ref mut input_state) => {
                match key {
                    KeyCode::Char(c) => {
                        input_state.input.push(c);
                    }
                    KeyCode::Backspace => {
                        input_state.input.pop();
                    }
                    KeyCode::Enter => {
                        self.handle_enter()?;
                    }
                    KeyCode::Esc => {
                        self.go_back();
                    }
                    _ => {} // Ignore other keys in input mode
                }
                return Ok(());
            }
            AppState::TicketSearch(_, _, ref mut search_state) => {
                match key {
                    KeyCode::Char(c) => {
                        search_state.query.push(c);
                        // Update search results based on new query
                        // This would trigger a search in a real implementation
                    }
                    KeyCode::Backspace => {
                        search_state.query.pop();
                    }
                    KeyCode::Enter => {
                        // Move to next search result
                        if !search_state.matches.is_empty() {
                            search_state.current_match = (search_state.current_match + 1) % search_state.matches.len();
                        }
                    }
                    KeyCode::Esc => {
                        // Exit search mode and return to ticket details view
                        if let AppState::TicketSearch(project_name, ticket_id, _) = &self.state {
                            let project_name = project_name.clone();
                            let ticket_id = ticket_id.clone();
                            self.show_ticket_details_view(project_name, ticket_id)?;
                        }
                    }
                    _ => {} // Ignore other keys in search mode
                }
                return Ok(());
            }
            AppState::TicketFieldEdit(_, _, ref mut edit_state) => {
                match key {
                    KeyCode::Char(c) => {
                        // Handle character input in edit mode
                        if matches!(edit_state.edit_type, EditType::TextEdit) {
                            edit_state.current_value.push(c);
                            edit_state.is_modified = edit_state.current_value != edit_state.original_value;
                        }
                    }
                    KeyCode::Backspace => {
                        if matches!(edit_state.edit_type, EditType::TextEdit) {
                            edit_state.current_value.pop();
                            edit_state.is_modified = edit_state.current_value != edit_state.original_value;
                        }
                    }
                    KeyCode::Enter => {
                        self.handle_enter()?;
                    }
                    KeyCode::Esc => {
                        // Cancel field editing and return to ticket details view
                        if let AppState::TicketFieldEdit(project_name, ticket_id, _) = &self.state {
                            let project_name = project_name.clone();
                            let ticket_id = ticket_id.clone();
                            self.show_ticket_details_view(project_name, ticket_id)?;
                        }
                    }
                    _ => {} // Ignore other keys in edit mode
                }
                return Ok(());
            }
            _ => {} // Continue to global key handling for other states
        }

        // Global key handling (only when not in input/edit/search modes)
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
            KeyCode::Char('/') => {
                // Enter search mode if in ticket details view
                if let AppState::TicketDetailsView(project_name, ticket_id) = &self.state {
                    let project_name = project_name.clone();
                    let ticket_id = ticket_id.clone();
                    self.start_ticket_search(project_name, ticket_id)?;
                }
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                // Toggle log pane visibility (only when not in input mode)
                self.show_logs = !self.show_logs;
            }
            KeyCode::Up => self.move_up(),
            KeyCode::Down => self.move_down(),
            KeyCode::Enter => self.handle_enter()?,
            KeyCode::Char(c) => {
                // Handle number keys for action execution in ticket details view
                if let AppState::TicketDetailsView(project_name, ticket_id) = &self.state {
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
                // Other character handling is now done in state-specific sections above
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
            AppState::TicketFieldEdit(project_name, ticket_id, edit_state) => {
                let project_name = project_name.clone();
                let ticket_id = ticket_id.clone();
                let edit_state = edit_state.clone();
                self.handle_field_edit_submit(project_name, ticket_id, edit_state)?;
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
                // "← Back to ticket details" - return to the ticket details view
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
                // "← Back to ticket list"
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
                
                // Add navigation options
                items.push("⚙️ Ticket Actions Menu".to_string());
                items.push("← Back to ticket list".to_string());
                
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
            // "← Back to ticket list" selected (always last item)
            self.show_ticket_list(project_name)?;
        } else if self.selected_index == self.items.len() - 2 {
            // "⚙️ Ticket Actions Menu" selected (second to last item)
            self.show_ticket_actions_menu(project_name, ticket_id)?;
        } else if self.selected_index < self.ticket_fields.len() {
            // A field was selected - execute the first action for that field (most common action)
            let field = self.ticket_fields[self.selected_index].clone();
            self.execute_field_action(project_name, ticket_id, field, 0)?;
        }
        Ok(())
    }

    fn execute_field_action(&mut self, project_name: String, ticket_id: TicketId, field: TicketField, action_index: usize) -> Result<(), Box<dyn std::error::Error>> {
        // Get the available actions for this field
        let (actions, _) = get_field_actions_with_content(self, &ticket_id, &field);
        
        if action_index < actions.len() {
            let action = &actions[action_index];
            
            // Handle different action types
            if action.contains("Edit") || action.contains("edit") {
                self.start_field_edit(project_name, ticket_id, field)?;
            } else if action.contains("View full") || action.contains("View") {
                self.show_field_detail_view(project_name, ticket_id, field)?;
            } else if action.contains("Change") {
                self.start_field_selection(project_name, ticket_id, field)?;
            } else if action.contains("Create refinement ticket") {
                self.start_term_refinement(project_name, ticket_id, field)?;
            } else if action.contains("Find related terms") {
                self.find_related_terms(project_name, ticket_id, field)?;
            } else if action.contains("Answer question") {
                self.start_question_answering(project_name, ticket_id, field)?;
            } else if action.contains("Add new dependency") {
                self.start_dependency_selection(project_name, ticket_id)?;
            } else if action.contains("Copy") {
                self.copy_field_content(field)?;
            } else if action.contains("Create research ticket") {
                // TODO: Implement research ticket creation
                let field_name = self.get_field_display_name(&field);
                self.transition_to_state(AppState::Loading(format!("✅ Research ticket creation ({})", field_name)));
            } else if action.contains("Mark as resolved") {
                // TODO: Implement question resolution
                let field_name = self.get_field_display_name(&field);
                self.transition_to_state(AppState::Loading(format!("✅ Marked as resolved ({})", field_name)));
            } else if action.contains("Navigate to") {
                // TODO: Implement navigation to related tickets
                let field_name = self.get_field_display_name(&field);
                self.transition_to_state(AppState::Loading(format!("Navigation to related tickets ({})", field_name)));
            } else {
                // For other actions, show loading message for now
                let field_name = self.get_field_display_name(&field);
                self.transition_to_state(AppState::Loading(format!("Executing: {} on {}", action, field_name)));
                // TODO: Implement remaining action types
            }
        } else {
            self.transition_to_state(AppState::Error("Invalid action selected".to_string()));
        }
        
        Ok(())
    }

    fn start_field_edit(&mut self, project_name: String, ticket_id: TicketId, field: TicketField) -> Result<(), Box<dyn std::error::Error>> {
        // Get the current value of the field
        let (current_value, edit_type) = self.get_field_current_value(&ticket_id, &field)?;
        
        let edit_state = FieldEditState {
            field: field.clone(),
            edit_type,
            current_value: current_value.clone(),
            original_value: current_value,
            is_modified: false,
        };
        
        self.state = AppState::TicketFieldEdit(project_name, ticket_id, edit_state);
        Ok(())
    }

    fn show_field_detail_view(&mut self, _project_name: String, _ticket_id: TicketId, field: TicketField) -> Result<(), Box<dyn std::error::Error>> {
        // For now, just show a loading message - this could be expanded to show full content
        let field_name = self.get_field_display_name(&field);
        self.transition_to_state(AppState::Loading(format!("Viewing full content of {}", field_name)));
        // TODO: Implement detailed view for large content
        Ok(())
    }

    fn start_field_selection(&mut self, project_name: String, ticket_id: TicketId, field: TicketField) -> Result<(), Box<dyn std::error::Error>> {
        // For status/priority/complexity fields, start selection mode
        let edit_type = match &field {
            TicketField::Status => EditType::StatusSelect,
            TicketField::Priority => EditType::PrioritySelect,
            TicketField::Complexity => EditType::ComplexitySelect,
            _ => return Err("Field does not support selection".into()),
        };
        
        let (current_value, _) = self.get_field_current_value(&ticket_id, &field)?;
        
        let edit_state = FieldEditState {
            field: field.clone(),
            edit_type,
            current_value: current_value.clone(),
            original_value: current_value,
            is_modified: false,
        };
        
        self.state = AppState::TicketFieldEdit(project_name, ticket_id, edit_state);
        Ok(())
    }

    fn get_field_current_value(&self, ticket_id: &TicketId, field: &TicketField) -> Result<(String, EditType), Box<dyn std::error::Error>> {
        if let Some(project) = &self.current_project {
            if let Some(node) = project.graph.get_ticket(ticket_id) {
                let ticket = &node.ticket;
                
                let (value, edit_type) = match field {
                    TicketField::Title => (ticket.original_ticket.title.clone(), EditType::TextEdit),
                    TicketField::RawInput => (ticket.original_ticket.raw_input.clone(), EditType::TextEdit),
                    TicketField::Status => (format!("{:?}", ticket.decomposed_ticket.metadata.status), EditType::StatusSelect),
                    TicketField::Priority => (format!("{:?}", ticket.decomposed_ticket.metadata.priority), EditType::PrioritySelect),
                    TicketField::Complexity => (format!("{:?}", ticket.decomposed_ticket.metadata.estimated_complexity), EditType::ComplexitySelect),
                    TicketField::Terms(term_key) => {
                        let definition = ticket.decomposed_ticket.terms.get(term_key)
                            .cloned()
                            .unwrap_or_else(|| "Definition not found".to_string());
                        (definition, EditType::TextEdit)
                    },
                    TicketField::ValidationMethod(index) => {
                        let method = ticket.decomposed_ticket.validation_method.get(*index)
                            .cloned()
                            .unwrap_or_else(|| "Method not found".to_string());
                        (method, EditType::TextEdit)
                    },
                    TicketField::OpenQuestion(index) => {
                        let question = ticket.decomposed_ticket.open_questions.get(*index)
                            .cloned()
                            .unwrap_or_else(|| "Question not found".to_string());
                        (question, EditType::TextEdit)
                    },
                    TicketField::EngineQuestion(index) => {
                        let question = ticket.decomposed_ticket.engine_questions.get(*index)
                            .cloned()
                            .unwrap_or_else(|| "Question not found".to_string());
                        (question, EditType::TextEdit)
                    },
                    TicketField::RefinementRequest(index) => {
                        let request = ticket.decomposed_ticket.terms_needing_refinement.get(*index);
                        let text = if let Some(req) = request {
                            format!("{} - {}", req.term, req.reason)
                        } else {
                            "Request not found".to_string()
                        };
                        (text, EditType::TextEdit)
                    },
                    _ => return Err("Field type not supported for editing".into()),
                };
                
                Ok((value, edit_type))
            } else {
                Err("Ticket not found".into())
            }
        } else {
            Err("No project loaded".into())
        }
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

    fn start_term_refinement(&mut self, project_name: String, ticket_id: TicketId, field: TicketField) -> Result<(), Box<dyn std::error::Error>> {
        if let TicketField::Terms(term_key) = &field {
            if let Some(project) = &self.current_project {
                if let Some(node) = project.graph.get_ticket(&ticket_id) {
                    let ticket = &node.ticket;
                    
                    // Get the term definition for context
                    let term_definition = ticket.decomposed_ticket.terms.get(term_key)
                        .cloned()
                        .unwrap_or_else(|| "Definition not found".to_string());
                    
                    // Build refinement context
                    let refinement_context = format!(
                        "Context from parent ticket '{}': {}\n\nTerm being refined: {}\nCurrent definition: {}\n\nPlease provide a more detailed definition for this term.",
                        ticket.original_ticket.title,
                        ticket.original_ticket.raw_input,
                        term_key,
                        term_definition
                    );
                    
                    self.transition_to_state(AppState::Loading("🔄 Creating refinement ticket for term...".to_string()));
                    
                    // Create refinement ticket asynchronously
                    self.handle_refinement_creation_async(project_name, ticket_id, term_key.clone(), refinement_context)?;
                } else {
                    self.transition_to_state(AppState::Error("Ticket not found".to_string()));
                }
            } else {
                self.transition_to_state(AppState::Error("No project loaded".to_string()));
            }
        } else {
            self.transition_to_state(AppState::Error("Invalid field for term refinement".to_string()));
        }
        
        Ok(())
    }
    
    fn handle_refinement_creation_async(&mut self, project_name: String, parent_ticket_id: TicketId, term: String, context: String) -> Result<(), Box<dyn std::error::Error>> {
        // Create refinement request and context objects
        let refinement_request = RefinementRequest {
            term: term.clone(),
            context: context.clone(),
            reason: format!("Term '{}' needs refinement for clarity", term),
            priority: RefinementPriority::Medium,
        };
        
        let refinement_context = RefinementContext {
            parent_ticket_id: parent_ticket_id.clone(),
            term_being_refined: term.clone(),
            original_context: context,
            additional_context: vec![],
        };
        
        // Since we're running in an async main, we need to use a blocking approach
        // We'll use std::thread to avoid the "runtime within runtime" issue
        let api_key = env::var("ANTHROPIC_API_KEY")
            .map_err(|_| "ANTHROPIC_API_KEY environment variable not set")?;
        
        // Launch background task (non-blocking)
        let sender = self.task_sender.clone();
        let project_name_clone = project_name.clone();
        let parent_ticket_id_clone = parent_ticket_id.clone();
        let term_clone = term.clone();
        
        std::thread::spawn(move || {
            // Create a new runtime in this thread
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                // Create new client and service in this thread
                let claude_client = ClaudeClient::new(api_key);
                let config = RetryConfig::default();
                let refinement_service = RefinementService::new(claude_client, config);
                
                refinement_service.refine_term(&refinement_request, Some(&refinement_context)).await
            });
            
            // Send result back to main thread
            match result {
                Ok(ticket) => {
                    let _ = sender.send(BackgroundTaskResult::RefinementComplete {
                        project_name: project_name_clone,
                        parent_ticket_id: parent_ticket_id_clone,
                        term: term_clone,
                        ticket,
                    });
                }
                Err(e) => {
                    let _ = sender.send(BackgroundTaskResult::TaskError {
                        error: format!("Refinement failed: {}", e),
                    });
                }
            }
        });
        
        // Return immediately - the result will come via the channel
        Ok(())
    }

    fn handle_background_task_completion(&mut self, result: BackgroundTaskResult) -> Result<(), Box<dyn std::error::Error>> {
        match result {
            BackgroundTaskResult::RefinementComplete { 
                project_name, 
                parent_ticket_id, 
                term, 
                ticket 
            } => {
                // Add refinement ticket to current project
                if let Some(project) = &mut self.current_project {
                    let refinement_ticket_id = project.add_ticket(ticket);
                    
                    // Link as dependency (refinement depends on parent)
                    if let Err(e) = project.graph.add_dependency(&refinement_ticket_id, &parent_ticket_id) {
                        self.transition_to_state(AppState::Error(format!("Failed to link refinement dependency: {}", e)));
                        return Ok(());
                    }
                    
                    // Save the project
                    if let Err(e) = self.save_current_project() {
                        self.transition_to_state(AppState::Error(e.to_string()));
                        return Ok(());
                    }
                    
                    // Navigate to the refinement ticket
                    self.show_ticket_detail(project_name, refinement_ticket_id.clone())?;
                    
                    // Show success message
                    self.transition_to_state(AppState::Loading(format!("✅ Refinement ticket created for term '{}': {}", term, refinement_ticket_id)));
                } else {
                    self.transition_to_state(AppState::Error("No project loaded".to_string()));
                }
            }
            BackgroundTaskResult::TicketComplete { project_name, ticket } => {
                // Add ticket to current project
                if let Some(project) = &mut self.current_project {
                    let ticket_id = project.add_ticket(ticket);
                    
                    // Save the project
                    if let Err(e) = self.save_current_project() {
                        self.transition_to_state(AppState::Error(e.to_string()));
                        return Ok(());
                    }
                    
                    // Navigate to the new ticket
                    self.show_ticket_detail(project_name, ticket_id.clone())?;
                    
                    // Show success message briefly
                    self.transition_to_state(AppState::Loading(format!("✅ Ticket created successfully! ID: {}", ticket_id)));
                } else {
                    self.transition_to_state(AppState::Error("No project loaded".to_string()));
                }
            }
            BackgroundTaskResult::TaskError { error } => {
                self.transition_to_state(AppState::Error(error));
            }
        }
        Ok(())
    }
    
    fn find_related_terms(&mut self, _project_name: String, ticket_id: TicketId, field: TicketField) -> Result<(), Box<dyn std::error::Error>> {
        if let TicketField::Terms(term_key) = &field {
            // Search across all tickets in the project for related terms
            if let Some(project) = &self.current_project {
                let mut related_terms = Vec::new();
                let search_term = term_key.to_lowercase();
                
                // Search through all tickets
                for (id, node) in &project.graph.nodes {
                    if *id == ticket_id {
                        continue; // Skip current ticket
                    }
                    
                    let ticket = &node.ticket;
                    
                    // Search in terms
                    for (term, definition) in &ticket.decomposed_ticket.terms {
                        if term.to_lowercase().contains(&search_term) || 
                           definition.to_lowercase().contains(&search_term) ||
                           search_term.contains(&term.to_lowercase()) {
                            related_terms.push(format!("🎫 {}: {} = {}", id, term, definition));
                        }
                    }
                    
                    // Search in title and raw input
                    if ticket.original_ticket.title.to_lowercase().contains(&search_term) ||
                       ticket.original_ticket.raw_input.to_lowercase().contains(&search_term) {
                        related_terms.push(format!("🎫 {}: {} (in content)", id, ticket.original_ticket.title));
                    }
                }
                
                if related_terms.is_empty() {
                    self.transition_to_state(AppState::Loading(format!("No related terms found for '{}'", term_key)));
                } else {
                    // For now, show loading message with results count
                    // TODO: Create a dedicated related terms view
                    self.transition_to_state(AppState::Loading(format!("✅ Found {} related references to '{}' across project", related_terms.len(), term_key)));
                }
            } else {
                self.transition_to_state(AppState::Error("No project loaded".to_string()));
            }
        } else {
            self.transition_to_state(AppState::Error("Invalid field for term search".to_string()));
        }
        
        Ok(())
    }
    
    fn start_question_answering(&mut self, project_name: String, ticket_id: TicketId, field: TicketField) -> Result<(), Box<dyn std::error::Error>> {
        // Get the question text based on field type
        let question_text = match &field {
            TicketField::OpenQuestion(index) => {
                if let Some(project) = &self.current_project {
                    if let Some(node) = project.graph.get_ticket(&ticket_id) {
                        node.ticket.decomposed_ticket.open_questions.get(*index).cloned()
                    } else { None }
                } else { None }
            }
            TicketField::EngineQuestion(index) => {
                if let Some(project) = &self.current_project {
                    if let Some(node) = project.graph.get_ticket(&ticket_id) {
                        node.ticket.decomposed_ticket.engine_questions.get(*index).cloned()
                    } else { None }
                } else { None }
            }
            _ => {
                self.transition_to_state(AppState::Error("Invalid field for question answering".to_string()));
                return Ok(());
            }
        };
        
        if let Some(question) = question_text {
            // Start input state for question answering
            self.state = AppState::Input(InputState {
                title: "Answer Question".to_string(),
                prompt: format!("Question: {}\n\nEnter your answer:", question),
                input: String::new(),
                return_state: Box::new(AppState::TicketDetailsView(project_name, ticket_id)),
            });
        } else {
            self.transition_to_state(AppState::Error("Question not found".to_string()));
        }
        
        Ok(())
    }
    
    fn start_dependency_selection(&mut self, project_name: String, ticket_id: TicketId) -> Result<(), Box<dyn std::error::Error>> {
        // Load the project to get all available tickets
        let project = self.project_manager.load_project(&project_name)?;
        
        // Get all ticket IDs except the current one (can't depend on itself)
        let available_tickets: Vec<(TicketId, String)> = project.graph.nodes.iter()
            .filter(|(id, _)| **id != ticket_id)
            .map(|(id, node)| (id.clone(), node.ticket.original_ticket.title.clone()))
            .collect();
            
        if available_tickets.is_empty() {
            self.transition_to_state(AppState::Error("No other tickets available to create dependencies".to_string()));
            return Ok(());
        }
        
        // Create a list selection interface
        let prompt = format!(
            "Select a ticket to add as a dependency to '{}'\n\nPress number to select, or ESC to cancel:",
            project.graph.get_ticket(&ticket_id)
                .map(|n| n.ticket.original_ticket.title.as_str())
                .unwrap_or("Unknown")
        );
        
        // Store the available tickets in a simplified format for the user
        let mut ticket_list = String::new();
        for (i, (_, title)) in available_tickets.iter().enumerate() {
            ticket_list.push_str(&format!("{}. {}\n", i + 1, title));
        }
        
        // For now, show the list and return to previous state
        // In a full implementation, this would be a new state with ticket selection
        let message = format!("{}\n\n{}\n(Dependency selection interface - press ESC to return)", prompt, ticket_list);
        self.transition_to_state(AppState::Loading(message));
        
        Ok(())
    }
    
    fn copy_field_content(&mut self, field: TicketField) -> Result<(), Box<dyn std::error::Error>> {
        // Get the current project and ticket
        let (project_name, ticket_id) = match &self.state {
            AppState::TicketFieldAction(proj, tid, _) => (proj.clone(), tid.clone()),
            AppState::TicketDetailsView(proj, tid) => (proj.clone(), tid.clone()),
            _ => return Err("No active ticket context for copying".into()),
        };
        
        let project = self.project_manager.load_project(&project_name)?;
        let ticket = project.graph.get_ticket(&ticket_id)
            .ok_or("Ticket not found")?;
            
        // Get the content to copy based on field type
        let content = match &field {
            TicketField::Title => ticket.ticket.original_ticket.title.clone(),
            TicketField::RawInput => ticket.ticket.original_ticket.raw_input.clone(),
            TicketField::Status => format!("{:?}", ticket.ticket.decomposed_ticket.metadata.status),
            TicketField::Priority => format!("{:?}", ticket.ticket.decomposed_ticket.metadata.priority),
            TicketField::Complexity => format!("{:?}", ticket.ticket.decomposed_ticket.metadata.estimated_complexity),
            TicketField::Terms(term_key) => {
                ticket.ticket.decomposed_ticket.terms.get(term_key)
                    .map(|def| format!("{}: {}", term_key, def))
                    .unwrap_or_else(|| format!("{}: [No definition]", term_key))
            },
            TicketField::ValidationMethod(index) => {
                ticket.ticket.decomposed_ticket.validation_method.get(*index)
                    .cloned()
                    .unwrap_or_else(|| "[Method not found]".to_string())
            },
            TicketField::OpenQuestion(index) => {
                ticket.ticket.decomposed_ticket.open_questions.get(*index)
                    .cloned()
                    .unwrap_or_else(|| "[Question not found]".to_string())
            },
            TicketField::EngineQuestion(index) => {
                ticket.ticket.decomposed_ticket.engine_questions.get(*index)
                    .cloned()
                    .unwrap_or_else(|| "[Question not found]".to_string())
            },
            TicketField::RefinementRequest(index) => {
                ticket.ticket.decomposed_ticket.terms_needing_refinement.get(*index)
                    .map(|req| format!("Term: {} | Reason: {}", req.term, req.reason))
                    .unwrap_or_else(|| "[Request not found]".to_string())
            },
            TicketField::Dependencies => {
                format!("Dependencies: {}", ticket.dependencies.len())
            },
            TicketField::Dependents => {
                format!("Dependents: {}", ticket.dependents.len())
            },
        };
        
        // Try to copy to system clipboard using pbcopy (macOS) or xclip (Linux)
        // This is a best-effort implementation
        let copy_result = if cfg!(target_os = "macos") {
            std::process::Command::new("pbcopy")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    if let Some(stdin) = child.stdin.take() {
                        use std::io::Write;
                        let mut stdin = stdin;
                        stdin.write_all(content.as_bytes())?;
                        drop(stdin);
                    }
                    child.wait()
                })
        } else {
            // Try xclip for Linux
            std::process::Command::new("xclip")
                .args(["-selection", "clipboard"])
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    if let Some(stdin) = child.stdin.take() {
                        use std::io::Write;
                        let mut stdin = stdin;
                        stdin.write_all(content.as_bytes())?;
                        drop(stdin);
                    }
                    child.wait()
                })
        };
        
        let field_name = self.get_field_display_name(&field);
        match copy_result {
            Ok(_) => {
                self.transition_to_state(AppState::Loading(format!("✅ {} content copied to clipboard", field_name)));
            },
            Err(_) => {
                // Fallback: show the content to user for manual copying
                let message = format!("📋 {} content (copy manually):\n\n{}", field_name, content);
                self.transition_to_state(AppState::Loading(message));
            }
        }
        
        Ok(())
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

    fn handle_field_edit_submit(&mut self, project_name: String, ticket_id: TicketId, edit_state: FieldEditState) -> Result<(), Box<dyn std::error::Error>> {
        if edit_state.is_modified {
            // Save the changes
            self.transition_to_state(AppState::Loading(format!("💾 Saving changes to {}", self.get_field_display_name(&edit_state.field))));
            
            // Update the ticket in memory
            if let Some(project) = &mut self.current_project {
                if let Some(node) = project.graph.get_ticket_mut(&ticket_id) {
                    let ticket = &mut node.ticket;
                    
                    // Apply the field changes
                    match &edit_state.field {
                        TicketField::Title => {
                            ticket.original_ticket.title = edit_state.current_value.clone();
                        }
                        TicketField::RawInput => {
                            ticket.original_ticket.raw_input = edit_state.current_value.clone();
                        }
                        TicketField::Status => {
                            if let Ok(status) = parse_status(&edit_state.current_value) {
                                ticket.decomposed_ticket.metadata.status = status;
                            }
                        }
                        TicketField::Priority => {
                            if let Ok(priority) = parse_priority(&edit_state.current_value) {
                                ticket.decomposed_ticket.metadata.priority = priority;
                            }
                        }
                        TicketField::Complexity => {
                            if let Ok(complexity) = parse_complexity(&edit_state.current_value) {
                                ticket.decomposed_ticket.metadata.estimated_complexity = complexity;
                            }
                        }
                        TicketField::Terms(term_key) => {
                            ticket.decomposed_ticket.terms.insert(term_key.clone(), edit_state.current_value.clone());
                        }
                        TicketField::ValidationMethod(index) => {
                            if *index < ticket.decomposed_ticket.validation_method.len() {
                                ticket.decomposed_ticket.validation_method[*index] = edit_state.current_value.clone();
                            }
                        }
                        TicketField::OpenQuestion(index) => {
                            if *index < ticket.decomposed_ticket.open_questions.len() {
                                ticket.decomposed_ticket.open_questions[*index] = edit_state.current_value.clone();
                            }
                        }
                        TicketField::EngineQuestion(index) => {
                            if *index < ticket.decomposed_ticket.engine_questions.len() {
                                ticket.decomposed_ticket.engine_questions[*index] = edit_state.current_value.clone();
                            }
                        }
                        _ => {
                            self.transition_to_state(AppState::Error("Field type not supported for editing".to_string()));
                            return Ok(());
                        }
                    }
                    
                    // Update the node's timestamp
                    node.updated_at = chrono::Utc::now().to_rfc3339();
                    
                    // Save the project to disk
                    if let Err(e) = self.project_manager.save_project(project) {
                        self.transition_to_state(AppState::Error(format!("Failed to save project: {}", e)));
                        return Ok(());
                    }
                    
                    // Show success and return to ticket view
                    self.transition_to_state(AppState::Loading("✅ Changes saved successfully!".to_string()));
                    
                    // Refresh the ticket details view to show updated content
                    self.show_ticket_details_view(project_name, ticket_id)?;
                } else {
                    self.transition_to_state(AppState::Error("Ticket not found".to_string()));
                }
            } else {
                self.transition_to_state(AppState::Error("No project loaded".to_string()));
            }
        } else {
            // No changes, just return to ticket view
            self.show_ticket_details_view(project_name, ticket_id)?;
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
                AppState::TicketDetailsView(ref project_name, ref ticket_id) => {
                    // This could be a question answer
                    if input_state.title.contains("Answer Question") {
                        self.handle_question_answer(project_name.clone(), ticket_id.clone(), input_text)?;
                    } else {
                        self.state = return_state;
                    }
                },
                _ => {
                    self.state = return_state;
                }
            }
        }
        Ok(())
    }
    
    fn handle_question_answer(&mut self, project_name: String, ticket_id: TicketId, answer: String) -> Result<(), Box<dyn std::error::Error>> {
        // For now, just show a success message
        // TODO: Implement actual question answer storage and processing
        if answer.trim().is_empty() {
            self.transition_to_state(AppState::Error("Answer cannot be empty".to_string()));
        } else {
            self.transition_to_state(AppState::Loading(format!("✅ Question answered: '{}'", answer.chars().take(50).collect::<String>())));
            // Return to ticket details view
            self.show_ticket_details_view(project_name, ticket_id)?;
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
            AppState::TicketDetailsView(project_name, _) => {
                let project_name = project_name.clone();
                self.show_ticket_list(project_name).ok();
            },
            AppState::TicketSearch(project_name, ticket_id, _) => {
                let project_name = project_name.clone();
                let ticket_id = ticket_id.clone();
                self.show_ticket_details_view(project_name, ticket_id).ok();
            },
            AppState::TicketFieldEdit(project_name, ticket_id, _) => {
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
        // Validate project name
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            self.transition_to_state(AppState::Error("Project name cannot be empty. Please try again.".to_string()));
            return Ok(());
        }
        
        // Check if project already exists
        let existing_projects = self.project_manager.list_projects();
        if existing_projects.iter().any(|p| p.as_str() == trimmed_name) {
            self.transition_to_state(AppState::Error(format!("Project '{}' already exists. Please choose a different name.", trimmed_name)));
            return Ok(());
        }
        
        // Store the name and ask for description
        self.pending_project_name = Some(trimmed_name.to_string());
        self.state = AppState::Input(InputState {
            title: format!("Create Project: {}", trimmed_name),
            prompt: "Enter project description:".to_string(),
            input: String::new(),
            return_state: Box::new(AppState::CreateProject),
        });
        Ok(())
    }

    fn finalize_project_creation(&mut self, description: String) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(name) = self.pending_project_name.take() {
            // Actually create the project
            match self.project_manager.create_project(name.clone(), description) {
                Ok(_) => {
                    // Show success message and then go to main menu
                    self.transition_to_state(AppState::Loading(format!("✅ Project '{}' created successfully!", name)));
                    // The auto-transition will take us back to main menu after 2 seconds
                },
                Err(e) => {
                    self.transition_to_state(AppState::Error(format!("Failed to create project '{}': {}", name, e)));
                }
            }
        } else {
            self.transition_to_state(AppState::Error("Project name not found - please try again".to_string()));
        }
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
        // Go directly to ticket details view (show content immediately)
        self.show_ticket_details_view(project_name, ticket_id)
    }

    fn show_ticket_actions_menu(&mut self, project_name: String, ticket_id: TicketId) -> Result<(), Box<dyn std::error::Error>> {
        // Update items based on current ticket status
        if let Some(project) = &self.current_project {
            if let Some(node) = project.graph.get_ticket(&ticket_id) {
                let refinement_count = node.ticket.decomposed_ticket.terms_needing_refinement.len();
                let dependencies_count = node.dependencies.len();
                let dependents_count = node.dependents.len();
                
                self.items = vec![
                    "← Back to ticket details".to_string(),
                    format!("View refinement requests ({})", refinement_count),
                    format!("Quick refine - show all terms ({})", refinement_count),
                    format!("View dependencies ({})", dependencies_count),
                    format!("View dependents ({})", dependents_count),
                    "← Back to ticket list".to_string(),
                ];
            } else {
                self.items = vec![
                    "← Back to ticket details".to_string(),
                    "View refinement requests".to_string(),
                    "Quick refine - show all terms".to_string(),
                    "View dependencies".to_string(),
                    "View dependents".to_string(),
                    "← Back to ticket list".to_string(),
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

    fn create_ticket_with_description(&mut self, project_name: String, description: String) -> Result<(), Box<dyn std::error::Error>> {
        self.transition_to_state(AppState::Loading("🔄 Creating ticket with AI decomposition...".to_string()));
        
        // Store the creation parameters for async processing
        // For now, we'll simulate the async operation by transitioning to a completion state
        // In a real implementation, this would spawn a background task
        self.handle_ticket_creation_async(project_name, description)
    }
    
    fn handle_ticket_creation_async(&mut self, project_name: String, description: String) -> Result<(), Box<dyn std::error::Error>> {
        // Since we're running in an async main, use a separate thread to avoid runtime conflicts
        let api_key = env::var("ANTHROPIC_API_KEY")
            .map_err(|_| "ANTHROPIC_API_KEY environment variable not set")?;
        
        // Launch background task (non-blocking)
        let sender = self.task_sender.clone();
        let project_name_clone = project_name.clone();
        
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                let claude_client = ClaudeClient::new(api_key);
                let config = RetryConfig::default();
                let ticket_service = TicketService::new(claude_client, config);
                
                ticket_service.decompose_ticket(description).await
            });
            
            // Send result back to main thread
            match result {
                Ok(ticket) => {
                    let _ = sender.send(BackgroundTaskResult::TicketComplete {
                        project_name: project_name_clone,
                        ticket,
                    });
                }
                Err(e) => {
                    let _ = sender.send(BackgroundTaskResult::TaskError {
                        error: format!("Ticket creation failed: {}", e),
                    });
                }
            }
        });
        
        // Return immediately - the result will come via the channel
        Ok(())
    }

    fn save_current_project(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.current_project {
            let project_name = project.name.clone();
            
            // Try to save with automatic recovery
            match self.project_manager.save_project_with_recovery(project) {
                Ok(_) => {
                    // Success! Log the save for debugging
                    tracing::info!("Successfully saved project '{}'", project_name);
                    Ok(())
                }
                Err(e) => {
                    // Even the recovery failed, provide detailed diagnostic info
                    let available_projects: Vec<String> = self.project_manager.list_projects().into_iter().cloned().collect();
                    
                    // Try to rebuild index one more time for diagnostics
                    if let Ok(discovered) = self.project_manager.rebuild_index_from_filesystem() {
                        info!("Project save failed even after recovery. Discovered projects: {:?}", discovered);
                    }
                    
                    Err(format!("Failed to save project '{}' even after automatic recovery: {}. \
                                Available projects in index: {:?}. \
                                This may indicate a deeper filesystem or permissions issue.", 
                                project_name, e, available_projects).into())
                }
            }
        } else {
            Ok(())
        }
    }

    fn reload_current_project(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(project) = &self.current_project {
            let project_name = project.name.clone();
            self.current_project = Some(self.project_manager.load_project(&project_name)?);
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
    // Create layout with optional log pane
    let chunks = if app.show_logs {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),      // Header
                Constraint::Percentage(60), // Main content (reduced when logs shown)
                Constraint::Percentage(30), // Log pane
                Constraint::Length(3),      // Footer
            ])
            .split(frame.size())
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Footer
            ])
            .split(frame.size())
    };

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
        AppState::TicketFieldEdit(name, ticket_id, _) => &format!("✏️ Edit: Ticket {} in: {}", ticket_id, name),
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
        AppState::TicketFieldEdit(project_name, ticket_id, edit_state) => {
            render_ticket_field_edit(frame, chunks[1], app, project_name, ticket_id, edit_state)
        },
        _ => render_menu(frame, chunks[1], app),
    }

    // Log pane (if enabled)
    if app.show_logs {
        let log_entries = app.log_collector.get_entries();
        let log_lines: Vec<Line> = log_entries
            .iter()
            .map(|entry| {
                let level_color = match entry.level.as_str() {
                    "ERROR" => Color::Red,
                    "WARN" => Color::Yellow,
                    "INFO" => Color::Green,
                    "DEBUG" => Color::Blue,
                    _ => Color::White,
                };
                Line::from(vec![
                    Span::styled(&entry.timestamp, Style::default().fg(Color::Gray)),
                    Span::raw(" "),
                    Span::styled(&entry.level, Style::default().fg(level_color)),
                    Span::raw(" "),
                    Span::raw(&entry.message),
                ])
            })
            .collect();

        let logs = Paragraph::new(log_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("📋 Logs (L to toggle)"))
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });
        frame.render_widget(logs, chunks[2]);
    }

    // Footer with instructions
    let instructions = match &app.state {
        AppState::Input(_) => "Type to enter text | Enter: Submit | Esc: Cancel | Backspace: Delete",
        AppState::TicketDetailsView(_, _) => "↑↓: Navigate | 1-9: Execute action | Enter: Default action | /: Search | L: Logs | Esc: Back",
        AppState::TicketSearch(_, _, _) => "Type to search | Enter: Next match | Esc: Exit search",
        AppState::TicketFieldEdit(_, _, _) => "Type to edit text | Enter: Save | Esc: Cancel",
        _ => "↑↓: Navigate | Enter: Select | L: Toggle Logs | Esc/Q: Back/Quit",
    };
    
    let footer = Paragraph::new(instructions)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Yellow));
    
    let footer_index = if app.show_logs { 3 } else { 2 };
    frame.render_widget(footer, chunks[footer_index]);
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
    // Create animated progress indicator
    let progress_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let progress_index = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() / 100) % progress_chars.len() as u128;
    let progress_char = progress_chars[progress_index as usize];
    
    // Enhanced message formatting based on content
    let formatted_message = if message.contains("AI") || message.contains("Creating ticket") {
        format!("{} {} (This may take 10-30 seconds...)", progress_char, message)
    } else if message.contains("Saving") {
        format!("💾 {}", message)
    } else if message.contains("✅") {
        format!("{}", message) // Success messages don't need spinner
    } else {
        format!("{} {}", progress_char, message)
    };
    
    let loading = Paragraph::new(formatted_message)
        .block(Block::default().borders(Borders::ALL).title("Processing"))
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
        #[allow(unused_assignments)]
        {
            field_index += 1;
        }
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

fn render_ticket_field_edit(frame: &mut Frame, area: Rect, app: &App, _project_name: &str, _ticket_id: &TicketId, edit_state: &FieldEditState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Field info header
            Constraint::Length(5),  // Edit input area
            Constraint::Min(0),     // Current content preview
        ])
        .split(area);

    // Field info header
    let field_name = app.get_field_display_name(&edit_state.field);
    let modification_indicator = if edit_state.is_modified { " (MODIFIED)" } else { "" };
    let header_text = format!("Editing: {}{}", field_name, modification_indicator);
    
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title("Field Edit"))
        .style(Style::default().fg(if edit_state.is_modified { Color::Yellow } else { Color::Cyan }));
    frame.render_widget(header, chunks[0]);

    // Edit input area
    match &edit_state.edit_type {
        EditType::TextEdit => {
            let input_text = &edit_state.current_value;
            let input = Paragraph::new(input_text.as_str())
                .block(Block::default().borders(Borders::ALL).title("Current Value"))
                .style(Style::default().fg(Color::White))
                .wrap(Wrap { trim: true });
            frame.render_widget(input, chunks[1]);
        },
        EditType::StatusSelect | EditType::PrioritySelect | EditType::ComplexitySelect => {
            // For now, show the current value - dropdown selection can be implemented later
            let selection_type = match edit_state.edit_type {
                EditType::StatusSelect => "Status",
                EditType::PrioritySelect => "Priority", 
                EditType::ComplexitySelect => "Complexity",
                _ => "Selection"
            };
            let input = Paragraph::new(format!("Current {}: {}", selection_type, edit_state.current_value))
                .block(Block::default().borders(Borders::ALL).title("Selection Mode"))
                .style(Style::default().fg(Color::Green));
            frame.render_widget(input, chunks[1]);
        }
    }

    // Current content preview (show original value for comparison)
    if edit_state.is_modified {
        let preview_text = format!("Original value:\n{}", edit_state.original_value);
        let preview = Paragraph::new(preview_text)
            .block(Block::default().borders(Borders::ALL).title("Original Value"))
            .style(Style::default().fg(Color::Gray))
            .wrap(Wrap { trim: true });
        frame.render_widget(preview, chunks[2]);
    } else {
        // Show field context or help text
        let help_text = match &edit_state.edit_type {
            EditType::TextEdit => "Type to edit the text. Changes will be highlighted.",
            EditType::StatusSelect => "Use arrow keys to select status (Coming soon)",
            EditType::PrioritySelect => "Use arrow keys to select priority (Coming soon)",
            EditType::ComplexitySelect => "Use arrow keys to select complexity (Coming soon)",
        };
        let help = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .style(Style::default().fg(Color::Gray))
            .wrap(Wrap { trim: true });
        frame.render_widget(help, chunks[2]);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create log collector for TUI
    let log_collector = LogCollector::new(100);
    
    // Initialize custom TUI logging (no stdout/stderr output)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "control_flow=info,client_implementations=info".into()),
        )
        .with(TuiLayer::new(log_collector.clone()))
        .init();

    info!("Starting Control Flow TUI");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app with shared log collector
    let mut app = App::new_with_log_collector(log_collector)?;

    // Main loop
    let mut last_success_time: Option<std::time::Instant> = None;
    
    loop {
        terminal.draw(|f| render(f, &app))?;

        if app.should_quit {
            break;
        }

        // Check for auto-transition from success loading states
        if let AppState::Loading(ref msg) = app.state {
            if msg.contains("✅") {
                if last_success_time.is_none() {
                    last_success_time = Some(std::time::Instant::now());
                } else if last_success_time.unwrap().elapsed().as_secs() >= 2 {
                    // Auto-return after 2 seconds of success message
                    app.return_to_context();
                    last_success_time = None;
                }
            } else {
                last_success_time = None;
            }
        } else {
            last_success_time = None;
        }

        // Check for background task completions (non-blocking)
        while let Ok(result) = app.task_receiver.try_recv() {
            app.handle_background_task_completion(result)?;
        }

        // Non-blocking event handling with timeout
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code)?;
                    last_success_time = None; // Reset on user input
                }
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

// Helper functions for parsing enum values from strings
fn parse_status(status_str: &str) -> Result<control_flow::ticket::TicketStatus, Box<dyn std::error::Error>> {
    use control_flow::ticket::TicketStatus;
    match status_str.to_uppercase().as_str() {
        "AWAITING_REFINEMENT" | "AWAITING REFINEMENT" | "REFINEMENT" => Ok(TicketStatus::AwaitingRefinement),
        "IN_PROGRESS" | "IN PROGRESS" | "PROGRESS" => Ok(TicketStatus::InProgress),
        "UNDER_REVIEW" | "UNDER REVIEW" | "REVIEW" => Ok(TicketStatus::UnderReview),
        "COMPLETE" | "COMPLETED" | "DONE" => Ok(TicketStatus::Complete),
        "BLOCKED" | "BLOCK" => Ok(TicketStatus::Blocked),
        _ => Err(format!("Invalid status: {}", status_str).into()),
    }
}

fn parse_priority(priority_str: &str) -> Result<control_flow::ticket::Priority, Box<dyn std::error::Error>> {
    use control_flow::ticket::Priority;
    match priority_str.to_uppercase().as_str() {
        "LOW" => Ok(Priority::Low),
        "MEDIUM" | "MED" => Ok(Priority::Medium),
        "HIGH" => Ok(Priority::High),
        "CRITICAL" | "CRIT" => Ok(Priority::Critical),
        _ => Err(format!("Invalid priority: {}", priority_str).into()),
    }
}

fn parse_complexity(complexity_str: &str) -> Result<control_flow::ticket::Complexity, Box<dyn std::error::Error>> {
    use control_flow::ticket::Complexity;
    match complexity_str.to_uppercase().as_str() {
        "LOW" => Ok(Complexity::Low),
        "MEDIUM" | "MED" => Ok(Complexity::Medium),
        "MEDIUM_HIGH" | "MEDIUM HIGH" | "MED HIGH" => Ok(Complexity::MediumHigh),
        "HIGH" => Ok(Complexity::High),
        "VERY_HIGH" | "VERY HIGH" | "VERY" => Ok(Complexity::VeryHigh),
        _ => Err(format!("Invalid complexity: {}", complexity_str).into()),
    }
}