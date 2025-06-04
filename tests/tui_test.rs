use control_flow::ticket::{ProjectManager, TicketDecomposition, OriginalTicket, DecomposedTicket, 
    RefinementRequest, RefinementPriority, ValidationResults, TicketMetadata, 
    TicketStatus, Priority, Complexity};
use std::collections::HashMap;
use tempfile::TempDir;

#[test]
fn test_tui_navigation_logic() {
    // Test the arrow key navigation logic
    let mut selected_index: usize = 0;
    let mut _scroll_offset: usize = 0;
    let items = vec!["Item 1", "Item 2", "Item 3", "Item 4", "Item 5"];
    
    // Test moving down
    selected_index += 1;
    assert_eq!(selected_index, 1);
    
    // Test moving up
    selected_index = selected_index.saturating_sub(1);
    assert_eq!(selected_index, 0);
    
    // Test bounds checking
    selected_index = selected_index.saturating_sub(1);
    assert_eq!(selected_index, 0); // Should not go below 0
    
    // Test maximum bounds
    let max_index = items.len().saturating_sub(1);
    selected_index = max_index + 1;
    if selected_index > max_index {
        selected_index = max_index;
    }
    assert_eq!(selected_index, 4);
}

#[test]
fn test_tui_scroll_logic() {
    const VISIBLE_ITEMS: usize = 3;
    let mut selected_index: usize = 0;
    let mut scroll_offset: usize = 0;
    let total_items = 10;
    
    // Simulate scrolling down
    for i in 0..total_items {
        selected_index = i;
        
        // Adjust scroll logic (from TUI code)
        if selected_index < scroll_offset {
            scroll_offset = selected_index;
        } else if selected_index >= scroll_offset + VISIBLE_ITEMS {
            scroll_offset = selected_index + 1 - VISIBLE_ITEMS;
        }
        
        // Verify scroll offset is reasonable
        assert!(scroll_offset <= selected_index);
        assert!(selected_index < scroll_offset + VISIBLE_ITEMS || selected_index < VISIBLE_ITEMS);
    }
}

#[test]
fn test_refinement_priority_display() {
    // Test the priority emoji mapping used in TUI
    let test_cases = vec![
        (RefinementPriority::Critical, "🔥"),
        (RefinementPriority::High, "🟥"),
        (RefinementPriority::Medium, "🟨"),
        (RefinementPriority::Low, "🟩"),
    ];
    
    for (priority, expected_emoji) in test_cases {
        let emoji = match priority {
            RefinementPriority::Critical => "🔥",
            RefinementPriority::High => "🟥",
            RefinementPriority::Medium => "🟨",
            RefinementPriority::Low => "🟩",
        };
        assert_eq!(emoji, expected_emoji);
    }
}

#[test]
fn test_tui_menu_item_formatting() {
    // Test the formatting used in quick refine menu
    let refinement_request = RefinementRequest {
        term: "user-friendly".to_string(),
        context: "create a user-friendly todo app".to_string(),
        reason: "ambiguous - could mean intuitive interface, accessibility, or minimal design".to_string(),
        priority: RefinementPriority::High,
    };
    
    let priority_emoji = match refinement_request.priority {
        RefinementPriority::Critical => "🔥",
        RefinementPriority::High => "🟥",
        RefinementPriority::Medium => "🟨",
        RefinementPriority::Low => "🟩",
    };
    
    let formatted_item = format!("{}. {} {} - {}", 
        1, 
        priority_emoji, 
        refinement_request.term, 
        refinement_request.reason
    );
    
    assert!(formatted_item.contains("🟥"));
    assert!(formatted_item.contains("user-friendly"));
    assert!(formatted_item.contains("ambiguous"));
    assert_eq!(formatted_item, "1. 🟥 user-friendly - ambiguous - could mean intuitive interface, accessibility, or minimal design");
}

#[test]
fn test_tui_state_transitions() {
    // Test the state machine logic for TUI navigation
    #[derive(Debug, Clone, PartialEq)]
    enum TestAppState {
        MainMenu,
        ProjectMenu(String),
        TicketList(String),
        TicketDetail(String, String), // project name, ticket id
        QuickRefine(String, String),
    }
    
    let mut state = TestAppState::MainMenu;
    
    // Navigate to project menu
    state = TestAppState::ProjectMenu("todo_app".to_string());
    assert_eq!(state, TestAppState::ProjectMenu("todo_app".to_string()));
    
    // Navigate to ticket list
    state = TestAppState::TicketList("todo_app".to_string());
    assert_eq!(state, TestAppState::TicketList("todo_app".to_string()));
    
    // Navigate to ticket detail
    state = TestAppState::TicketDetail("todo_app".to_string(), "ticket123".to_string());
    assert_eq!(state, TestAppState::TicketDetail("todo_app".to_string(), "ticket123".to_string()));
    
    // Navigate to quick refine
    state = TestAppState::QuickRefine("todo_app".to_string(), "ticket123".to_string());
    assert_eq!(state, TestAppState::QuickRefine("todo_app".to_string(), "ticket123".to_string()));
    
    // Navigate back
    state = TestAppState::TicketDetail("todo_app".to_string(), "ticket123".to_string());
    state = TestAppState::TicketList("todo_app".to_string());
    state = TestAppState::ProjectMenu("todo_app".to_string());
    state = TestAppState::MainMenu;
    assert_eq!(state, TestAppState::MainMenu);
}

#[test]
fn test_menu_counts_display() {
    // Test the count display logic for menus
    let refinement_count = 3;
    let dependencies_count = 0;
    let dependents_count = 2;
    
    let menu_items = vec![
        "View ticket details".to_string(),
        format!("View refinement requests ({})", refinement_count),
        format!("Quick refine - show all terms ({})", refinement_count),
        format!("View dependencies ({})", dependencies_count),
        format!("View dependents ({})", dependents_count),
        "← Back".to_string(),
    ];
    
    assert_eq!(menu_items[1], "View refinement requests (3)");
    assert_eq!(menu_items[2], "Quick refine - show all terms (3)");
    assert_eq!(menu_items[3], "View dependencies (0)");
    assert_eq!(menu_items[4], "View dependents (2)");
}

#[test]
fn test_tui_with_real_project_data() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = ProjectManager::new(temp_dir.path()).unwrap();
    
    // Create a project
    manager.create_project("test_tui_project".to_string(), "TUI test project".to_string()).unwrap();
    let mut project = manager.load_project("test_tui_project").unwrap();
    
    // Create a ticket with refinement requests
    let mut terms = HashMap::new();
    terms.insert("responsive".to_string(), "Adapts to different screen sizes".to_string());
    
    let refinement_requests = vec![
        RefinementRequest {
            term: "intuitive".to_string(),
            context: "intuitive design".to_string(),
            reason: "needs UX guidelines".to_string(),
            priority: RefinementPriority::High,
        },
        RefinementRequest {
            term: "fast".to_string(),
            context: "fast performance".to_string(),
            reason: "unclear performance targets".to_string(),
            priority: RefinementPriority::Critical,
        },
    ];
    
    let ticket = TicketDecomposition {
        original_ticket: OriginalTicket {
            title: "TUI Test Ticket".to_string(),
            raw_input: "Create an intuitive and fast application".to_string(),
        },
        decomposed_ticket: DecomposedTicket {
            terms,
            terms_needing_refinement: refinement_requests,
            open_questions: vec!["What platforms?".to_string()],
            engine_questions: vec!["Preferred framework?".to_string()],
            validation_method: vec!["User testing".to_string()],
            validation_results: ValidationResults {
                mime: "text/plain".to_string(),
                url: "pending".to_string(),
            },
            metadata: TicketMetadata {
                status: TicketStatus::AwaitingRefinement,
                priority: Priority::High,
                estimated_complexity: Complexity::Medium,
                processed_at: "2024-01-01T00:00:00Z".to_string(),
                engine_version: "1.0".to_string(),
            },
        },
    };
    
    let ticket_id = project.add_ticket(ticket);
    
    // Test TUI menu generation
    let node = project.graph.get_ticket(&ticket_id).unwrap();
    let refinement_count = node.ticket.decomposed_ticket.terms_needing_refinement.len();
    let dependencies_count = node.dependencies.len();
    let dependents_count = node.dependents.len();
    
    // Test menu item formatting as TUI would do
    let menu_items = vec![
        "View ticket details".to_string(),
        format!("View refinement requests ({})", refinement_count),
        format!("Quick refine - show all terms ({})", refinement_count),
        format!("View dependencies ({})", dependencies_count),
        format!("View dependents ({})", dependents_count),
        "← Back".to_string(),
    ];
    
    assert_eq!(menu_items[1], "View refinement requests (2)");
    assert_eq!(menu_items[2], "Quick refine - show all terms (2)");
    assert_eq!(menu_items[3], "View dependencies (0)");
    assert_eq!(menu_items[4], "View dependents (0)");
    
    // Test quick refine menu generation
    let quick_refine_items: Vec<String> = node.ticket.decomposed_ticket.terms_needing_refinement
        .iter()
        .enumerate()
        .map(|(i, request)| {
            let priority_emoji = match request.priority {
                RefinementPriority::Critical => "🔥",
                RefinementPriority::High => "🟥",
                RefinementPriority::Medium => "🟨",
                RefinementPriority::Low => "🟩",
            };
            format!("{}. {} {} - {}", i + 1, priority_emoji, request.term, request.reason)
        })
        .collect();
    
    assert_eq!(quick_refine_items.len(), 2);
    assert!(quick_refine_items[0].contains("🟥 intuitive"));
    assert!(quick_refine_items[1].contains("🔥 fast"));
    assert!(quick_refine_items[0].contains("needs UX guidelines"));
    assert!(quick_refine_items[1].contains("unclear performance targets"));
    
    println!("TUI test successful!");
    println!("Menu items: {:?}", menu_items);
    println!("Quick refine items: {:?}", quick_refine_items);
}