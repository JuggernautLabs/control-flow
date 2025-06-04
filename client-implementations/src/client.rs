use crate::error::{QueryResolverError, AIError};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use async_trait::async_trait;
use tracing::{info, warn, error, debug, instrument};
use regex::Regex;

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
        max_retries.insert("json_parse_error".to_string(), 0);
        
        Self {
            max_retries,
            default_max_retries: 1,
        }
    }
}
#[async_trait]
pub trait LowLevelClient {
    async fn ask_raw(&self, prompt: String) -> Result<String, AIError>;
    
    fn find_json(&self, response: &str) -> String {
        debug!(response_len = response.len(), "Attempting to extract JSON from response");
        
        // First try to extract JSON from markdown code blocks
        if let Some(json_content) = self.extract_json_from_markdown(response) {
            debug!(extracted_len = json_content.len(), "Successfully extracted JSON from markdown");
            return json_content;
        }
        
        // If no markdown found, try the raw response
        debug!("No markdown JSON found, using raw response");
        response.to_string()
    }
    
    fn extract_json_from_markdown(&self, response: &str) -> Option<String> {
        // Try different markdown patterns
        let patterns = [
            r"```json\s*\n([\s\S]*?)\n\s*```",
            r"```json([\s\S]*?)```",
            r"```\s*\n([\s\S]*?)\n\s*```",
            r"```([\s\S]*?)```",
        ];
        
        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(captures) = re.captures(response) {
                    if let Some(json_match) = captures.get(1) {
                        let content = json_match.as_str().trim();
                        debug!(pattern = pattern, content_len = content.len(), "Found JSON in markdown");
                        return Some(content.to_string());
                    }
                }
            }
        }
        
        None
    }
    
    async fn ask_json(&self, prompt: String) -> Result<String, AIError> {
        debug!(prompt_len = prompt.len(), "Starting ask_json");
        let raw_response = self.ask_raw(prompt).await?;
        let json_content = self.find_json(&raw_response);
        debug!(raw_len = raw_response.len(), json_len = json_content.len(), "Extracted JSON from response");
        Ok(json_content)
    }
    
    #[instrument(skip(self, prompt, config), fields(prompt_len = prompt.len()))]
    async fn ask_with_retry<T>(&self, prompt: String, config: &RetryConfig) -> Result<T, QueryResolverError>
    where
        T: DeserializeOwned,
    {
        let mut attempt = 0;
        let mut context = String::new();
        
        info!(attempt = 0, max_retries = config.default_max_retries, "Starting retry loop for prompt");
        
        loop {
            let full_prompt = if context.is_empty() {
                prompt.clone()
            } else {
                format!("{}\n\nPrevious attempt failed: {}\nPlease fix the issue and respond with valid JSON.", prompt, context)
            };
            
            debug!(attempt = attempt + 1, prompt_len = full_prompt.len(), "Making API call");
            match self.ask_json(full_prompt).await {
                Ok(response) => {
                    debug!(response_len = response.len(), "Received API response");
                    match serde_json::from_str::<T>(&response) {
                        Ok(parsed) => {
                            info!(attempt = attempt + 1, "Successfully parsed JSON response");
                            return Ok(parsed);
                        },
                        Err(json_err) => {
                            error!(
                                error = %json_err, 
                                response_preview = &response[..response.len().min(200)],
                                "JSON parsing failed, returning raw response"
                            );
                            return Err(QueryResolverError::JsonDeserialization(json_err, response));
                        }
                    }
                }
                Err(ai_error) => {
                    warn!(error = %ai_error, attempt = attempt + 1, "API call failed");
                    let error_type = match &ai_error {
                        AIError::Claude(claude_err) => match claude_err {
                            crate::error::ClaudeError::RateLimit => "rate_limit",
                            crate::error::ClaudeError::Http(_) => "http_error",
                            crate::error::ClaudeError::Api(_) => "api_error",
                            _ => "other",
                        },
                        AIError::OpenAI(openai_err) => match openai_err {
                            crate::error::OpenAIError::RateLimit => "rate_limit",
                            crate::error::OpenAIError::Http(_) => "http_error", 
                            crate::error::OpenAIError::Api(_) => "api_error",
                            _ => "other",
                        },
                    };
                    
                    let max_retries = config.max_retries.get(error_type)
                        .unwrap_or(&config.default_max_retries);
                    
                    if attempt >= *max_retries {
                        error!(
                            error = %ai_error, 
                            error_type = error_type,
                            max_retries = max_retries,
                            "Max retries exceeded for API error"
                        );
                        return Err(QueryResolverError::Ai(ai_error));
                    }
                    
                    info!(
                        error_type = error_type,
                        attempt = attempt + 1,
                        max_retries = max_retries,
                        "Retrying after API error"
                    );
                    context = format!("API call failed: {}", ai_error);
                    attempt += 1;
                }
            }
        }
    }
}

pub struct QueryResolver<C: LowLevelClient> {
    client: C,
    config: RetryConfig,
}

impl<C: LowLevelClient + Send + Sync> QueryResolver<C> {
    pub fn new(client: C, config: RetryConfig) -> Self {
        info!(default_max_retries = config.default_max_retries, "Creating new QueryResolver with retry config");
        Self { client, config }
    }
    
    #[instrument(skip(self, prompt), fields(prompt_len = prompt.len()))]
    pub async fn query<T>(&self, prompt: String) -> Result<T, QueryResolverError>
    where
        T: DeserializeOwned,
    {
        info!(prompt_len = prompt.len(), "Starting query");
        let result = self.client.ask_with_retry(prompt, &self.config).await;
        match &result {
            Ok(_) => info!("Query completed successfully"),
            Err(e) => error!(error = %e, "Query failed"),
        }
        result
    }
}