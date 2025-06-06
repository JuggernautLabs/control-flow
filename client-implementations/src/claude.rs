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
    content: ClaudeMessageContent,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum ClaudeMessageContent {
    Simple(String),
    Structured(Vec<ClaudeContentBlock>),
}

#[derive(Debug, Serialize)]
struct ClaudeContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_control: Option<CacheControl>,
}

#[derive(Debug, Serialize)]
struct CacheControl {
    #[serde(rename = "type")]
    cache_type: String,
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
    enable_caching: bool,
}

impl ClaudeClient {
    pub fn new(api_key: String) -> Self {
        info!(model = "claude-3-5-sonnet-20241022", "Creating new Claude client with caching support");
        Self {
            api_key,
            client: Client::new(),
            model: "claude-3-5-sonnet-20241022".to_string(), // Use Sonnet for better caching
            enable_caching: true,
        }
    }
    
    pub fn with_model(mut self, model: String) -> Self {
        info!(model = %model, "Setting Claude model");
        self.model = model;
        self
    }
    
    pub fn with_caching(mut self, enable: bool) -> Self {
        info!(enable_caching = enable, "Setting cache control");
        self.enable_caching = enable;
        self
    }
    
    /// Split prompt into cacheable base instructions and variable content
    /// Returns (base_instructions, variable_content)
    fn split_prompt_for_caching(&self, prompt: String) -> (String, String) {
        // Look for our specific markers that indicate transition to variable content
        if let Some(split_pos) = prompt.find("--- BEGIN ANALYSIS ---") {
            let base_instructions = prompt[..split_pos].trim().to_string();
            let variable_content = prompt[split_pos..].to_string();
            
            debug!(
                base_len = base_instructions.len(),
                variable_len = variable_content.len(),
                "Split prompt using analysis marker"
            );
            
            return (base_instructions, variable_content);
        }
        
        if let Some(split_pos) = prompt.find("--- BEGIN ALIGNMENT ANALYSIS ---") {
            let base_instructions = prompt[..split_pos].trim().to_string();
            let variable_content = prompt[split_pos..].to_string();
            
            debug!(
                base_len = base_instructions.len(),
                variable_len = variable_content.len(),
                "Split prompt using alignment analysis marker"
            );
            
            return (base_instructions, variable_content);
        }
        
        // Fallback: look for common patterns to identify the base instructions vs variable content
        let lines: Vec<&str> = prompt.lines().collect();
        let mut split_index = 0;
        
        for (i, line) in lines.iter().enumerate() {
            // Look for markers that indicate transition to variable content
            if line.contains("Artifact Type:") || 
               line.contains("CLAIM TO EVALUATE:") ||
               line.contains("Content:") ||
               line.contains("```") {
                split_index = i;
                break;
            }
        }
        
        // Ensure we have enough content for caching (minimum 1024 tokens ≈ 3000 chars for safety)
        let base_content = lines[..split_index].join("\n");
        if base_content.len() < 2500 {
            // If base instructions are too short, include more context
            split_index = std::cmp::min(lines.len() * 2 / 3, lines.len());
        }
        
        let base_instructions = lines[..split_index].join("\n");
        let variable_content = lines[split_index..].join("\n");
        
        debug!(
            base_len = base_instructions.len(),
            variable_len = variable_content.len(),
            "Split prompt using fallback logic"
        );
        
        (base_instructions, variable_content)
    }
}

#[async_trait]
impl LowLevelClient for ClaudeClient {
    #[instrument(skip(self, prompt), fields(prompt_len = prompt.len(), model = %self.model, caching_enabled = %self.enable_caching))]
    async fn ask_raw(&self, prompt: String) -> Result<String, AIError> {
        debug!(model = %self.model, prompt_len = prompt.len(), caching_enabled = %self.enable_caching, "Preparing Claude API request");
        
        let content = if self.enable_caching && prompt.len() > 3000 {
            // Split prompt for optimal caching
            let (base_instructions, variable_content) = self.split_prompt_for_caching(prompt);
            
            info!(
                base_len = base_instructions.len(),
                variable_len = variable_content.len(),
                "Using structured prompt with caching"
            );
            
            ClaudeMessageContent::Structured(vec![
                // Base instructions - cacheable
                ClaudeContentBlock {
                    block_type: "text".to_string(),
                    text: base_instructions,
                    cache_control: Some(CacheControl {
                        cache_type: "ephemeral".to_string(),
                    }),
                },
                // Variable content - not cached
                ClaudeContentBlock {
                    block_type: "text".to_string(),
                    text: variable_content,
                    cache_control: None,
                },
            ])
        } else {
            // Fallback to simple content for short prompts
            debug!("Using simple prompt (too short for caching or caching disabled)");
            ClaudeMessageContent::Simple(prompt)
        };
        
        let request = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 4096,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content,
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