use crate::client::LowLevelClient;
use crate::error::{AIError, ClaudeError};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug, instrument};

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
}

#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Debug, Deserialize)]
struct ClaudeContent {
    text: String,
}

pub struct ClaudeClient {
    api_key: String,
    client: Client,
    model: String,
}

impl ClaudeClient {
    pub fn new(api_key: String) -> Self {
        info!(model = "claude-3-5-haiku-20241022", "Creating new Claude client");
        Self {
            api_key,
            client: Client::new(),
            model: "claude-3-5-haiku-20241022".to_string(),
        }
    }
    
    pub fn with_model(mut self, model: String) -> Self {
        info!(model = %model, "Setting Claude model");
        self.model = model;
        self
    }
}

#[async_trait]
impl LowLevelClient for ClaudeClient {
    #[instrument(skip(self, prompt), fields(prompt_len = prompt.len(), model = %self.model))]
    async fn ask_raw(&self, prompt: String) -> Result<String, AIError> {
        debug!(model = %self.model, prompt_len = prompt.len(), "Preparing Claude API request");
        let request = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 4096,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: prompt,
            }],
        };
        
        debug!("Sending request to Claude API");
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "HTTP request failed");
                AIError::Claude(ClaudeError::Http(e.to_string()))
            })?;
            
        debug!(status = %response.status(), "Received response from Claude API");
            
        if response.status() == 429 {
            warn!("Claude API rate limit exceeded");
            return Err(AIError::Claude(ClaudeError::RateLimit));
        }
        
        if response.status() == 401 {
            error!("Claude API authentication failed");
            return Err(AIError::Claude(ClaudeError::Authentication));
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(status = %status, error = %error_text, "Claude API error");
            return Err(AIError::Claude(ClaudeError::Api(error_text)));
        }
        
        let claude_response: ClaudeResponse = response
            .json()
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to parse Claude response JSON");
                AIError::Claude(ClaudeError::Http(e.to_string()))
            })?;
            
        debug!(content_count = claude_response.content.len(), "Parsed Claude response");
            
        let result = claude_response
            .content
            .first()
            .map(|content| content.text.clone())
            .ok_or_else(|| {
                error!("No content in Claude response");
                AIError::Claude(ClaudeError::Api("No content in response".to_string()))
            });
            
        match &result {
            Ok(text) => info!(response_len = text.len(), "Successfully received Claude response"),
            Err(e) => error!(error = %e, "Failed to extract content from Claude response"),
        }
        
        result
    }
}