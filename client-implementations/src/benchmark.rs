use crate::client::{LowLevelClient, QueryResolver, RetryConfig};
use crate::error::QueryResolverError;
use async_trait::async_trait;
use serde::{Deserialize, de::DeserializeOwned};
use schemars::JsonSchema;
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Simple test structure for basic functionality
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(title = "Math Result", description = "Result of a mathematical calculation")]
pub struct MathResult {
    /// The calculated result
    #[schemars(description = "The numerical result of the calculation")]
    pub result: i32,
    /// Whether the calculation was correct
    #[schemars(description = "True if the calculation appears correct")]
    pub is_correct: bool,
}

/// More complex structure to test rich schema generation
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(title = "Code Analysis", description = "Analysis of code quality and issues")]
pub struct CodeAnalysis {
    /// Confidence score from 0.0 to 1.0
    #[schemars(range(min = 0.0, max = 1.0), description = "How confident the analysis is")]
    pub confidence: f64,
    /// Primary finding from the analysis
    #[schemars(description = "The main conclusion from analyzing the code")]
    pub finding: String,
    /// List of specific issues found
    #[schemars(description = "Detailed list of problems or observations")]
    pub issues: Vec<String>,
    /// Severity level of the overall finding
    pub severity: Severity,
}

/// Severity levels for findings
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(description = "Severity classification for findings")]
pub enum Severity {
    /// Low impact issue
    #[schemars(description = "Minor issue that can be addressed later")]
    Low,
    /// Medium impact issue  
    #[schemars(description = "Issue that should be addressed soon")]
    Medium,
    /// High impact issue
    #[schemars(description = "Critical issue requiring immediate attention")]
    High,
}

/// Results from running a benchmark test
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Name of the test that was run
    pub test_name: String,
    /// Whether the test passed
    pub passed: bool,
    /// Time taken to complete the test
    pub duration: Duration,
    /// Optional error message if test failed
    pub error: Option<String>,
    /// Additional metadata about the test result
    pub metadata: std::collections::HashMap<String, String>,
}

impl BenchmarkResult {
    pub fn success(test_name: String, duration: Duration) -> Self {
        Self {
            test_name,
            passed: true,
            duration,
            error: None,
            metadata: std::collections::HashMap::new(),
        }
    }
    
    pub fn failure(test_name: String, duration: Duration, error: String) -> Self {
        Self {
            test_name,
            passed: false,
            duration,
            error: Some(error),
            metadata: std::collections::HashMap::new(),
        }
    }
    
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Summary of all benchmark results
#[derive(Debug)]
pub struct BenchmarkSummary {
    /// Name of the client being benchmarked
    pub client_name: String,
    /// Individual test results
    pub results: Vec<BenchmarkResult>,
    /// Total time for all tests
    pub total_duration: Duration,
}

impl BenchmarkSummary {
    pub fn new(client_name: String) -> Self {
        Self {
            client_name,
            results: Vec::new(),
            total_duration: Duration::ZERO,
        }
    }
    
    pub fn add_result(&mut self, result: BenchmarkResult) {
        self.total_duration += result.duration;
        self.results.push(result);
    }
    
    pub fn passed_tests(&self) -> usize {
        self.results.iter().filter(|r| r.passed).count()
    }
    
    pub fn failed_tests(&self) -> usize {
        self.results.iter().filter(|r| !r.passed).count()
    }
    
    pub fn success_rate(&self) -> f64 {
        if self.results.is_empty() {
            0.0
        } else {
            self.passed_tests() as f64 / self.results.len() as f64
        }
    }
    
    pub fn average_duration(&self) -> Duration {
        if self.results.is_empty() {
            Duration::ZERO
        } else {
            self.total_duration / self.results.len() as u32
        }
    }
}

/// Trait for creating client implementations for benchmarking
#[async_trait]
pub trait ClientFactory: Send + Sync {
    type Client: LowLevelClient + Send + Sync + Clone;
    
    /// Create a new client instance for testing
    async fn create_client(&self) -> Option<Self::Client>;
    
    /// Name of this client implementation
    fn client_name(&self) -> &str;
    
    /// Whether this client should be skipped (e.g., missing API keys)
    fn should_skip(&self) -> bool;
}

/// Abstract benchmark runner that works with any client implementation
pub struct BenchmarkRunner;

impl BenchmarkRunner {
    /// Run a single benchmark test
    pub async fn run_test<T, F>(
        test_name: &str,
        test_fn: F,
    ) -> BenchmarkResult
    where
        T: DeserializeOwned + JsonSchema + Send,
        F: std::future::Future<Output = Result<T, QueryResolverError>> + Send,
    {
        info!(test_name = test_name, "Starting benchmark test");
        let start_time = Instant::now();
        
        match test_fn.await {
            Ok(_) => {
                let duration = start_time.elapsed();
                info!(test_name = test_name, duration_ms = duration.as_millis(), "Test passed");
                BenchmarkResult::success(test_name.to_string(), duration)
            }
            Err(e) => {
                let duration = start_time.elapsed();
                warn!(test_name = test_name, error = %e, duration_ms = duration.as_millis(), "Test failed");
                BenchmarkResult::failure(test_name.to_string(), duration, e.to_string())
            }
        }
    }
    
    /// Run all standard benchmark tests for a client
    pub async fn run_all_tests<F: ClientFactory>(
        factory: &F,
    ) -> Option<BenchmarkSummary> {
        if factory.should_skip() {
            info!(client = factory.client_name(), "Skipping benchmark - requirements not met");
            return None;
        }
        
        let client = factory.create_client().await?;
        let mut summary = BenchmarkSummary::new(factory.client_name().to_string());
        
        info!(client = factory.client_name(), "Starting benchmark suite");
        let suite_start = Instant::now();
        
        // Run all the standard tests sequentially
        let result = Self::basic_schema_test(client.clone()).await;
        summary.add_result(result);
        
        let result = Self::complex_schema_test(client.clone()).await;
        summary.add_result(result);
        
        let result = Self::schema_constraint_test(client.clone()).await;
        summary.add_result(result);
        
        let result = Self::retry_behavior_test(client.clone()).await;
        summary.add_result(result);
        
        let result = Self::schema_generation_accuracy_test(client.clone()).await;
        summary.add_result(result);
        
        let result = Self::performance_test(client.clone()).await;
        summary.add_result(result);
        
        let result = Self::error_handling_test(client).await;
        summary.add_result(result);
        
        let total_time = suite_start.elapsed();
        summary.total_duration = total_time;
        
        info!(
            client = factory.client_name(),
            total_duration_ms = total_time.as_millis(),
            passed = summary.passed_tests(),
            failed = summary.failed_tests(),
            success_rate = format!("{:.1}%", summary.success_rate() * 100.0),
            "Benchmark suite completed"
        );
        
        Some(summary)
    }
    
    /// Basic schema validation test
    async fn basic_schema_test<C: LowLevelClient + Send + Sync + Clone>(client: C) -> BenchmarkResult {
        
        let resolver = QueryResolver::new(client, RetryConfig::default());
        
        Self::run_test(
            "basic_schema_test",
            async move {
                let result: MathResult = resolver.query_with_schema(
                    "Calculate 15 + 27 and tell me if the result is correct".to_string()
                ).await?;
                
                // Validate schema compliance - schema should ensure this is boolean
                // No additional validation needed for boolean fields
                
                Ok(result)
            }
        ).await
    }
    
    /// Complex schema with enums test
    async fn complex_schema_test<C: LowLevelClient + Send + Sync + Clone>(client: C) -> BenchmarkResult {
        
        let resolver = QueryResolver::new(client, RetryConfig::default());
        
        Self::run_test(
            "complex_schema_test", 
            async move {
                let code_sample = r#"
                fn unsafe_function() {
                    let ptr = std::ptr::null_mut();
                    unsafe {
                        *ptr = 42; // This will segfault!
                    }
                }
                "#;
                
                let result: CodeAnalysis = resolver.query_with_schema(
                    format!("Analyze this Rust code for issues:\n\n{}", code_sample)
                ).await?;
                
                // Validate schema compliance
                if !(result.confidence >= 0.0 && result.confidence <= 1.0) {
                    return Err(QueryResolverError::JsonDeserialization(
                        serde_json::from_str::<()>("").unwrap_err(),
                        format!("confidence out of range: {}", result.confidence)
                    ));
                }
                
                if result.finding.is_empty() {
                    return Err(QueryResolverError::JsonDeserialization(
                        serde_json::from_str::<()>("").unwrap_err(),
                        "finding should not be empty".to_string()
                    ));
                }
                
                Ok(result)
            }
        ).await
    }
    
    /// Schema constraint validation test
    async fn schema_constraint_test<C: LowLevelClient + Send + Sync + Clone>(client: C) -> BenchmarkResult {
        
        let resolver = QueryResolver::new(client, RetryConfig::default());
        
        Self::run_test(
            "schema_constraint_test",
            async move {
                let result: CodeAnalysis = resolver.query_with_schema(
                    "Give a high-confidence analysis of this simple function: fn add(a: i32, b: i32) -> i32 { a + b }".to_string()
                ).await?;
                
                // Validate constraints
                if !(result.confidence >= 0.0 && result.confidence <= 1.0) {
                    return Err(QueryResolverError::JsonDeserialization(
                        serde_json::from_str::<()>("").unwrap_err(),
                        format!("confidence out of range: {}", result.confidence)
                    ));
                }
                
                if result.finding.is_empty() {
                    return Err(QueryResolverError::JsonDeserialization(
                        serde_json::from_str::<()>("").unwrap_err(),
                        "finding should not be empty".to_string()
                    ));
                }
                
                Ok(result)
            }
        ).await
    }
    
    /// Retry behavior test
    async fn retry_behavior_test<C: LowLevelClient + Send + Sync + Clone>(client: C) -> BenchmarkResult {
        
        let mut config = RetryConfig::default();
        config.max_retries.insert("json_parse_error".to_string(), 3);
        let resolver = QueryResolver::new(client, config);
        
        Self::run_test(
            "retry_behavior_test",
            async move {
                let result: MathResult = resolver.query_with_schema(
                    "Calculate the square root of 144. Be very verbose in your explanation but still return the JSON.".to_string()
                ).await?;
                
                Ok(result)
            }
        ).await
    }
    
    /// Schema generation accuracy test
    async fn schema_generation_accuracy_test<C: LowLevelClient + Send + Sync + Clone>(client: C) -> BenchmarkResult {
        
        let resolver = QueryResolver::new(client, RetryConfig::default());
        
        Self::run_test(
            "schema_generation_accuracy_test",
            async move {
                let result: MathResult = resolver.query_with_schema(
                    "What is 8 * 7? Return exactly what the schema asks for.".to_string()
                ).await?;
                
                Ok(result)
            }
        ).await
    }
    
    /// Performance test - measures response time
    async fn performance_test<C: LowLevelClient + Send + Sync + Clone>(client: C) -> BenchmarkResult {
        
        let resolver = QueryResolver::new(client, RetryConfig::default());
        
        let result = Self::run_test(
            "performance_test",
            async move {
                let result: MathResult = resolver.query_with_schema(
                    "Quick calculation: 5 + 3".to_string()
                ).await?;
                
                Ok(result)
            }
        ).await;
        
        // Add performance metadata
        let category = if result.duration < Duration::from_secs(5) {
            "fast".to_string()
        } else if result.duration < Duration::from_secs(15) {
            "moderate".to_string()
        } else {
            "slow".to_string()
        };
        
        result.with_metadata("response_time_category".to_string(), category)
    }
    
    /// Error handling test - tests with malformed prompt
    async fn error_handling_test<C: LowLevelClient + Send + Sync + Clone>(client: C) -> BenchmarkResult {
        
        let resolver = QueryResolver::new(client, RetryConfig::default());
        
        Self::run_test(
            "error_handling_test",
            async move {
                // This should still work despite the confusing prompt
                let result: MathResult = resolver.query_with_schema(
                    "This is a very confusing prompt that asks for multiple things but please just calculate 2+2 and follow the schema exactly.".to_string()
                ).await?;
                
                Ok(result)
            }
        ).await
    }
}

/// Claude client factory for benchmarking
pub struct ClaudeFactory;

#[async_trait]
impl ClientFactory for ClaudeFactory {
    type Client = crate::claude::ClaudeClient;
    
    async fn create_client(&self) -> Option<Self::Client> {
        crate::claude::ClaudeClient::new().ok()
    }
    
    fn client_name(&self) -> &str {
        "Claude"
    }
    
    fn should_skip(&self) -> bool {
        std::env::var("ANTHROPIC_API_KEY").is_err() && 
        std::fs::read_to_string(".env").map_or(true, |content| !content.contains("ANTHROPIC_API_KEY"))
    }
}

/// DeepSeek client factory for benchmarking
pub struct DeepSeekFactory;

#[async_trait]
impl ClientFactory for DeepSeekFactory {
    type Client = crate::deepseek::DeepSeekClient;
    
    async fn create_client(&self) -> Option<Self::Client> {
        crate::deepseek::DeepSeekClient::new().ok()
    }
    
    fn client_name(&self) -> &str {
        "DeepSeek"
    }
    
    fn should_skip(&self) -> bool {
        std::env::var("DEEPSEEK_API_KEY").is_err() && 
        std::fs::read_to_string(".env").map_or(true, |content| !content.contains("DEEPSEEK_API_KEY"))
    }
}

/// Print a formatted summary of benchmark results
pub fn print_benchmark_summary(summary: &BenchmarkSummary) {
    println!("\n=== {} Benchmark Results ===", summary.client_name);
    println!("Total tests: {}", summary.results.len());
    println!("Passed: {} ({})", summary.passed_tests(), format!("{:.1}%", summary.success_rate() * 100.0));
    println!("Failed: {}", summary.failed_tests());
    println!("Total time: {:?}", summary.total_duration);
    println!("Average time per test: {:?}", summary.average_duration());
    
    println!("\nTest Details:");
    for result in &summary.results {
        let status = if result.passed { "✅" } else { "❌" };
        let duration_ms = result.duration.as_millis();
        
        println!("  {} {} ({} ms)", status, result.test_name, duration_ms);
        
        if let Some(error) = &result.error {
            println!("     Error: {}", error);
        }
        
        for (key, value) in &result.metadata {
            println!("     {}: {}", key, value);
        }
    }
    
    println!();
}