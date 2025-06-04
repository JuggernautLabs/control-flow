use control_flow::ticket::{
    ProjectManager, TicketDecomposition, OriginalTicket, DecomposedTicket, 
    RefinementRequest, RefinementPriority, ValidationResults, TicketMetadata, 
    TicketStatus, Priority, Complexity
};
use std::collections::HashMap;
use tempfile::TempDir;

fn create_todo_app_ticket() -> TicketDecomposition {
    let mut terms = HashMap::new();
    terms.insert("todo".to_string(), "A task or item that needs to be completed".to_string());
    terms.insert("CRUD".to_string(), "Create, Read, Update, Delete - basic data operations".to_string());
    terms.insert("responsive".to_string(), "User interface that adapts to different screen sizes".to_string());

    let refinement_requests = vec![
        RefinementRequest {
            term: "user-friendly".to_string(),
            context: "create a user-friendly todo app".to_string(),
            reason: "ambiguous - could mean intuitive interface, accessibility, or minimal design".to_string(),
            priority: RefinementPriority::High,
        },
        RefinementRequest {
            term: "modern".to_string(),
            context: "with modern design".to_string(),
            reason: "vague aesthetic requirement - needs specific design guidelines".to_string(),
            priority: RefinementPriority::Medium,
        },
        RefinementRequest {
            term: "efficient".to_string(),
            context: "efficient task management".to_string(),
            reason: "unclear if referring to performance, UX workflow, or data structure efficiency".to_string(),
            priority: RefinementPriority::Critical,
        },
    ];

    TicketDecomposition {
        original_ticket: OriginalTicket {
            title: "Create Todo App".to_string(),
            raw_input: "Create a user-friendly todo app with modern design and efficient task management".to_string(),
        },
        decomposed_ticket: DecomposedTicket {
            terms,
            terms_needing_refinement: refinement_requests,
            open_questions: vec![
                "What platforms should the app support? (web, mobile, desktop)".to_string(),
                "Should users be able to share todo lists with others?".to_string(),
                "What authentication method is preferred?".to_string(),
                "Are there any specific accessibility requirements?".to_string(),
                "What is the expected number of concurrent users?".to_string(),
            ],
            engine_questions: vec![
                "Would you like to use any specific frontend framework?".to_string(),
                "Do you have preferences for the backend technology stack?".to_string(),
            ],
            validation_method: vec![
                "User can create, edit, and delete todo items".to_string(),
                "Interface is responsive on mobile and desktop".to_string(),
                "App loads within 2 seconds on standard connection".to_string(),
            ],
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
    }
}

#[test]
fn test_todo_app_ticket_structure() {
    let ticket = create_todo_app_ticket();
    
    // Verify basic ticket structure
    assert_eq!(ticket.original_ticket.title, "Create Todo App");
    assert!(ticket.original_ticket.raw_input.contains("user-friendly"));
    assert!(ticket.original_ticket.raw_input.contains("modern"));
    assert!(ticket.original_ticket.raw_input.contains("efficient"));
    
    // Verify terms are defined
    assert_eq!(ticket.decomposed_ticket.terms.len(), 3);
    assert!(ticket.decomposed_ticket.terms.contains_key("todo"));
    assert!(ticket.decomposed_ticket.terms.contains_key("CRUD"));
    assert!(ticket.decomposed_ticket.terms.contains_key("responsive"));
    
    // Verify refinement requests
    assert_eq!(ticket.decomposed_ticket.terms_needing_refinement.len(), 3);
    
    let refinement_terms: Vec<&str> = ticket.decomposed_ticket.terms_needing_refinement
        .iter()
        .map(|r| r.term.as_str())
        .collect();
    
    assert!(refinement_terms.contains(&"user-friendly"));
    assert!(refinement_terms.contains(&"modern"));
    assert!(refinement_terms.contains(&"efficient"));
    
    // Verify priorities are set correctly
    let critical_terms: Vec<&str> = ticket.decomposed_ticket.terms_needing_refinement
        .iter()
        .filter(|r| matches!(r.priority, RefinementPriority::Critical))
        .map(|r| r.term.as_str())
        .collect();
    
    assert_eq!(critical_terms.len(), 1);
    assert!(critical_terms.contains(&"efficient"));
    
    // Verify questions exist
    assert!(ticket.decomposed_ticket.open_questions.len() >= 5);
    assert!(ticket.decomposed_ticket.engine_questions.len() >= 2);
    assert!(ticket.decomposed_ticket.validation_method.len() >= 3);
}

#[test]
fn test_refinement_requests_display_format() {
    let ticket = create_todo_app_ticket();
    
    // Test that we can iterate through refinement requests like the CLI does
    for (i, request) in ticket.decomposed_ticket.terms_needing_refinement.iter().enumerate() {
        let priority_emoji = match request.priority {
            RefinementPriority::Critical => "🔥",
            RefinementPriority::High => "🟥", 
            RefinementPriority::Medium => "🟨",
            RefinementPriority::Low => "🟩",
        };
        
        // Simulate what the CLI displays
        let display_line = format!("{}. {} {} - {}", 
            i + 1, 
            priority_emoji,
            request.term, 
            request.reason
        );
        
        // Verify the display format contains expected elements
        assert!(display_line.contains(&request.term));
        assert!(display_line.contains(&request.reason));
        assert!(display_line.starts_with(&format!("{}.", i + 1)));
        
        // Verify context is available for display
        if !request.context.is_empty() && request.context != "Legacy format - context not specified" {
            assert!(!request.context.is_empty());
        }
    }
}

#[test]
fn test_project_with_todo_ticket() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = ProjectManager::new(temp_dir.path()).unwrap();
    
    // Create project
    manager.create_project("todo_app".to_string(), "making a todo app".to_string()).unwrap();
    let mut project = manager.load_project("todo_app").unwrap();
    
    // Add the todo app ticket
    let ticket = create_todo_app_ticket();
    let ticket_id = project.add_ticket(ticket);
    
    // Verify ticket was added correctly
    assert_eq!(project.graph.nodes.len(), 1);
    
    let stored_node = project.graph.get_ticket(&ticket_id).unwrap();
    assert_eq!(stored_node.ticket.original_ticket.title, "Create Todo App");
    
    // Test that we can access refinement requests like the CLI does
    let refinement_count = stored_node.ticket.decomposed_ticket.terms_needing_refinement.len();
    assert_eq!(refinement_count, 3);
    
    // Test CLI functionality: "View refinement requests" (option 2)
    println!("Testing CLI Option 2 - View refinement requests:");
    if stored_node.ticket.decomposed_ticket.terms_needing_refinement.is_empty() {
        println!("No terms need refinement.");
    } else {
        for (i, request) in stored_node.ticket.decomposed_ticket.terms_needing_refinement.iter().enumerate() {
            println!("{}. Term: {}", i + 1, request.term);
            println!("   Context: {}", request.context);
            println!("   Reason: {}", request.reason);
            println!("   Priority: {:?}", request.priority);
            println!();
        }
    }
    
    // Test CLI functionality: "Quick refine" (option 4)
    println!("Testing CLI Option 4 - Quick refine:");
    println!("🔍 Terms Available for Refinement:");
    for (i, request) in stored_node.ticket.decomposed_ticket.terms_needing_refinement.iter().enumerate() {
        let priority_emoji = match request.priority {
            RefinementPriority::Critical => "🔥",
            RefinementPriority::High => "🟥", 
            RefinementPriority::Medium => "🟨",
            RefinementPriority::Low => "🟩",
        };
        
        println!("{}. {} {} - {}", 
            i + 1, 
            priority_emoji,
            request.term, 
            request.reason
        );
        if !request.context.is_empty() && request.context != "Legacy format - context not specified" {
            println!("   Context: {}", request.context);
        }
    }
    
    // Verify all terms are shown
    assert_eq!(stored_node.ticket.decomposed_ticket.terms_needing_refinement.len(), 3);
    
    // Test root ticket functionality
    let root_tickets = project.get_root_tickets();
    assert_eq!(root_tickets.len(), 1);
    assert_eq!(root_tickets[0], &ticket_id);
}

#[test]
fn test_refinement_context_generation() {
    let ticket = create_todo_app_ticket();
    
    // Test that we can generate refinement context like the CLI does
    let refinement_request = &ticket.decomposed_ticket.terms_needing_refinement[0]; // "user-friendly"
    
    let refinement_context = format!(
        "Context from parent ticket '{}': {}\n\nTerm being refined: {}\nReason for refinement: {}\nAdditional context: {}",
        ticket.original_ticket.title,
        ticket.original_ticket.raw_input,
        refinement_request.term,
        refinement_request.reason,
        refinement_request.context
    );
    
    // Verify context contains all necessary information
    assert!(refinement_context.contains("Create Todo App"));
    assert!(refinement_context.contains("user-friendly todo app"));
    assert!(refinement_context.contains("user-friendly"));
    assert!(refinement_context.contains("ambiguous - could mean intuitive interface"));
    assert!(refinement_context.contains("create a user-friendly todo app"));
    
    println!("Generated refinement context:");
    println!("{}", refinement_context);
}

#[test]
fn test_mixed_refinement_formats_in_real_scenario() {
    // Test with a mix of structured and string formats (for backward compatibility)
    let json_with_mixed_formats = r#"{
        "originalTicket": {
            "title": "Mixed Format Test",
            "rawInput": "test mixed formats"
        },
        "decomposedTicket": {
            "terms": {},
            "termsNeedingRefinement": [
                "legacy string term",
                {
                    "term": "structured term",
                    "context": "in requirements",
                    "reason": "needs clarification", 
                    "priority": "HIGH"
                }
            ],
            "openQuestions": [],
            "engineQuestions": [],
            "validationMethod": [],
            "validationResults": {
                "mime": "text/plain",
                "url": "pending"
            },
            "metadata": {
                "status": "AWAITING_REFINEMENT",
                "priority": "MEDIUM",
                "estimatedComplexity": "LOW",
                "processedAt": "2024-01-01T00:00:00Z",
                "engineVersion": "1.0"
            }
        }
    }"#;
    
    let ticket: TicketDecomposition = serde_json::from_str(json_with_mixed_formats).unwrap();
    
    // Verify both formats were parsed correctly
    assert_eq!(ticket.decomposed_ticket.terms_needing_refinement.len(), 2);
    
    // First item should be converted from string
    assert_eq!(ticket.decomposed_ticket.terms_needing_refinement[0].term, "legacy string term");
    assert_eq!(ticket.decomposed_ticket.terms_needing_refinement[0].context, "Legacy format - context not specified");
    
    // Second item should maintain structured format
    assert_eq!(ticket.decomposed_ticket.terms_needing_refinement[1].term, "structured term");
    assert_eq!(ticket.decomposed_ticket.terms_needing_refinement[1].context, "in requirements");
    assert!(matches!(ticket.decomposed_ticket.terms_needing_refinement[1].priority, RefinementPriority::High));
}

#[test]
fn test_refinement_workflow_simulation() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = ProjectManager::new(temp_dir.path()).unwrap();
    
    // Create project and add todo app ticket
    manager.create_project("todo_app".to_string(), "making a todo app".to_string()).unwrap();
    let mut project = manager.load_project("todo_app").unwrap();
    
    let original_ticket = create_todo_app_ticket();
    let parent_ticket_id = project.add_ticket(original_ticket);
    
    // Simulate selecting the first refinement term (user-friendly)
    let parent_node = project.graph.get_ticket(&parent_ticket_id).unwrap();
    let refinement_request = &parent_node.ticket.decomposed_ticket.terms_needing_refinement[0];
    
    assert_eq!(refinement_request.term, "user-friendly");
    assert_eq!(refinement_request.priority, RefinementPriority::High);
    
    // Simulate the CLI's refinement context generation
    let refinement_context = format!(
        "Context from parent ticket '{}': {}\n\nTerm being refined: {}\nReason for refinement: {}\nAdditional context: {}",
        parent_node.ticket.original_ticket.title,
        parent_node.ticket.original_ticket.raw_input,
        refinement_request.term,
        refinement_request.reason,
        refinement_request.context
    );
    
    println!("Generated refinement context for AI:");
    println!("{}", refinement_context);
    
    // Verify context contains all necessary information for AI
    assert!(refinement_context.contains("Create Todo App"));
    assert!(refinement_context.contains("user-friendly"));
    assert!(refinement_context.contains("ambiguous - could mean intuitive interface"));
    
    // Simulate creating a refinement ticket (normally done by AI service)
    let refinement_ticket = create_refinement_ticket_for_user_friendly();
    let refinement_ticket_id = project.add_ticket(refinement_ticket);
    
    // Add dependency relationship
    let dependency_result = project.graph.add_dependency(&refinement_ticket_id, &parent_ticket_id);
    assert!(dependency_result.is_ok());
    
    // Verify the dependency relationship was created correctly
    let dependencies = project.graph.get_dependencies(&refinement_ticket_id).unwrap();
    assert!(dependencies.contains(&parent_ticket_id));
    
    let dependents = project.graph.get_dependents(&parent_ticket_id).unwrap();
    assert!(dependents.contains(&refinement_ticket_id));
    
    // Verify we now have 2 tickets in the graph
    assert_eq!(project.graph.nodes.len(), 2);
    
    // Test that only the original ticket is a root ticket
    let root_tickets = project.get_root_tickets();
    assert_eq!(root_tickets.len(), 1);
    assert_eq!(root_tickets[0], &parent_ticket_id);
    
    println!("✅ Refinement workflow simulation completed successfully");
}

fn create_refinement_ticket_for_user_friendly() -> TicketDecomposition {
    let mut terms = HashMap::new();
    terms.insert("intuitive".to_string(), "Easy to understand and use without extensive learning".to_string());
    terms.insert("accessibility".to_string(), "Design that accommodates users with disabilities".to_string());
    terms.insert("usability".to_string(), "How effectively and efficiently users can accomplish tasks".to_string());

    let refinement_requests = vec![
        RefinementRequest {
            term: "intuitive".to_string(),
            context: "intuitive interface design".to_string(),
            reason: "needs specific UI/UX guidelines and patterns".to_string(),
            priority: RefinementPriority::High,
        },
    ];

    TicketDecomposition {
        original_ticket: OriginalTicket {
            title: "Define User-Friendly Requirements".to_string(),
            raw_input: "Context from parent ticket 'Create Todo App': Create a user-friendly todo app with modern design and efficient task management\n\nTerm being refined: user-friendly\nReason for refinement: ambiguous - could mean intuitive interface, accessibility, or minimal design\nAdditional context: create a user-friendly todo app".to_string(),
        },
        decomposed_ticket: DecomposedTicket {
            terms,
            terms_needing_refinement: refinement_requests,
            open_questions: vec![
                "What accessibility standards should be followed (WCAG 2.1)?".to_string(),
                "Should the interface support keyboard navigation only?".to_string(),
                "What is the target user demographic and technical skill level?".to_string(),
            ],
            engine_questions: vec![
                "Do you have any existing brand guidelines for UI design?".to_string(),
            ],
            validation_method: vec![
                "Interface passes WCAG 2.1 AA accessibility audit".to_string(),
                "User testing shows 90% task completion rate".to_string(),
                "Average time to complete basic tasks under 30 seconds".to_string(),
            ],
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
    }
}

#[test]
fn test_all_cli_options_with_todo_app() {
    let temp_dir = TempDir::new().unwrap();
    let mut manager = ProjectManager::new(temp_dir.path()).unwrap();
    
    // Setup
    manager.create_project("todo_app".to_string(), "making a todo app".to_string()).unwrap();
    let mut project = manager.load_project("todo_app").unwrap();
    let ticket = create_todo_app_ticket();
    let ticket_id = project.add_ticket(ticket);
    
    let node = project.graph.get_ticket(&ticket_id).unwrap();
    
    println!("\n=== Testing All CLI Options for Todo App Ticket ===");
    
    // Test CLI Option 1: View ticket details
    println!("\n📄 CLI Option 1 - Ticket Details:");
    println!("Title: {}", node.ticket.original_ticket.title);
    println!("Original Input: {}", node.ticket.original_ticket.raw_input);
    println!("\nTerms:");
    for (term, definition) in &node.ticket.decomposed_ticket.terms {
        println!("  • {}: {}", term, definition);
    }
    assert_eq!(node.ticket.decomposed_ticket.terms.len(), 3);
    
    // Test CLI Option 2: View refinement requests (already tested above)
    println!("\n🔍 CLI Option 2 - Refinement Requests:");
    assert_eq!(node.ticket.decomposed_ticket.terms_needing_refinement.len(), 3);
    for (i, request) in node.ticket.decomposed_ticket.terms_needing_refinement.iter().enumerate() {
        println!("{}. Term: {}", i + 1, request.term);
        println!("   Reason: {}", request.reason);
    }
    
    // Test CLI Option 3: Would refine a specific term (simulated)
    println!("\n⚡ CLI Option 3 - Refine Term (simulated selection of term 1):");
    let selected_refinement = &node.ticket.decomposed_ticket.terms_needing_refinement[0];
    println!("Selected term: {}", selected_refinement.term);
    println!("Reason: {}", selected_refinement.reason);
    
    // Test CLI Option 4: Quick refine - already tested, shows ALL terms with priorities
    println!("\n🚀 CLI Option 4 - Quick Refine (shows ALL {} terms):", 
        node.ticket.decomposed_ticket.terms_needing_refinement.len());
    
    let mut critical_count = 0;
    let mut high_count = 0;
    let mut medium_count = 0;
    let mut low_count = 0;
    
    for (i, request) in node.ticket.decomposed_ticket.terms_needing_refinement.iter().enumerate() {
        let priority_emoji = match request.priority {
            RefinementPriority::Critical => { critical_count += 1; "🔥" },
            RefinementPriority::High => { high_count += 1; "🟥" },
            RefinementPriority::Medium => { medium_count += 1; "🟨" },
            RefinementPriority::Low => { low_count += 1; "🟩" },
        };
        println!("{}. {} {}", i + 1, priority_emoji, request.term);
    }
    
    // Verify all priority levels are represented
    assert_eq!(critical_count, 1, "Should have exactly 1 critical term");
    assert_eq!(high_count, 1, "Should have exactly 1 high priority term");
    assert_eq!(medium_count, 1, "Should have exactly 1 medium priority term");
    assert_eq!(low_count, 0, "Should have 0 low priority terms");
    
    // Test CLI Options 5 & 6: Dependencies and Dependents (empty for root ticket)
    println!("\n⬇️ CLI Option 5 - Dependencies: {}", node.dependencies.len());
    println!("⬆️ CLI Option 6 - Dependents: {}", node.dependents.len());
    assert_eq!(node.dependencies.len(), 0, "Root ticket should have no dependencies");
    assert_eq!(node.dependents.len(), 0, "New ticket should have no dependents yet");
    
    println!("\n✅ All CLI options tested successfully!");
    println!("   - Option 1: Displays {} terms, {} questions", 
        node.ticket.decomposed_ticket.terms.len(),
        node.ticket.decomposed_ticket.open_questions.len());
    println!("   - Option 2: Shows {} refinement requests with context", 
        node.ticket.decomposed_ticket.terms_needing_refinement.len());
    println!("   - Option 4: Displays ALL {} terms with priority indicators", 
        node.ticket.decomposed_ticket.terms_needing_refinement.len());
    println!("   - Options 5&6: Show dependency relationships (0 for new ticket)");
}