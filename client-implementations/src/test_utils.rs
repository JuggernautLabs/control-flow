use crate::client::{LowLevelClient, QueryResolver, RetryConfig, MockVoid};
use crate::claude::ClaudeClient;
use crate::deepseek::DeepSeekClient;
use std::env;
use std::sync::OnceLock;

/// Supported AI clients for testing
#[derive(Debug, Clone)]
pub enum ClientType {
    Claude,
    DeepSeek, 
    Mock,
}

impl ClientType {
    /// Parse client type from string (case insensitive)
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "claude" => Ok(Self::Claude),
            "deepseek" => Ok(Self::DeepSeek),
            "mock" => Ok(Self::Mock),
            _ => Err(format!("Unknown client type: '{}'. Supported: claude, deepseek, mock", s))
        }
    }
    
    /// Get the default client type based on available API keys
    pub fn default() -> Self {
        // Check for API keys in order of preference
        if env::var("ANTHROPIC_API_KEY").is_ok() || 
           std::fs::read_to_string(".env").map_or(false, |content| content.contains("ANTHROPIC_API_KEY")) {
            Self::Claude
        } else if env::var("DEEPSEEK_API_KEY").is_ok() || 
                 std::fs::read_to_string(".env").map_or(false, |content| content.contains("DEEPSEEK_API_KEY")) {
            Self::DeepSeek
        } else {
            Self::Mock
        }
    }
}

/// Global lazy-initialized client instance
static CLIENT_INSTANCE: OnceLock<Box<dyn LowLevelClient + Send + Sync>> = OnceLock::new();

/// Get the configured client type from environment
pub fn get_client_type() -> ClientType {
    env::var("TEST_CLIENT")
        .ok()
        .and_then(|s| ClientType::from_str(&s).ok())
        .unwrap_or_else(ClientType::default)
}

/// Create a client instance based on the client type
fn create_client(client_type: ClientType) -> Result<Box<dyn LowLevelClient + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
    match client_type {
        ClientType::Claude => {
            let client = ClaudeClient::new()
                .map_err(|e| format!("Failed to create Claude client: {}", e))?;
            Ok(Box::new(client))
        },
        ClientType::DeepSeek => {
            let client = DeepSeekClient::new()
                .map_err(|e| format!("Failed to create DeepSeek client: {}", e))?;
            Ok(Box::new(client))
        },
        ClientType::Mock => {
            Ok(Box::new(MockVoid))
        }
    }
}

/// Get the global client instance (lazy initialized)
pub fn get_test_client() -> &'static Box<dyn LowLevelClient + Send + Sync> {
    CLIENT_INSTANCE.get_or_init(|| {
        let client_type = get_client_type();
        
        match create_client(client_type.clone()) {
            Ok(client) => {
                eprintln!("✅ Created test client: {:?}", client_type);
                client
            },
            Err(e) => {
                eprintln!("⚠️  Failed to create {:?} client: {}", client_type, e);
                eprintln!("   Falling back to MockVoid for tests");
                Box::new(MockVoid)
            }
        }
    })
}

/// Create a QueryResolver with the configured test client
/// Note: We clone the Box to avoid lifetime issues
pub fn create_test_resolver() -> QueryResolver<Box<dyn LowLevelClient + Send + Sync>> {
    let _client = get_test_client();
    
    // Clone the client based on its type
    let cloned_client: Box<dyn LowLevelClient + Send + Sync> = match get_client_type() {
        ClientType::Claude => {
            // We can't clone Claude client easily, so create a new one
            match ClaudeClient::new() {
                Ok(client) => Box::new(client),
                Err(_) => Box::new(MockVoid),
            }
        },
        ClientType::DeepSeek => {
            // We can't clone DeepSeek client easily, so create a new one
            match DeepSeekClient::new() {
                Ok(client) => Box::new(client),
                Err(_) => Box::new(MockVoid),
            }
        },
        ClientType::Mock => Box::new(MockVoid),
    };
    
    QueryResolver::new(cloned_client, RetryConfig::default())
}

/// Create a QueryResolver with custom retry configuration
pub fn create_test_resolver_with_config(config: RetryConfig) -> QueryResolver<Box<dyn LowLevelClient + Send + Sync>> {
    let cloned_client: Box<dyn LowLevelClient + Send + Sync> = match get_client_type() {
        ClientType::Claude => {
            match ClaudeClient::new() {
                Ok(client) => Box::new(client),
                Err(_) => Box::new(MockVoid),
            }
        },
        ClientType::DeepSeek => {
            match DeepSeekClient::new() {
                Ok(client) => Box::new(client),
                Err(_) => Box::new(MockVoid),
            }
        },
        ClientType::Mock => Box::new(MockVoid),
    };
    
    QueryResolver::new(cloned_client, config)
}

/// Check if we should skip integration tests (i.e., we're using MockVoid)
pub fn should_skip_integration_tests() -> bool {
    matches!(get_client_type(), ClientType::Mock)
}

/// Print test client information
pub fn print_test_client_info() {
    let client_type = get_client_type();
    println!("🧪 Test Configuration:");
    println!("   Client: {:?}", client_type);
    
    match client_type {
        ClientType::Claude => {
            if env::var("ANTHROPIC_API_KEY").is_ok() {
                println!("   API Key: ✅ Found in environment");
            } else {
                println!("   API Key: ✅ Found in .env file");
            }
        },
        ClientType::DeepSeek => {
            if env::var("DEEPSEEK_API_KEY").is_ok() {
                println!("   API Key: ✅ Found in environment");
            } else {
                println!("   API Key: ✅ Found in .env file");
            }
        },
        ClientType::Mock => {
            println!("   Mode: Mock (no API calls will be made)");
        }
    }
    
    println!("   Override with: TEST_CLIENT=claude|deepseek|mock");
    println!();
}