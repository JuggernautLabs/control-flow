//! Full workflow example: Discussion → Claims → Tests → Implementation → Verification
//! 
//! This example demonstrates the complete SATS v2 pipeline from a technical discussion
//! all the way through to verified implementation.

use sats_v2::*;
use sats_v2::semantic::LlmClient;
use sats_v2::workflow::DiscussionContext;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for detailed logging
    tracing_subscriber::fmt::init();

    println!("🚀 SATS v2 Full Workflow Example");
    println!("================================");
    println!();

    // Step 0: Create a technical discussion
    let discussion = create_sample_discussion();
    println!("📝 Created discussion: {}", discussion.title);
    println!("Context: {} - {}", discussion.context.project_name, discussion.context.domain);
    println!();

    // Step 1: Set up the workflow orchestrator
    let workflow_orchestrator = create_workflow_orchestrator().await?;
    println!("⚙️  Workflow orchestrator ready");
    println!();

    // Step 2: Execute the complete workflow
    println!("🔄 Starting full workflow execution...");
    println!();

    let workflow_result = workflow_orchestrator
        .execute_full_workflow(&discussion)
        .await?;

    // Step 3: Display comprehensive results
    display_workflow_results(&workflow_result);

    println!();
    println!("✅ Full workflow completed successfully!");
    
    Ok(())
}

fn create_sample_discussion() -> Discussion {
    Discussion {
        id: uuid::Uuid::new_v4(),
        title: "User Authentication System Requirements".to_string(),
        content: r#"
We need to implement a secure user authentication system for our web application.

Key Requirements:
1. Password validation must enforce strong security policies
2. System should reject weak passwords with clear error messages
3. All passwords must be hashed using bcrypt before storage
4. The system should handle edge cases like empty passwords gracefully
5. Performance requirement: validation should complete under 100ms
6. Security requirement: system must be resistant to common attacks

Technical Decisions:
- Use Rust for implementation
- bcrypt for password hashing with default cost
- Return structured validation results
- Comprehensive error handling

Expected Interface:
- validate_password(password: String) -> Result<ValidationResult, ValidationError>
- hash_password(password: &str) -> Result<String, BcryptError>

Edge Cases to Consider:
- Empty passwords
- Passwords with only numbers
- Passwords without special characters
- Extremely long passwords
- Malicious input attempts
        "#.to_string(),
        participants: vec![
            "alice@company.com".to_string(),
            "bob@company.com".to_string(),
            "security-team@company.com".to_string(),
        ],
        context: DiscussionContext {
            project_name: "SecureWebApp".to_string(),
            domain: "Web Security".to_string(),
            existing_codebase: vec![
                "src/auth/mod.rs".to_string(),
                "src/models/user.rs".to_string(),
            ],
            requirements: vec![
                "OWASP security compliance".to_string(),
                "Sub-100ms response time".to_string(),
                "Clear error messages".to_string(),
            ],
            constraints: vec![
                "Must use Rust".to_string(),
                "No external security libraries except bcrypt".to_string(),
                "Memory usage under 1MB".to_string(),
            ],
        },
        created_at: chrono::Utc::now(),
    }
}

async fn create_workflow_orchestrator() -> Result<WorkflowOrchestrator, Box<dyn std::error::Error>> {
    // Initialize LLM client
    let llm_client = LlmClient::default();

    // Create all workflow components
    let discussion_analyzer: Box<dyn DiscussionAnalyzer> = 
        Box::new(LlmDiscussionAnalyzer::new(llm_client.clone()));
    
    let claim_extractor: Box<dyn ClaimExtractor> = 
        Box::new(LlmClaimExtractor::new(llm_client.clone()));
    
    let test_spec_generator: Box<dyn TestSpecGenerator> = 
        Box::new(LlmTestSpecGenerator::new(llm_client.clone()));
    
    let test_semantic_verifier: Box<dyn TestSemanticVerifier> = 
        Box::new(LlmTestSemanticVerifier::new(llm_client.clone()));
    
    let compilation_verifier: Box<dyn CompilationVerifier> = 
        Box::new(RustCompilationVerifier::new().map_err(|e| format!("Failed to create compilation verifier: {}", e))?);
    
    let implementation_generator: Box<dyn workflow::ImplementationGenerator> = 
        Box::new(LlmTddImplementationGenerator::new(llm_client.clone()));
    
    let execution_verifier: Box<dyn ExecutionVerifier> = 
        Box::new(DefaultExecutionVerifier::new().map_err(|e| format!("Failed to create execution verifier: {}", e))?);

    Ok(WorkflowOrchestrator::new(
        discussion_analyzer,
        claim_extractor,
        test_spec_generator,
        test_semantic_verifier,
        compilation_verifier,
        implementation_generator,
        execution_verifier,
        llm_client,
    ))
}

fn display_workflow_results(result: &WorkflowResult) {
    println!("📊 WORKFLOW RESULTS");
    println!("==================");
    println!();

    // Discussion Analysis Results
    if let Some(analysis) = &result.discussion_analysis {
        println!("🔍 Discussion Analysis:");
        println!("  Key Decisions: {}", analysis.key_decisions.len());
        for decision in &analysis.key_decisions {
            println!("    • {} (Impact: {:?})", decision.description, decision.impact);
        }
        
        println!("  Technical Requirements: {}", analysis.technical_requirements.len());
        for req in &analysis.technical_requirements {
            println!("    • {} (Priority: {}/10)", req.description, req.priority);
        }
        
        println!("  Functional Specs: {}", analysis.functional_specifications.len());
        for spec in &analysis.functional_specifications {
            println!("    • {}: {}", spec.name, spec.description);
        }
        
        println!("  Analysis Confidence: {:.1}%", analysis.confidence.value() * 100.0);
        println!();
    }

    // Claims Extracted
    println!("📝 Claims Extracted: {}", result.claims.len());
    for (i, claim) in result.claims.iter().enumerate() {
        println!("  {}. {} ({:?})", i + 1, claim.statement, claim.claim_type);
        println!("     Confidence: {:.1}%", claim.extraction_confidence.value() * 100.0);
    }
    println!();

    // Claim Pipeline Results
    println!("🔄 Claim Pipeline Results:");
    for (i, claim_result) in result.claim_results.iter().enumerate() {
        println!("  Claim {}:", i + 1);
        
        // Test Specifications
        println!("    📋 Test Specifications: {}", claim_result.test_specs.len());
        for spec in &claim_result.test_specs {
            println!("      • {}: {}", spec.name, spec.description);
            if let Some(perf) = &spec.performance_criteria {
                println!("        ⏱️  Max time: {}ms", perf.max_execution_time_ms);
            }
        }

        // Generated Tests
        if let Some(tests) = &claim_result.generated_tests {
            println!("    🧪 Generated Tests: {}", tests.test_cases.len());
            println!("        Coverage Estimate: {:.1}%", tests.total_coverage_estimate * 100.0);
        }

        // Test Semantic Verification
        if let Some(semantic) = &claim_result.test_semantic_verification {
            println!("    🔬 Semantic Verification:");
            println!("        Spec Compliance: {:.1}%", semantic.spec_compliance * 100.0);
            if !semantic.missing_assertions.is_empty() {
                println!("        Missing Assertions: {}", semantic.missing_assertions.len());
            }
            if !semantic.misaligned_tests.is_empty() {
                println!("        Misaligned Tests: {}", semantic.misaligned_tests.len());
            }
        }

        // Test Compilation
        if let Some(compilation) = &claim_result.test_compilation {
            println!("    🔧 Test Compilation: {}", 
                if compilation.success { "✅ SUCCESS" } else { "❌ FAILED" });
            if !compilation.success {
                println!("        Errors: {}", compilation.errors.len());
            }
        }

        // Generated Implementation
        if let Some(impl_) = &claim_result.generated_implementation {
            println!("    💻 Generated Implementation:");
            println!("        File: {}", impl_.file_path);
            println!("        Dependencies: {:?}", impl_.dependencies);
            println!("        Test Compatibility: {:.1}%", impl_.test_compatibility * 100.0);
        }

        // Implementation Compilation
        if let Some(compilation) = &claim_result.implementation_compilation {
            println!("    🔧 Implementation Compilation: {}", 
                if compilation.success { "✅ SUCCESS" } else { "❌ FAILED" });
            println!("        Compilation Time: {}ms", compilation.compilation_time_ms);
        }

        // Execution Results
        if let Some(execution) = &claim_result.execution_result {
            println!("    🚀 Test Execution:");
            println!("        Status: {:?}", execution.status);
            println!("        Tests Passed: {}/{}", execution.total_passed, execution.results.len());
            println!("        Success Rate: {:.1}%", execution.success_rate() * 100.0);
            if let Some(coverage) = execution.coverage {
                println!("        Code Coverage: {:.1}%", coverage * 100.0);
            }
            println!("        Execution Time: {}ms", execution.execution_time.as_millis());
        }

        println!("    Overall Success: {}", if claim_result.success { "✅" } else { "❌" });
        println!();
    }

    // Integration Results
    if let Some(integration) = &result.integration_result {
        println!("🔗 Integration Results:");
        println!("  Success: {}", if integration.success { "✅" } else { "❌" });
        println!("  Integration Tests: {} passed, {} failed", 
            integration.integration_tests_passed, 
            integration.integration_tests_failed);
        println!("  Cross-claim Conflicts: {}", integration.cross_claim_conflicts.len());
        println!("  Overall Confidence: {:.1}%", integration.overall_confidence.value() * 100.0);
        println!();
    }

    // Summary
    let total_claims = result.claims.len();
    let successful_claims = result.claim_results.iter().filter(|r| r.success).count();
    let overall_success_rate = if total_claims > 0 {
        (successful_claims as f64) / (total_claims as f64) * 100.0
    } else {
        0.0
    };

    println!("📈 SUMMARY");
    println!("=========");
    println!("Total Claims Processed: {}", total_claims);
    println!("Successfully Verified: {}", successful_claims);
    println!("Overall Success Rate: {:.1}%", overall_success_rate);
    println!("Total Execution Time: {}ms", result.total_execution_time_ms);

    if overall_success_rate >= 80.0 {
        println!("🎉 Excellent! High confidence in implementation quality.");
    } else if overall_success_rate >= 60.0 {
        println!("⚠️  Good progress, but some claims need attention.");
    } else {
        println!("🔧 Significant work needed to meet all requirements.");
    }
}