use control_flow::ticket_service::TicketService;
use client_implementations::claude::ClaudeClient;
use client_implementations::client::RetryConfig;
use std::env;
use std::io::{self, Write};
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    
    // Initialize structured logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "control_flow=info,client_implementations=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
    
    info!("Starting Ticket Decomposition Engine CLI");
    
    let api_key = env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");
    
    info!("Loaded API key from environment");
    
    let claude_client = ClaudeClient::new(api_key);
    let retry_config = RetryConfig::default();
    let ticket_service = TicketService::new(claude_client, retry_config);
    
    info!("Initialized ticket service and Claude client");
    
    println!("🎫 Ticket Decomposition Engine");
    println!("Enter your request (or 'quit' to exit):");
    info!("CLI started, waiting for user input");
    
    loop {
        print!("> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input == "quit" || input == "exit" {
            info!("User requested exit");
            println!("Goodbye!");
            break;
        }
        
        if input.is_empty() {
            continue;
        }
        
        info!(input_len = input.len(), "Processing user input");
        
        println!("🔄 Processing ticket decomposition...");
        
        match ticket_service.decompose_ticket(input.to_string()).await {
            Ok(decomposition) => {
                info!("Ticket decomposition successful");
                println!("✅ Ticket decomposed successfully:");
                println!("{}", serde_json::to_string_pretty(&decomposition)?);
            }
            Err(e) => {
                error!(error = %e, "Ticket decomposition failed");
                println!("❌ Error decomposing ticket: {}", e);
            }
        }
        
        println!("\nEnter another request (or 'quit' to exit):");
        info!("Ready for next user input");
    }
    
    Ok(())
}