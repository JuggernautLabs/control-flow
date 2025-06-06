//! Basic usage example for SATS v2
//! 
//! This example demonstrates how to use SATS v2 to verify a simple claim
//! and generate work items when verification chains have gaps.

use sats_v2::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("SATS v2 Basic Usage Example");
    println!("===========================");

    // Create a sample claim
    let claim = create_sample_claim();
    println!("Created claim: {}", claim.statement);

    // Set up verification engine
    let verification_engine = create_verification_engine();
    
    // Verify the claim
    println!("\nVerifying claim...");
    let verification_result = verification_engine.verify_claim(&claim).await?;
    
    // Display results
    display_verification_result(&verification_result);

    // If there are work items, demonstrate work item management
    if !verification_result.work_items.is_empty() {
        println!("\nManaging work items...");
        demonstrate_work_item_management(&verification_result.work_items).await?;
    }

    Ok(())
}

fn create_sample_claim() -> Claim {
    Claim {
        id: uuid::Uuid::new_v4(),
        artifact_id: uuid::Uuid::new_v4(),
        statement: "User authentication system validates passwords correctly".to_string(),
        claim_type: ClaimType::Security,
        extraction_confidence: Confidence::new(0.9).unwrap(),
        source_excerpt: "// TODO: Implement secure password validation".to_string(),
        extracted_at: chrono::Utc::now(),
        verification_chain: None,
    }
}

fn create_verification_engine() -> VerificationEngine {
    use sats_v2::verification::*;

    let config = VerificationConfig::default();
    
    // Create mock implementations for the traits
    let requirement_extractor: Box<dyn RequirementExtractor> = 
        Box::new(MockRequirementExtractor);
    let implementation_checker: Box<dyn ImplementationChecker> = 
        Box::new(MockImplementationChecker);
    let test_checker: Box<dyn TestChecker> = 
        Box::new(MockTestChecker);
    let semantic_verifier: Box<dyn SemanticVerifier> = 
        Box::new(MockSemanticVerifier);

    VerificationEngine::new(
        config,
        requirement_extractor,
        implementation_checker,
        test_checker,
        semantic_verifier,
    )
}

fn display_verification_result(result: &VerificationResult) {
    println!("Verification Status: {:?}", result.status);
    println!("Work Items Generated: {}", result.work_items.len());
    
    for (i, work_item) in result.work_items.iter().enumerate() {
        println!("  {}. {} ({:?})", i + 1, work_item.title, work_item.work_item_type);
        println!("     Effort: {}/10, Skills: {:?}", 
                 work_item.estimated_effort, work_item.required_skills);
    }

    if let Some(evidence) = &result.evidence {
        println!("Evidence confidence: {}", evidence.confidence.value());
    }
}

async fn demonstrate_work_item_management(work_items: &[WorkItem]) -> Result<(), Box<dyn std::error::Error>> {
    use sats_v2::work_items::*;

    // Create work item manager with smart assignment strategy
    let ai_agents = vec![AvailableAgent {
        name: "CodeGen AI".to_string(),
        capabilities: vec!["rust".to_string(), "security".to_string()],
        max_complexity: 8,
        available: true,
    }];

    let human_developers = vec![AvailableDeveloper {
        name: "Senior Developer".to_string(),
        email: "dev@example.com".to_string(),
        skills: vec!["rust".to_string(), "security".to_string()],
        availability: 0.7,
    }];

    let assignment_strategy = SmartAssignmentStrategy::new(ai_agents, human_developers);
    let mut work_item_manager = WorkItemManager::new(Box::new(assignment_strategy));

    // Assign each work item
    for work_item in work_items {
        match work_item_manager.assign_work_item(work_item).await {
            Ok(assignment) => {
                println!("Assigned work item '{}' to: {:?}", 
                         work_item.title, assignment.assignee);
            }
            Err(e) => {
                println!("Failed to assign work item '{}': {}", work_item.title, e);
            }
        }
    }

    println!("Active assignments: {}", work_item_manager.get_active_assignments().len());

    Ok(())
}

// Mock implementations for demonstration
struct MockRequirementExtractor;

#[async_trait::async_trait]
impl RequirementExtractor for MockRequirementExtractor {
    async fn extract_requirements(&self, claim: &Claim) -> Result<Vec<Requirement>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![Requirement {
            id: uuid::Uuid::new_v4(),
            claim_id: claim.id,
            requirement_type: RequirementType::Functional,
            description: "Password validation function must exist".to_string(),
            acceptance_criteria: vec![
                "Function accepts password string".to_string(),
                "Returns validation result".to_string(),
                "Checks minimum length".to_string(),
            ],
            priority: 8,
            extracted_at: chrono::Utc::now(),
        }])
    }
}

struct MockImplementationChecker;

#[async_trait::async_trait]
impl ImplementationChecker for MockImplementationChecker {
    async fn check_implementation(&self, _requirements: &[Requirement]) -> Result<Implementation, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Implementation {
            id: uuid::Uuid::new_v4(),
            requirements: vec![],
            status: ImplementationStatus::NotFound, // Trigger work item generation
            location: Location::File {
                path: "src/auth.rs".to_string(),
                line_range: None,
            },
            code_snippets: vec![],
            detected_at: chrono::Utc::now(),
            confidence: Confidence::new(0.0).unwrap(),
        })
    }
}

struct MockTestChecker;

#[async_trait::async_trait]
impl TestChecker for MockTestChecker {
    async fn check_tests(&self, _implementation: &Implementation) -> Result<TestSuite, Box<dyn std::error::Error + Send + Sync>> {
        Ok(TestSuite {
            id: uuid::Uuid::new_v4(),
            implementation_id: uuid::Uuid::new_v4(),
            test_cases: vec![], // No tests found
            framework: "cargo".to_string(),
            total_tests: 0,
            detected_at: chrono::Utc::now(),
        })
    }
}

struct MockSemanticVerifier;

#[async_trait::async_trait]
impl SemanticVerifier for MockSemanticVerifier {
    async fn verify_test_coverage(&self, claim: &Claim, test_suite: &TestSuite) -> Result<SemanticResult, Box<dyn std::error::Error + Send + Sync>> {
        Ok(SemanticResult {
            claim_id: claim.id,
            test_suite_id: test_suite.id,
            coverage_score: Confidence::new(0.0).unwrap(),
            gaps: vec![],
            verified_aspects: vec![],
            suggestions: vec!["Add comprehensive password validation tests".to_string()],
            analyzed_at: chrono::Utc::now(),
        })
    }
}