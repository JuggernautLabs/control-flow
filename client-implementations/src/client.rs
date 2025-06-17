use crate::error::{QueryResolverError, AIError};
use serde::de::DeserializeOwned;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use async_trait::async_trait;
use tracing::{info, warn, error, debug, instrument};
use regex::Regex;
use schemars::{JsonSchema, schema_for};

pub trait SmartClient: LowLevelClient + Send + Sync {}

// Auto-implement for any type that meets the bounds
impl<T> SmartClient for T where T: LowLevelClient + Send + Sync {}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientResponse {
    /// The raw response from the AI model
    pub raw: String,
    /// Text segments that couldn't be parsed as JSON
    pub segmented: Vec<String>,
    /// The extracted JSON response, if any
    pub json_response: Option<String>,
    /// Time taken to process the response in milliseconds
    pub processing_time_ms: u64,
    /// Whether JSON extraction was successful
    pub json_extraction_successful: bool,
    /// Method used to extract JSON (markdown, advanced, line_by_line, raw)
    pub extraction_method: String,
}

impl ClientResponse {
    pub fn new(raw: String, processing_time_ms: u64) -> Self {
        Self {
            raw,
            segmented: Vec::new(),
            json_response: None,
            processing_time_ms,
            json_extraction_successful: false,
            extraction_method: "none".to_string(),
        }
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
    
    fn process_response(&self, raw_response: String, processing_time_ms: u64) -> ClientResponse {
        debug!(response_len = raw_response.len(), "Processing response to extract JSON");
        
        let start_time = std::time::Instant::now();
        let mut response = ClientResponse::new(raw_response.clone(), processing_time_ms);
        
        // First try to extract JSON from markdown code blocks
        if let Some(json_content) = self.extract_json_from_markdown(&raw_response) {
            debug!(extracted_len = json_content.len(), "Successfully extracted JSON from markdown");
            response.json_response = Some(json_content);
            response.json_extraction_successful = true;
            response.extraction_method = "markdown".to_string();
            response.segmented = self.segment_non_json_content(&raw_response, response.json_response.as_ref().unwrap());
            return response;
        }
        
        // Try advanced JSON extraction
        if let Some(json_content) = self.extract_json_advanced(&raw_response) {
            debug!(extracted_len = json_content.len(), "Successfully extracted JSON using advanced method");
            response.json_response = Some(json_content);
            response.json_extraction_successful = true;
            response.extraction_method = "advanced".to_string();
            response.segmented = self.segment_non_json_content(&raw_response, response.json_response.as_ref().unwrap());
            return response;
        }
        
        // If no JSON found, segment the entire response
        debug!("No JSON found, treating entire response as segmented content");
        response.segmented = vec![raw_response.clone()];
        response.json_response = Some(raw_response.clone()); // Fallback for backward compatibility
        response.extraction_method = "raw".to_string();
        
        let processing_end = std::time::Instant::now();
        response.processing_time_ms += processing_end.duration_since(start_time).as_millis() as u64;
        
        response
    }
    
    fn segment_non_json_content(&self, raw_response: &str, json_content: &str) -> Vec<String> {
        // Split the raw response into segments, excluding the JSON content
        if let Some(json_start) = raw_response.find(json_content) {
            let mut segments = Vec::new();
            
            // Add content before JSON
            let before = &raw_response[..json_start].trim();
            if !before.is_empty() {
                segments.push(before.to_string());
            }
            
            // Add content after JSON
            let after_start = json_start + json_content.len();
            if after_start < raw_response.len() {
                let after = &raw_response[after_start..].trim();
                if !after.is_empty() {
                    segments.push(after.to_string());
                }
            }
            
            segments
        } else {
            // If JSON not found in raw response, return the whole response as segment
            vec![raw_response.to_string()]
        }
    }
    
    fn find_json(&self, response: &str) -> String {
        // Backward compatibility method - just return the JSON part
        let client_response = self.process_response(response.to_string(), 0);
        client_response.json_response.unwrap_or_else(|| response.to_string())
    }
    
    /// Advanced JSON extraction that searches for valid JSON objects in the response
    fn extract_json_advanced(&self, response: &str) -> Option<String> {
        debug!("Starting advanced JSON extraction");
        
        // Find all positions where '{' appears
        let open_positions: Vec<usize> = response.char_indices()
            .filter_map(|(i, c)| if c == '{' { Some(i) } else { None })
            .collect();
        
        if open_positions.is_empty() {
            debug!("No opening braces found in response");
            return None;
        }
        
        // Try each opening brace position
        for &start_pos in &open_positions {
            debug!(start_pos = start_pos, "Trying JSON extraction from position");
            
            if let Some(json_str) = self.find_matching_json_object(&response[start_pos..]) {
                let full_json = &response[start_pos..start_pos + json_str.len()];
                
                // Test if this is valid JSON by attempting to parse it
                if serde_json::from_str::<serde_json::Value>(full_json).is_ok() {
                    debug!(json_len = full_json.len(), "Found valid JSON object");
                    return Some(full_json.to_string());
                }
            }
        }
        
        // Try line-by-line approach as fallback
        debug!("Trying line-by-line JSON extraction");
        self.try_line_by_line_json(response)
    }
    
    /// Find a complete JSON object starting from the given text
    fn find_matching_json_object(&self, text: &str) -> Option<String> {
        let mut brace_count = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let chars = text.char_indices();
        
        // Skip to first '{'
        if !text.starts_with('{') {
            return None;
        }
        
        for (i, c) in chars {
            if escape_next {
                escape_next = false;
                continue;
            }
            
            match c {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '{' if !in_string => brace_count += 1,
                '}' if !in_string => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        // Found complete JSON object
                        return Some(&text[..=i]).map(|s| s.to_string());
                    }
                }
                _ => {}
            }
        }
        
        None
    }
    
    /// Try to find JSON by testing each line that starts with '{'
    fn try_line_by_line_json(&self, response: &str) -> Option<String> {
        let lines: Vec<&str> = response.lines().collect();
        
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('{') {
                debug!(line_num = i + 1, "Testing line starting with opening brace");
                
                // Try this single line first
                if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
                    debug!(line_num = i + 1, "Found valid single-line JSON");
                    return Some(trimmed.to_string());
                }
                
                // Try combining this line with subsequent lines
                for end_line in i + 1..lines.len() {
                    let combined = lines[i..=end_line].join("\n");
                    if serde_json::from_str::<serde_json::Value>(&combined).is_ok() {
                        debug!(start_line = i + 1, end_line = end_line + 1, "Found valid multi-line JSON");
                        return Some(combined);
                    }
                    
                    // Stop if we've gone too far (e.g., more than 50 lines)
                    if end_line - i > 50 {
                        break;
                    }
                }
            }
        }
        
        None
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
        T: DeserializeOwned + Send,
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
            match self.ask_json(full_prompt.clone()).await {
                Ok(response) => {
                    debug!(response_len = response.len(), "Received API response");
                    match serde_json::from_str::<T>(&response) {
                        Ok(parsed) => {
                            info!(attempt = attempt + 1, "Successfully parsed JSON response");
                            return Ok(parsed);
                        },
                        Err(json_err) => {
                            warn!(
                                error = %json_err, 
                                response_preview = &response[..response.len().min(200)],
                                "Initial JSON parsing failed, trying advanced extraction"
                            );
                            
                            // Try advanced JSON extraction on the raw response
                            if let Ok(raw_response) = self.ask_raw(full_prompt.clone()).await {
                                if let Some(extracted_json) = self.extract_json_advanced(&raw_response) {
                                    debug!(extracted_len = extracted_json.len(), "Trying to parse extracted JSON after initial failure");
                                    match serde_json::from_str::<T>(&extracted_json) {
                                        Ok(parsed) => {
                                            info!(attempt = attempt + 1, "Successfully parsed extracted JSON after initial deserialization failure");
                                            return Ok(parsed);
                                        },
                                        Err(extracted_err) => {
                                            warn!(
                                                error = %extracted_err,
                                                extracted_preview = &extracted_json[..extracted_json.len().min(200)],
                                                "Advanced extraction also failed to parse"
                                            );
                                        }
                                    }
                                } else {
                                    warn!("Advanced extraction could not find valid JSON in raw response");
                                }
                            }
                            
                            // If we're at max retries, return the error
                            let max_retries = config.max_retries.get("json_parse_error")
                                .unwrap_or(&config.default_max_retries);
                            
                            if attempt >= *max_retries {
                                error!(
                                    error = %json_err,
                                    attempt = attempt + 1,
                                    max_retries = max_retries,
                                    "Max retries exceeded for JSON parsing"
                                );
                                return Err(QueryResolverError::JsonDeserialization(json_err, response));
                            }
                            
                            // Otherwise, retry with context about the JSON parsing failure
                            warn!(
                                attempt = attempt + 1,
                                max_retries = max_retries,
                                "Retrying due to JSON parsing failure"
                            );
                            context = format!("JSON parsing failed: {}. Response was: {}", 
                                             json_err, 
                                             &response[..response.len().min(500)]);
                            attempt += 1;
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
                        AIError::DeepSeek(deepseek_err) => match deepseek_err {
                            crate::error::DeepSeekError::RateLimit => "rate_limit",
                            crate::error::DeepSeekError::Http(_) => "http_error",
                            crate::error::DeepSeekError::Api(_) => "api_error",
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
    
    /// Generate a JSON schema for the return type and append it to the prompt
    fn augment_prompt_with_schema<T>(&self, prompt: String) -> String
    where
        T: JsonSchema,
    {
        let schema = schema_for!(T);
        let schema_json = serde_json::to_string_pretty(&schema)
            .unwrap_or_else(|_| "{}".to_string());
        
        debug!(schema_len = schema_json.len(), "Generated JSON schema for return type");
        
        format!(
            r#"{prompt}

Please respond with JSON that matches this exact schema:

```json
{schema_json}
```

Your response must be valid JSON that can be parsed into this structure. Include all required fields and follow the specified types."#
        )
    }
    
    /// Ask with automatic schema-aware prompt augmentation
    #[instrument(skip(self, prompt, config), fields(prompt_len = prompt.len()))]
    async fn ask_with_schema<T>(&self, prompt: String, config: &RetryConfig) -> Result<T, QueryResolverError>
    where
        T: DeserializeOwned + JsonSchema + Send,
    {
        info!("Starting schema-aware query");
        let augmented_prompt = self.augment_prompt_with_schema::<T>(prompt);
        debug!(augmented_prompt_len = augmented_prompt.len(), "Generated schema-augmented prompt");
        self.ask_with_retry(augmented_prompt, config).await
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
        T: DeserializeOwned + Send,
    {
        info!(prompt_len = prompt.len(), "Starting query");
        let result = self.client.ask_with_retry(prompt, &self.config).await;
        match &result {
            Ok(_) => info!("Query completed successfully"),
            Err(e) => error!(error = %e, "Query failed"),
        }
        result
    }
    
    /// Query with automatic schema-aware prompt augmentation
    #[instrument(skip(self, prompt), fields(prompt_len = prompt.len()))]
    pub async fn query_with_schema<T>(&self, prompt: String) -> Result<T, QueryResolverError>
    where
        T: DeserializeOwned + JsonSchema + Send,
    {
        info!(prompt_len = prompt.len(), "Starting schema-aware query");
        let result = self.client.ask_with_schema(prompt, &self.config).await;
        match &result {
            Ok(_) => info!("Schema-aware query completed successfully"),
            Err(e) => error!(error = %e, "Schema-aware query failed"),
        }
        result
    }
}

/// Mock client for testing that returns empty responses
#[derive(Debug, Clone)]
pub struct MockVoid;

#[async_trait]
impl LowLevelClient for MockVoid {
    async fn ask_raw(&self, _prompt: String) -> Result<String, AIError> {
        Ok("{}".to_string())
    }
}