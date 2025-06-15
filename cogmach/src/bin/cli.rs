use clap::{Parser, Subcommand};
use tracing::{info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use client_implementations::client::MockVoid;
use cogmach::FundamentalCognitionMachine;

#[derive(Parser)]
#[command(name = "cogmach")]
#[command(about = "Fundamental Primitives Cognition Machine")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
    
    /// Enable trace logging
    #[arg(short, long)]
    trace: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the fundamental experiment
    Experiment {
        /// Goal to achieve
        goal: String,
    },
    /// Show system information
    Info,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Initialize tracing
    let level = if cli.trace {
        Level::TRACE
    } else if cli.debug {
        Level::DEBUG
    } else {
        Level::INFO
    };
    
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_line_number(true)
                .with_file(true)
        )
        .with(EnvFilter::from_default_env().add_directive(level.into()))
        .init();

    info!("🧠 Cognition Machine CLI Starting");

    match cli.command {
        Commands::Experiment { goal } => {
            info!(goal = %goal, "Starting experiment");
            
            let client = MockVoid;
            let cogmach = FundamentalCognitionMachine::new(client);
            let result = cogmach.achieve_goal(&goal).await?;
            
            println!("🎉 Experiment complete!");
            println!("Dialogues run: {}", result.dialogues.len());
            for (i, dialogue) in result.dialogues.iter().enumerate() {
                println!("Dialogue {}: {} exchanges, complete: {}", 
                    i + 1, dialogue.exchanges.len(), dialogue.is_complete);
            }
        },
        Commands::Info => {
            println!("🧠 Fundamental Primitives Cognition Machine");
            println!("Version: {}", env!("CARGO_PKG_VERSION"));
            println!("Built on the observe/generate duality");
            println!();
            println!("Core Operations:");
            println!("  - Observe(Lens): Discover what IS");
            println!("  - Generate(Specification): Create what SHOULD BE");
            println!("  - Execute: Run and verify generated content");
        }
    }

    Ok(())
}
