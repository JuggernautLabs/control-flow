#!/bin/bash

# Initialize Fundamental Primitives Cognition Machine Project
set -e

PROJECT_NAME="cogmach"
echo "🧠 Initializing Cognition Machine Project: $PROJECT_NAME"

# Create project directory
mkdir -p $PROJECT_NAME
cd $PROJECT_NAME

# Initialize Cargo project
echo "📦 Creating Rust project..."
cargo init --name $PROJECT_NAME

# Update Cargo.toml with dependencies
cat > Cargo.toml << 'EOF'
[package]
name = "cogmach"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "cogmach-cli"
path = "src/bin/cli.rs"

[[bin]]
name = "cogmach-experiment"
path = "src/bin/experiment.rs"

[dependencies]
# Core dependencies
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"

# Tracing and logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-appender = "0.2"

# HTTP client for AI APIs
reqwest = { version = "0.11", features = ["json"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# CLI utilities
clap = { version = "4.0", features = ["derive"] }

# Testing
regex = "1.0"

[dev-dependencies]
tokio-test = "0.4"
EOF

# Create src directory structure
mkdir -p src/bin

# Create error types
cat > src/error.rs << 'EOF'
use thiserror::Error;
use serde_json;

#[derive(Error, Debug)]
pub enum QueryResolverError {
    #[error("AI error: {0}")]
    Ai(#[from] AIError),
    
    #[error("JSON deserialization failed: {0}. Response was: {1}")]
    JsonDeserialization(serde_json::Error, String),
    
    #[error("Other error: {0}")]
    Other(String),
}

#[derive(Error, Debug)]
pub enum AIError {
    #[error("Claude error: {0}")]
    Claude(#[from] ClaudeError),
    
    #[error("OpenAI error: {0}")]
    OpenAI(#[from] OpenAIError),
}

#[derive(Error, Debug)]
pub enum ClaudeError {
    #[error("Rate limit exceeded")]
    RateLimit,
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("API error: {0}")]
    Api(String),
    
    #[error("Authentication failed")]
    Auth,
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

#[derive(Error, Debug)]
pub enum OpenAIError {
    #[error("Rate limit exceeded")]
    RateLimit,
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("API error: {0}")]
    Api(String),
    
    #[error("Authentication failed")]
    Auth,
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}
EOF

# Create client interface (provided from your existing code)
cat > src/client.rs << 'EOF'
// This file should contain your existing client.rs implementation
// For now, we'll create a basic structure that matches the interface

use crate::error::{QueryResolverError, AIError, ClaudeError};
use serde::de::DeserializeOwned;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use async_trait::async_trait;
use tracing::{info, warn, error, debug, instrument};
use regex::Regex;

// Basic mock implementation for testing
pub struct MockVoid;

#[async_trait]
impl LowLevelClient for MockVoid {
    async fn ask_raw(&self, _prompt: String) -> Result<String, AIError> {
        Ok(r#"{"artifact": "# Mock TicTacToe API\nprint('Hello from mock!')", "confidence": 0.5}"#.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: HashMap<String, usize>,
    pub default_max_retries: usize,
}

impl Default for RetryConfig {
    fn default() -> Self {
        let mut max_retries = HashMap::new();
        max_retries.insert("rate_limit".to_string(), 1);
        max_retries.insert("api_error".to_string(), 1);
        max_retries.insert("http_error".to_string(), 1);
        max_retries.insert("json_parse_error".to_string(), 2);
        
        Self {
            max_retries,
            default_max_retries: 1,
        }
    }
}

#[async_trait]
pub trait LowLevelClient {
    async fn ask_raw(&self, prompt: String) -> Result<String, AIError>;
}

pub struct QueryResolver<C: LowLevelClient> {
    client: C,
    config: RetryConfig,
}

impl<C: LowLevelClient + Send + Sync> QueryResolver<C> {
    pub fn new(client: C, config: RetryConfig) -> Self {
        Self { client, config }
    }
    
    pub async fn query<T>(&self, prompt: String) -> Result<T, QueryResolverError>
    where
        T: DeserializeOwned + Send,
    {
        let response = self.client.ask_raw(prompt).await?;
        let parsed: T = serde_json::from_str(&response)
            .map_err(|e| QueryResolverError::JsonDeserialization(e, response))?;
        Ok(parsed)
    }
}
EOF

# Create the main cognition machine implementation
# (This would contain your updated artifact code)
cat > src/cogmach.rs << 'EOF'
// The fundamental primitives cognition machine implementation
// Copy the artifact content here with all the tracing instrumentation
EOF

# Create main lib.rs
cat > src/lib.rs << 'EOF'
pub mod error;
pub mod client;
pub mod cogmach;

pub use cogmach::*;
EOF

# Create CLI binary
cat > src/bin/cli.rs << 'EOF'
use clap::{Parser, Subcommand};
use tracing::{info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use cogmach::client::MockVoid;
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
EOF

# Create experiment binary
cat > src/bin/experiment.rs << 'EOF'
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
EOF

# Create development environment setup
cat > .env.example << 'EOF'
# AI API Keys (copy to .env and fill in)
CLAUDE_API_KEY=your_claude_api_key_here
OPENAI_API_KEY=your_openai_api_key_here

# Logging configuration
RUST_LOG=cogmach=debug,info
RUST_BACKTRACE=1
EOF

# Create .gitignore
cat > .gitignore << 'EOF'
/target
.env
*.log
Cargo.lock
.DS_Store
*.tmp
*.temp
/logs
EOF

# Create README
cat > README.md << 'EOF'
# Fundamental Primitives Cognition Machine

A Rust implementation of the fundamental observe/generate primitives for automated software development.

## Core Concept

All software development reduces to two operations:
- **Observe(Lens)**: Discover what IS
- **Generate(Specification)**: Create what SHOULD BE

## Quick Start

```bash
# Copy environment variables
cp .env.example .env
# Edit .env with your API keys

# Run the experiment demo
cargo run --bin cogmach-experiment

# Run interactive CLI
cargo run --bin cogmach-cli experiment "create a tictactoe api"

# Show system info
cargo run --bin cogmach-cli info

# Enable debug logging
cargo run --bin cogmach-cli --debug experiment "your goal"

# Enable trace logging
cargo run --bin cogmach-cli --trace experiment "your goal"
```

## Architecture

- **Mechanical Observation**: Direct system interrogation (file existence, code execution)
- **Semantic Observation**: AI-powered analysis (code structure, relationships)
- **Generation**: AI-powered content creation based on deltas
- **Execution**: Real code execution with pytest integration

## Tracing

The system uses structured tracing throughout:
- All major operations are instrumented
- Detailed logging of AI interactions
- Performance metrics and execution results
- Error tracking and debugging information

## Development

```bash
# Run tests
cargo test

# Check formatting
cargo fmt

# Run clippy
cargo clippy

# Build release
cargo build --release
```
EOF

# Create a simple test
mkdir -p tests
cat > tests/integration_test.rs << 'EOF'
use cogmach::*;
use cogmach::client::MockVoid;

#[tokio::test]
async fn test_cognition_machine_creation() {
    let client = MockVoid;
    let _cogmach = FundamentalCognitionMachine::new(client);
    // Basic test to ensure the machine can be created
}

#[tokio::test]
async fn test_fundamental_experiment() {
    // Test that the experiment function runs without panic
    let result = run_fundamental_experiment().await;
    assert!(result.is_ok());
}
EOF

# Build the project
echo "🔨 Building project..."
cargo check

echo "✅ Project initialized successfully!"
echo ""
echo "📋 Next steps:"
echo "1. Copy .env.example to .env and add your AI API keys"
echo "2. Copy your complete client.rs implementation to src/client.rs"
echo "3. Copy the updated cogmach artifact to src/cogmach.rs"
echo "4. Run: cargo run --bin cogmach-experiment"
echo "5. Run: cargo run --bin cogmach-cli experiment 'create a tictactoe api'"
echo ""
echo "🧠 The Cognition Machine is ready for experimentation!"
