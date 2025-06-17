use client_implementations::benchmark::{
    BenchmarkRunner, ClaudeFactory, DeepSeekFactory, print_benchmark_summary,
    ClientFactory
};
use client_implementations::client::{QueryResolver, RetryConfig};
use serde::Deserialize;
use schemars::JsonSchema;

/// Additional test structure for language understanding
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(title = "Language Analysis", description = "Analysis of natural language text")]
pub struct LanguageAnalysis {
    /// The detected language of the text
    #[schemars(description = "ISO language code or language name")]
    pub language: String,
    /// Sentiment score from -1.0 to 1.0
    #[schemars(range(min = -1.0, max = 1.0), description = "Sentiment polarity score")]
    pub sentiment: f64,
    /// Key topics identified in the text
    #[schemars(description = "Main themes or subjects discussed")]
    pub topics: Vec<String>,
    /// Text complexity level
    pub complexity: TextComplexity,
}

/// Text complexity levels
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(description = "Complexity classification for text")]
pub enum TextComplexity {
    /// Simple, easy to understand
    #[schemars(description = "Basic vocabulary and simple sentences")]
    Simple,
    /// Moderate complexity
    #[schemars(description = "Some technical terms or complex ideas")]
    Moderate,
    /// Complex, advanced concepts
    #[schemars(description = "Technical jargon or advanced concepts")]
    Complex,
}

/// Programming challenge test structure
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(title = "Code Challenge", description = "Solution to a programming challenge")]
pub struct CodeChallenge {
    /// The programming language used
    #[schemars(description = "Language used for the solution")]
    pub language: String,
    /// The actual code solution
    #[schemars(description = "Working code that solves the problem")]
    pub solution: String,
    /// Time complexity analysis
    #[schemars(description = "Big O notation for time complexity")]
    pub time_complexity: String,
    /// Space complexity analysis
    #[schemars(description = "Big O notation for space complexity")]
    pub space_complexity: String,
    /// Whether the solution is optimal
    #[schemars(description = "True if this is an optimal solution")]
    pub is_optimal: bool,
}

#[tokio::test]
async fn test_claude_benchmark_suite() {
    let _ = tracing_subscriber::fmt::try_init();
    
    let factory = ClaudeFactory;
    
    if let Some(summary) = BenchmarkRunner::run_all_tests(&factory).await {
        print_benchmark_summary(&summary);
        
        // Ensure we have some basic success criteria
        assert!(summary.results.len() >= 5, "Should run at least 5 tests");
        assert!(summary.success_rate() > 0.0, "Should have some successful tests");
    } else {
        println!("Skipping Claude benchmark - no API key available");
    }
}

#[tokio::test]
async fn test_deepseek_benchmark_suite() {
    let _ = tracing_subscriber::fmt::try_init();
    
    let factory = DeepSeekFactory;
    
    if let Some(summary) = BenchmarkRunner::run_all_tests(&factory).await {
        print_benchmark_summary(&summary);
        
        // Ensure we have some basic success criteria
        assert!(summary.results.len() >= 5, "Should run at least 5 tests");
        assert!(summary.success_rate() > 0.0, "Should have some successful tests");
    } else {
        println!("Skipping DeepSeek benchmark - no API key available");
    }
}

#[tokio::test]
async fn test_language_analysis_benchmark() {
    let _ = tracing_subscriber::fmt::try_init();
    
    // Test with Claude if available
    let claude_factory = ClaudeFactory;
    if !claude_factory.should_skip() {
        if let Some(client) = claude_factory.create_client().await {
            let resolver = QueryResolver::new(client, RetryConfig::default());
            
            let result = BenchmarkRunner::run_test(
                "language_analysis_test",
                async move {
                    let analysis: LanguageAnalysis = resolver.query_with_schema(
                        "Analyze this text: 'The weather today is absolutely fantastic! I love sunny days like this.' Detect the language, sentiment, topics, and complexity.".to_string()
                    ).await?;
                    
                    // Validate schema compliance
                    if !(analysis.sentiment >= -1.0 && analysis.sentiment <= 1.0) {
                        return Err(client_implementations::error::QueryResolverError::JsonDeserialization(
                            serde_json::from_str::<()>("").unwrap_err(),
                            format!("sentiment out of range: {}", analysis.sentiment)
                        ));
                    }
                    
                    if analysis.language.is_empty() {
                        return Err(client_implementations::error::QueryResolverError::JsonDeserialization(
                            serde_json::from_str::<()>("").unwrap_err(),
                            "language should not be empty".to_string()
                        ));
                    }
                    
                    Ok(analysis)
                }
            ).await;
            
            println!("Claude Language Analysis Result: {:?}", result);
            assert!(result.passed, "Language analysis test should pass");
        }
    }
    
    // Test with DeepSeek if available
    let deepseek_factory = DeepSeekFactory;
    if !deepseek_factory.should_skip() {
        if let Some(client) = deepseek_factory.create_client().await {
            let resolver = QueryResolver::new(client, RetryConfig::default());
            
            let result = BenchmarkRunner::run_test(
                "deepseek_language_analysis_test",
                async move {
                    let analysis: LanguageAnalysis = resolver.query_with_schema(
                        "Analyze this text: 'The weather today is absolutely fantastic! I love sunny days like this.' Detect the language, sentiment, topics, and complexity.".to_string()
                    ).await?;
                    
                    // Validate schema compliance
                    if !(analysis.sentiment >= -1.0 && analysis.sentiment <= 1.0) {
                        return Err(client_implementations::error::QueryResolverError::JsonDeserialization(
                            serde_json::from_str::<()>("").unwrap_err(),
                            format!("sentiment out of range: {}", analysis.sentiment)
                        ));
                    }
                    
                    Ok(analysis)
                }
            ).await;
            
            println!("DeepSeek Language Analysis Result: {:?}", result);
            assert!(result.passed, "DeepSeek language analysis test should pass");
        }
    }
}

#[tokio::test]
async fn test_programming_challenge_benchmark() {
    let _ = tracing_subscriber::fmt::try_init();
    
    // Test with Claude if available
    let claude_factory = ClaudeFactory;
    if !claude_factory.should_skip() {
        if let Some(client) = claude_factory.create_client().await {
            let resolver = QueryResolver::new(client, RetryConfig::default());
            
            let result = BenchmarkRunner::run_test(
                "programming_challenge_test",
                async move {
                    let challenge: CodeChallenge = resolver.query_with_schema(
                        "Solve this programming challenge: Write a function to find the maximum element in an array. Provide the solution in Python with complexity analysis.".to_string()
                    ).await?;
                    
                    // Validate schema compliance
                    if challenge.language.is_empty() {
                        return Err(client_implementations::error::QueryResolverError::JsonDeserialization(
                            serde_json::from_str::<()>("").unwrap_err(),
                            "language should not be empty".to_string()
                        ));
                    }
                    
                    if challenge.solution.is_empty() {
                        return Err(client_implementations::error::QueryResolverError::JsonDeserialization(
                            serde_json::from_str::<()>("").unwrap_err(),
                            "solution should not be empty".to_string()
                        ));
                    }
                    
                    if challenge.time_complexity.is_empty() {
                        return Err(client_implementations::error::QueryResolverError::JsonDeserialization(
                            serde_json::from_str::<()>("").unwrap_err(),
                            "time_complexity should not be empty".to_string()
                        ));
                    }
                    
                    Ok(challenge)
                }
            ).await;
            
            println!("Claude Programming Challenge Result: {:?}", result);
            assert!(result.passed, "Programming challenge test should pass");
        }
    }
}

#[tokio::test]
async fn test_cross_client_comparison() {
    let _ = tracing_subscriber::fmt::try_init();
    
    let claude_factory = ClaudeFactory;
    let deepseek_factory = DeepSeekFactory;
    
    let mut results = Vec::new();
    
    // Collect results from both clients
    if let Some(claude_summary) = BenchmarkRunner::run_all_tests(&claude_factory).await {
        results.push(claude_summary);
    }
    
    if let Some(deepseek_summary) = BenchmarkRunner::run_all_tests(&deepseek_factory).await {
        results.push(deepseek_summary);
    }
    
    if results.len() >= 2 {
        println!("\n=== Cross-Client Comparison ===");
        
        for summary in &results {
            println!("\n{} Summary:", summary.client_name);
            println!("  Success Rate: {:.1}%", summary.success_rate() * 100.0);
            println!("  Average Duration: {:?}", summary.average_duration());
            println!("  Total Tests: {}", summary.results.len());
        }
        
        // Find the fastest and most reliable client
        let fastest = results.iter().min_by_key(|s| s.average_duration()).unwrap();
        let most_reliable = results.iter().max_by(|a, b| 
            a.success_rate().partial_cmp(&b.success_rate()).unwrap_or(std::cmp::Ordering::Equal)
        ).unwrap();
        
        println!("\nFastest Client: {} ({:?} avg)", fastest.client_name, fastest.average_duration());
        println!("Most Reliable Client: {} ({:.1}% success rate)", most_reliable.client_name, most_reliable.success_rate() * 100.0);
    } else {
        println!("Not enough clients available for comparison");
    }
}