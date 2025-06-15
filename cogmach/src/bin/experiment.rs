use tracing::{info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use cogmach::run_fundamental_experiment;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing with detailed output
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_line_number(true)
                .with_file(true)
                .pretty()
        )
        .with(EnvFilter::from_default_env().add_directive(Level::DEBUG.into()))
        .init();

    info!("🧠 Starting Fundamental Primitives Experiment");
    
    run_fundamental_experiment().await?;
    
    info!("✅ Experiment complete");
    Ok(())
}
