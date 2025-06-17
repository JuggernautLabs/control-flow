use client_implementations::deepseek::DeepSeekClient;
use client_implementations::claude::ClaudeClient;
use client_implementations::client::{QueryResolver, RetryConfig};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CodeAnalysis {
    /// The programming language detected
    pub language: String,
    /// Code quality score from 1-10
    pub quality_score: u8,
    /// List of suggestions for improvement
    pub suggestions: Vec<String>,
    /// Whether the code follows best practices
    pub follows_best_practices: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    println!("=== DeepSeek Client Demo ===");
    
    // Demo 1: Basic DeepSeek usage
    match DeepSeekClient::new() {
        Ok(deepseek_client) => {
            println!("✓ DeepSeek client created successfully");
            
            let resolver = QueryResolver::new(deepseek_client, RetryConfig::default());
            
            let prompt = "Analyze this Rust code snippet:\n\nfn add(a: i32, b: i32) -> i32 {\n    a + b\n}".to_string();
            
            match resolver.query_with_schema::<CodeAnalysis>(prompt).await {
                Ok(analysis) => {
                    println!("✓ DeepSeek analysis completed:");
                    println!("  Language: {}", analysis.language);
                    println!("  Quality Score: {}/10", analysis.quality_score);
                    println!("  Best Practices: {}", analysis.follows_best_practices);
                    println!("  Suggestions: {:?}", analysis.suggestions);
                },
                Err(e) => println!("✗ DeepSeek analysis failed: {}", e),
            }
        },
        Err(e) => {
            println!("✗ Failed to create DeepSeek client: {}", e);
            println!("  Make sure DEEPSEEK_API_KEY environment variable is set");
        }
    }
    
    println!("\n=== Model Selection Demo ===");
    
    // Demo 2: Using different models
    if let Ok(api_key) = std::env::var("DEEPSEEK_API_KEY") {
        let deepseek_client = DeepSeekClient::with_api_key(api_key)
            .with_model("deepseek-coder".to_string());
            
        println!("✓ DeepSeek client configured with deepseek-coder model");
        
        let resolver = QueryResolver::new(deepseek_client, RetryConfig::default());
        
        let prompt = "Write a simple Python function to calculate factorial".to_string();
        
        match resolver.query_with_schema::<String>(prompt).await {
            Ok(response) => {
                println!("✓ DeepSeek coder response:");
                println!("{}", response);
            },
            Err(e) => println!("✗ DeepSeek coder request failed: {}", e),
        }
    }
    
    println!("\n=== Claude vs DeepSeek Comparison ===");
    
    // Demo 3: Compare Claude and DeepSeek responses
    let prompt = "Explain the concept of ownership in Rust in one sentence.".to_string();
    
    // Try Claude
    if let Ok(claude_client) = ClaudeClient::new() {
        let claude_client = claude_client.with_model("claude-3-haiku-20240307".to_string());
        let claude_resolver = QueryResolver::new(claude_client, RetryConfig::default());
        
        match claude_resolver.query_with_schema::<String>(prompt.clone()).await {
            Ok(response) => {
                println!("Claude (haiku) response:");
                println!("  {}", response.trim());
            },
            Err(e) => println!("Claude request failed: {}", e),
        }
    }
    
    // Try DeepSeek
    if let Ok(deepseek_client) = DeepSeekClient::new() {
        let deepseek_resolver = QueryResolver::new(deepseek_client, RetryConfig::default());
        
        match deepseek_resolver.query_with_schema::<String>(prompt).await {
            Ok(response) => {
                println!("DeepSeek response:");
                println!("  {}", response.trim());
            },
            Err(e) => println!("DeepSeek request failed: {}", e),
        }
    }
    
    Ok(())
}