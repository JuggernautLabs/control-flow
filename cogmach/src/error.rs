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
