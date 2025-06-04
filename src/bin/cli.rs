use control_flow::ticket::{ProjectManager, Project, TicketId, RefinementPriority};
use control_flow::ticket_service::TicketService;
use client_implementations::claude::ClaudeClient;
use client_implementations::client::RetryConfig;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "control_flow=info,client_implementations=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
    
    info!("Starting Control Flow CLI");
    
    let api_key = env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");
    
    let claude_client = ClaudeClient::new(api_key);
    let retry_config = RetryConfig::default();
    let ticket_service = TicketService::new(claude_client, retry_config);
    
    let workspace_dir = PathBuf::from("./control-flow-projects");
    let mut project_manager = match ProjectManager::load_index(&workspace_dir) {
        Ok(manager) => manager,
        Err(e) => {
            info!("Could not load existing project index: {}, creating new one", e);
            ProjectManager::new(&workspace_dir).map_err(|err| {
                format!("Failed to create project manager: {}", err)
            })?
        }
    };
    
    println!("🎫 Control Flow - Ticket Decomposition & Project Management");
    
    loop {
        match main_menu(&mut project_manager, &ticket_service).await {
            Ok(should_continue) => {
                if !should_continue {
                    break;
                }
            }
            Err(e) => {
                error!(error = %e, "Error in main menu");
                println!("❌ Error: {}", e);
            }
        }
    }
    
    println!("Goodbye!");
    Ok(())
}

async fn main_menu(
    project_manager: &mut ProjectManager,
    ticket_service: &TicketService<ClaudeClient>,
) -> Result<bool, Box<dyn std::error::Error>> {
    println!("\n📁 Project Menu:");
    println!("1. List projects");
    println!("2. Create new project");
    println!("3. Open project");
    println!("4. Delete project");
    println!("5. Exit");
    
    print!("\nSelect option: ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {},
        Err(e) => {
            println!("Error reading input: {}", e);
            return Ok(true);
        }
    }
    let choice = input.trim();
    
    if choice.is_empty() {
        return Ok(true);
    }
    
    match choice {
        "1" => list_projects(project_manager),
        "2" => create_project(project_manager).await?,
        "3" => open_project(project_manager, ticket_service).await?,
        "4" => delete_project(project_manager).await?,
        "5" => return Ok(false),
        _ => println!("Invalid option, please try again."),
    }
    
    Ok(true)
}

fn list_projects(project_manager: &ProjectManager) {
    let projects = project_manager.list_projects();
    if projects.is_empty() {
        println!("No projects found.");
    } else {
        println!("\n📋 Projects:");
        for (i, project_name) in projects.iter().enumerate() {
            println!("{}. {}", i + 1, project_name);
        }
    }
}

async fn create_project(project_manager: &mut ProjectManager) -> Result<(), Box<dyn std::error::Error>> {
    print!("Enter project name: ");
    io::stdout().flush()?;
    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    let name = name.trim().to_string();
    
    print!("Enter project description: ");
    io::stdout().flush()?;
    let mut description = String::new();
    io::stdin().read_line(&mut description)?;
    let description = description.trim().to_string();
    
    match project_manager.create_project(name.clone(), description) {
        Ok(_) => println!("✅ Project '{}' created successfully!", name),
        Err(e) => {
            println!("❌ Failed to create project '{}': {}", name, e);
            return Err(e);
        }
    }
    
    Ok(())
}

async fn open_project(
    project_manager: &mut ProjectManager,
    ticket_service: &TicketService<ClaudeClient>,
) -> Result<(), Box<dyn std::error::Error>> {
    list_projects(project_manager);
    
    print!("Enter project name to open: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let project_name = input.trim();
    
    let mut project = project_manager.load_project(project_name)?;
    println!("📂 Opened project: {}", project.name);
    println!("Description: {}", project.description);
    
    loop {
        match project_menu(&mut project, project_manager, ticket_service).await {
            Ok(should_continue) => {
                if !should_continue {
                    break;
                }
            }
            Err(e) => {
                error!(error = %e, "Error in project menu");
                println!("❌ Error: {}", e);
            }
        }
    }
    
    Ok(())
}

async fn delete_project(project_manager: &mut ProjectManager) -> Result<(), Box<dyn std::error::Error>> {
    list_projects(project_manager);
    
    print!("Enter project name to delete: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let project_name = input.trim();
    
    print!("Are you sure you want to delete '{}'? (y/N): ", project_name);
    io::stdout().flush()?;
    let mut confirmation = String::new();
    io::stdin().read_line(&mut confirmation)?;
    
    if confirmation.trim().to_lowercase() == "y" {
        project_manager.delete_project(project_name)?;
        println!("✅ Project '{}' deleted successfully!", project_name);
    } else {
        println!("Deletion cancelled.");
    }
    
    Ok(())
}

async fn project_menu(
    project: &mut Project,
    project_manager: &ProjectManager,
    ticket_service: &TicketService<ClaudeClient>,
) -> Result<bool, Box<dyn std::error::Error>> {
    println!("\n🎫 Project: {} - Ticket Menu:", project.name);
    println!("1. View root tickets");
    println!("2. Navigate tickets");
    println!("3. Create new ticket");
    println!("4. Save project");
    println!("5. Back to main menu");
    
    print!("\nSelect option: ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice = input.trim();
    
    match choice {
        "1" => view_root_tickets(project),
        "2" => navigate_tickets(project, ticket_service).await?,
        "3" => create_ticket(project, ticket_service).await?,
        "4" => {
            project_manager.save_project(project)?;
            println!("✅ Project saved successfully!");
        }
        "5" => return Ok(false),
        _ => println!("Invalid option, please try again."),
    }
    
    Ok(true)
}

fn view_root_tickets(project: &Project) {
    let root_tickets = project.get_root_tickets();
    if root_tickets.is_empty() {
        println!("No root tickets found. Create a new ticket to get started.");
    } else {
        println!("\n🌳 Root Tickets:");
        for (i, ticket_id) in root_tickets.iter().enumerate() {
            if let Some(node) = project.graph.get_ticket(ticket_id) {
                println!("{}. {} - {}", 
                    i + 1, 
                    ticket_id, 
                    node.ticket.original_ticket.title
                );
            }
        }
    }
}

async fn navigate_tickets(
    project: &mut Project,
    ticket_service: &TicketService<ClaudeClient>,
) -> Result<(), Box<dyn std::error::Error>> {
    let root_tickets = project.get_root_tickets();
    if root_tickets.is_empty() {
        println!("No tickets to navigate. Create a ticket first.");
        return Ok(());
    }
    
    println!("\n🧭 Select a ticket to navigate:");
    for (i, ticket_id) in root_tickets.iter().enumerate() {
        if let Some(node) = project.graph.get_ticket(ticket_id) {
            println!("{}. {} - {}", 
                i + 1, 
                ticket_id, 
                node.ticket.original_ticket.title
            );
        }
    }
    
    print!("Enter ticket number (or ticket ID): ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    
    let ticket_id = if let Ok(index) = input.parse::<usize>() {
        if index > 0 && index <= root_tickets.len() {
            root_tickets[index - 1].clone()
        } else {
            println!("Invalid ticket number.");
            return Ok(());
        }
    } else {
        // Try to parse as ticket ID
        println!("Ticket ID parsing not yet implemented - use numbers for now.");
        return Ok(());
    };
    
    ticket_detail_menu(project, &ticket_id, ticket_service).await
}

async fn ticket_detail_menu(
    project: &mut Project,
    ticket_id: &TicketId,
    ticket_service: &TicketService<ClaudeClient>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let node = project.graph.get_ticket(ticket_id)
            .ok_or("Ticket not found")?;
        
        println!("\n🎫 Ticket: {}", node.ticket.original_ticket.title);
        println!("ID: {}", ticket_id);
        println!("Status: {:?}", node.ticket.decomposed_ticket.metadata.status);
        println!("Priority: {:?}", node.ticket.decomposed_ticket.metadata.priority);
        
        // Show iteration summary
        let refinement_count = node.ticket.decomposed_ticket.terms_needing_refinement.len();
        let dependencies_count = node.dependencies.len();
        let dependents_count = node.dependents.len();
        
        if refinement_count > 0 {
            println!("🔍 {} terms need refinement", refinement_count);
        }
        if dependencies_count > 0 {
            println!("⬇️  {} dependencies", dependencies_count);
        }
        if dependents_count > 0 {
            println!("⬆️  {} dependents", dependents_count);
        }
        
        println!("\nOptions:");
        println!("1. View ticket details");
        println!("2. View refinement requests");
        println!("3. Refine a term (iterate)");
        println!("4. Quick refine - show all terms and select");
        println!("5. View dependencies");
        println!("6. View dependents");
        println!("7. Back");
        
        print!("\nSelect option: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();
        
        match choice {
            "1" => view_ticket_details(project, ticket_id)?,
            "2" => view_refinement_requests(project, ticket_id)?,
            "3" => refine_term(project, ticket_id, ticket_service).await?,
            "4" => quick_refine(project, ticket_id, ticket_service).await?,
            "5" => view_dependencies(project, ticket_id)?,
            "6" => view_dependents(project, ticket_id)?,
            "7" => break,
            _ => println!("Invalid option, please try again."),
        }
    }
    
    Ok(())
}

fn view_ticket_details(project: &Project, ticket_id: &TicketId) -> Result<(), Box<dyn std::error::Error>> {
    let node = project.graph.get_ticket(ticket_id)
        .ok_or("Ticket not found")?;
    
    println!("\n📄 Ticket Details:");
    println!("Title: {}", node.ticket.original_ticket.title);
    println!("Original Input: {}", node.ticket.original_ticket.raw_input);
    println!("\nTerms:");
    for (term, definition) in &node.ticket.decomposed_ticket.terms {
        println!("  • {}: {}", term, definition);
    }
    
    println!("\nOpen Questions:");
    for question in &node.ticket.decomposed_ticket.open_questions {
        println!("  • {}", question);
    }
    
    println!("\nEngine Questions:");
    for question in &node.ticket.decomposed_ticket.engine_questions {
        println!("  • {}", question);
    }
    
    println!("\nValidation Methods:");
    for method in &node.ticket.decomposed_ticket.validation_method {
        println!("  • {}", method);
    }
    
    Ok(())
}

fn view_refinement_requests(project: &Project, ticket_id: &TicketId) -> Result<(), Box<dyn std::error::Error>> {
    let node = project.graph.get_ticket(ticket_id)
        .ok_or("Ticket not found")?;
    
    println!("\n🔍 Refinement Requests:");
    if node.ticket.decomposed_ticket.terms_needing_refinement.is_empty() {
        println!("No terms need refinement.");
    } else {
        for (i, request) in node.ticket.decomposed_ticket.terms_needing_refinement.iter().enumerate() {
            println!("{}. Term: {}", i + 1, request.term);
            println!("   Context: {}", request.context);
            println!("   Reason: {}", request.reason);
            println!("   Priority: {:?}", request.priority);
            println!();
        }
    }
    
    Ok(())
}

async fn refine_term(
    project: &mut Project,
    ticket_id: &TicketId,
    ticket_service: &TicketService<ClaudeClient>,
) -> Result<(), Box<dyn std::error::Error>> {
    let node = project.graph.get_ticket(ticket_id)
        .ok_or("Ticket not found")?;
    
    if node.ticket.decomposed_ticket.terms_needing_refinement.is_empty() {
        println!("No terms available for refinement.");
        return Ok(());
    }
    
    println!("\n🔍 Select term to refine:");
    for (i, request) in node.ticket.decomposed_ticket.terms_needing_refinement.iter().enumerate() {
        println!("{}. {} - {}", i + 1, request.term, request.reason);
    }
    
    print!("Enter term number: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let index: usize = input.trim().parse()?;
    
    if index == 0 || index > node.ticket.decomposed_ticket.terms_needing_refinement.len() {
        println!("Invalid term number.");
        return Ok(());
    }
    
    let refinement_request = &node.ticket.decomposed_ticket.terms_needing_refinement[index - 1];
    
    let refinement_context = format!(
        "Context from parent ticket '{}': {}\n\nTerm being refined: {}\nReason for refinement: {}\nAdditional context: {}",
        node.ticket.original_ticket.title,
        node.ticket.original_ticket.raw_input,
        refinement_request.term,
        refinement_request.reason,
        refinement_request.context
    );
    
    println!("🔄 Creating refinement ticket for term: {}", refinement_request.term);
    
    match ticket_service.decompose_ticket(refinement_context).await {
        Ok(refined_ticket) => {
            let new_ticket_id = project.add_ticket(refined_ticket);
            project.graph.add_dependency(&new_ticket_id, ticket_id)?;
            println!("✅ Refinement ticket created with ID: {}", new_ticket_id);
        }
        Err(e) => {
            println!("❌ Error creating refinement ticket: {}", e);
        }
    }
    
    Ok(())
}

fn view_dependencies(project: &Project, ticket_id: &TicketId) -> Result<(), Box<dyn std::error::Error>> {
    let dependencies = project.graph.get_dependencies(ticket_id)
        .ok_or("Ticket not found")?;
    
    println!("\n⬇️ Dependencies:");
    if dependencies.is_empty() {
        println!("No dependencies.");
    } else {
        for dep_id in dependencies {
            if let Some(dep_node) = project.graph.get_ticket(dep_id) {
                println!("  • {} - {}", dep_id, dep_node.ticket.original_ticket.title);
            }
        }
    }
    
    Ok(())
}

fn view_dependents(project: &Project, ticket_id: &TicketId) -> Result<(), Box<dyn std::error::Error>> {
    let dependents = project.graph.get_dependents(ticket_id)
        .ok_or("Ticket not found")?;
    
    println!("\n⬆️ Dependents:");
    if dependents.is_empty() {
        println!("No dependents.");
    } else {
        for dep_id in dependents {
            if let Some(dep_node) = project.graph.get_ticket(dep_id) {
                println!("  • {} - {}", dep_id, dep_node.ticket.original_ticket.title);
            }
        }
    }
    
    Ok(())
}

async fn quick_refine(
    project: &mut Project,
    ticket_id: &TicketId,
    ticket_service: &TicketService<ClaudeClient>,
) -> Result<(), Box<dyn std::error::Error>> {
    let node = project.graph.get_ticket(ticket_id)
        .ok_or("Ticket not found")?;
    
    if node.ticket.decomposed_ticket.terms_needing_refinement.is_empty() {
        println!("\n✅ No terms need refinement in this ticket!");
        return Ok(());
    }
    
    println!("\n🔍 Terms Available for Refinement:");
    for (i, request) in node.ticket.decomposed_ticket.terms_needing_refinement.iter().enumerate() {
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
    
    print!("\nSelect term to refine (or 0 to go back): ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice = input.trim();
    
    if choice == "0" {
        return Ok(());
    }
    
    match choice.parse::<usize>() {
        Ok(index) if index > 0 && index <= node.ticket.decomposed_ticket.terms_needing_refinement.len() => {
            let refinement_request = &node.ticket.decomposed_ticket.terms_needing_refinement[index - 1];
            
            let refinement_context = format!(
                "Context from parent ticket '{}': {}\n\nTerm being refined: {}\nReason for refinement: {}\nAdditional context: {}",
                node.ticket.original_ticket.title,
                node.ticket.original_ticket.raw_input,
                refinement_request.term,
                refinement_request.reason,
                refinement_request.context
            );
            
            println!("🔄 Creating refinement ticket for term: {}", refinement_request.term);
            
            match ticket_service.decompose_ticket(refinement_context).await {
                Ok(refined_ticket) => {
                    let new_ticket_id = project.add_ticket(refined_ticket);
                    project.graph.add_dependency(&new_ticket_id, ticket_id)?;
                    println!("✅ Refinement ticket created with ID: {}", new_ticket_id);
                    
                    // Ask if they want to continue iterating on the new ticket
                    print!("Continue iterating on the new refinement ticket? (y/N): ");
                    io::stdout().flush()?;
                    let mut response = String::new();
                    io::stdin().read_line(&mut response)?;
                    
                    if response.trim().to_lowercase() == "y" {
                        println!("🚀 Switching to refinement ticket...");
                        Box::pin(ticket_detail_menu(project, &new_ticket_id, ticket_service)).await?;
                    }
                }
                Err(e) => {
                    println!("❌ Error creating refinement ticket: {}", e);
                }
            }
        }
        _ => {
            println!("Invalid selection.");
        }
    }
    
    Ok(())
}

async fn create_ticket(
    project: &mut Project,
    ticket_service: &TicketService<ClaudeClient>,
) -> Result<(), Box<dyn std::error::Error>> {
    print!("Enter ticket description: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let description = input.trim().to_string();
    
    println!("🔄 Creating ticket...");
    
    match ticket_service.decompose_ticket(description).await {
        Ok(ticket) => {
            let ticket_id = project.add_ticket(ticket);
            println!("✅ Ticket created with ID: {}", ticket_id);
            
            // Ask if user wants to start working on this ticket
            print!("Would you like to start iterating on this ticket? (y/N): ");
            io::stdout().flush()?;
            let mut response = String::new();
            io::stdin().read_line(&mut response)?;
            
            if response.trim().to_lowercase() == "y" {
                println!("🚀 Opening ticket for iteration...");
                Box::pin(ticket_detail_menu(project, &ticket_id, ticket_service)).await?;
            }
        }
        Err(e) => {
            println!("❌ Error creating ticket: {}", e);
        }
    }
    
    Ok(())
}