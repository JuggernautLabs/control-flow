use client_implementations::deepseek::DeepSeekClient;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
struct TodoAppRequirements {
    /// Core functionality the app must provide
    #[schemars(
        description = "List of specific features the todo app needs (e.g., 'add tasks', 'mark complete', 'list tasks')"
    )]
    core_features: Vec<String>,

    /// How users will interact with the app
    #[schemars(description = "Interface type: 'command_line', 'web_api', 'gui', etc.")]
    interface_type: String,

    /// How data should be stored
    #[schemars(description = "Storage method: 'memory', 'file', 'database', etc.")]
    storage_method: String,

    /// Confidence that these requirements are complete
    #[schemars(
        range(min = 0.0, max = 1.0),
        description = "How confident are you these requirements capture the full scope? 0.0 = very incomplete, 1.0 = comprehensive"
    )]
    completeness_confidence: f64,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct PythonApplication {
    /// Complete Python code for the application
    #[schemars(
        description = "Complete, runnable Python code including all imports, functions, classes, and a main section"
    )]
    code: String,

    /// Test code that validates the application works
    #[schemars(
        description = "Complete pytest-compatible test code that thoroughly tests the application"
    )]
    tests: String,

    /// Confidence that the code implements the requirements correctly
    #[schemars(
        range(min = 0.0, max = 1.0),
        description = "How confident are you this code correctly implements all requirements?"
    )]
    implementation_confidence: f64,

    /// Instructions for running the application
    #[schemars(description = "Clear instructions for how to run this application and its tests")]
    usage_instructions: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SemanticCorrespondence {
    /// Whether the code matches the requirements
    #[schemars(
        description = "Does the implemented code actually fulfill the stated requirements?"
    )]
    requirements_fulfilled: bool,

    /// Whether the tests adequately verify the requirements
    #[schemars(description = "Do the tests actually verify that the requirements are met?")]
    tests_adequate: bool,

    /// Specific issues found, if any
    #[schemars(
        description = "List any gaps between requirements and implementation, or between requirements and tests"
    )]
    identified_gaps: Vec<String>,

    /// Overall correspondence score
    #[schemars(
        range(min = 0.0, max = 1.0),
        description = "Overall alignment between intent, requirements, implementation, and tests"
    )]
    correspondence_score: f64,
}

use client_implementations::claude::ClaudeClient;
use client_implementations::client::{QueryResolver, RetryConfig};

async fn build_todo_app_v1(description: &str) -> Result<VerifiedApp, BuildError> {
    let client = DeepSeekClient::new()?;
    let resolver = QueryResolver::new(client, RetryConfig::default());

    println!("🎯 Goal: {}", description);

    // Step 1: Extract requirements from the description
    let requirements_prompt = format!(
        "Analyze this request and define specific, testable requirements: '{}'
        
        Focus on:
        - What specific actions users need to perform
        - How they will interact with the app
        - How data should be handled
        - What constitutes 'working correctly'",
        description
    );

    let requirements: TodoAppRequirements = resolver.query_with_schema(requirements_prompt).await?;

    println!("📋 Requirements extracted:");
    println!("  Features: {:?}", requirements.core_features);
    println!("  Interface: {}", requirements.interface_type);
    println!("  Storage: {}", requirements.storage_method);
    println!("  Confidence: {:.2}", requirements.completeness_confidence);

    // Step 2: Generate the Python application
    let implementation_prompt = format!(
        "Create a complete single-file Python application that implements these requirements:
        
        Requirements: {:?}
        
        Generate:
        1. Complete, runnable Python code with proper imports
        2. Comprehensive pytest tests that verify all functionality
        3. Clear usage instructions
        
        The code should be production-ready and handle edge cases appropriately.",
        requirements
    );

    let app: PythonApplication = resolver.query_with_schema(implementation_prompt).await?;

    println!(
        "💻 Application generated ({} chars code, {} chars tests)",
        app.code.len(),
        app.tests.len()
    );
    println!(
        "  Implementation confidence: {:.2}",
        app.implementation_confidence
    );

    // Step 3: Verify semantic correspondence
    let verification_prompt = format!(
        "Analyze whether this implementation matches the requirements:
        
        Original Requirements: {:?}
        
        Generated Code:
        {}
        
        Generated Tests:
        {}
        
        Check:
        1. Does the code implement all required features?
        2. Do the tests verify all requirements are met?
        3. Are there any gaps or misalignments?",
        requirements, app.code, app.tests
    );

    let correspondence: SemanticCorrespondence =
        resolver.query_with_schema(verification_prompt).await?;

    println!("🔍 Semantic verification:");
    println!(
        "  Requirements fulfilled: {}",
        correspondence.requirements_fulfilled
    );
    println!("  Tests adequate: {}", correspondence.tests_adequate);
    println!(
        "  Correspondence score: {:.2}",
        correspondence.correspondence_score
    );

    if !correspondence.identified_gaps.is_empty() {
        println!("  Gaps identified: {:?}", correspondence.identified_gaps);
    }

    Ok(VerifiedApp {
        requirements,
        application: app,
        correspondence,
    })
}

#[derive(Debug)]
struct VerifiedApp {
    requirements: TodoAppRequirements,
    application: PythonApplication,
    correspondence: SemanticCorrespondence,
}

#[derive(thiserror::Error, Debug)]
enum BuildError {
    #[error("Query resolver error: {0}")]
    QueryError(#[from] client_implementations::error::QueryResolverError),

    #[error("claude client error: {0}")]
    ClaudeError(#[from] client_implementations::error::AIError),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[tokio::main]
async fn main() -> Result<(), BuildError> {
    // Start with the simplest possible case
       let result = build_todo_app_v1("Build a simple command-line todo app").await?;

       println!("\n✅ App built successfully!");
       println!("Usage: {}", result.application.usage_instructions);

       // Write the files to disk so we can actually test them
       std::fs::write("todo_app.py", &result.application.code)?;
       std::fs::write("test_todo_app.py", &result.application.tests)?;

    println!("\nFiles written: todo_app.py, test_todo_app.py");

    // Step 4: Actually run the tests
    println!("\n🧪 Running tests...");


    let test_output = std::process::Command::new("pytest")
        .arg("test_todo_app.py")
        .arg("-v")
        .output()?;

    let test_stdout = String::from_utf8_lossy(&test_output.stdout);
    let test_stderr = String::from_utf8_lossy(&test_output.stderr);

    println!("Test stdout:\n{}", test_stdout);
    if !test_stderr.is_empty() {
        println!("Test stderr:\n{}", test_stderr);
    }

    if test_output.status.success() {
        println!("✅ Tests passed!");
    } else {
        println!("❌ Tests failed!");
    }

   
    // Step 6: Summarize what we learned
       println!("\n📊 Summary:");
       println!("  Requirements confidence: {:.2}", result.requirements.completeness_confidence);
       println!("  Implementation confidence: {:.2}", result.application.implementation_confidence);
       println!("  Correspondence score: {:.2}", result.correspondence.correspondence_score);
       println!("  Tests passed: {}", test_output.status.success());

       // Step 7: Reality check
       let actual_success = test_output.status.success();
       let predicted_success = result.correspondence.correspondence_score > 0.8;

       println!("  AI predicted success: {}", predicted_success);
       println!("  Actual success: {}", actual_success);
       println!("  Prediction accuracy: {}", predicted_success == actual_success);

       if !result.correspondence.identified_gaps.is_empty() {
           println!("  Identified gaps: {:?}", result.correspondence.identified_gaps);
       }

    Ok(())
}
