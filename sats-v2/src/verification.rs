//! Verification engine for SATS v2
//! 
//! This module implements the core verification chain logic that discovers
//! implementation gaps and generates concrete work items.

use crate::types::*;
use async_trait::async_trait;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VerificationError {
    #[error("Failed to extract requirements: {0}")]
    RequirementExtraction(String),
    #[error("Implementation check failed: {0}")]
    ImplementationCheck(String),
    #[error("Test detection failed: {0}")]
    TestDetection(String),
    #[error("Execution failed: {0}")]
    Execution(String),
    #[error("Semantic verification failed: {0}")]
    SemanticVerification(String),
    #[error("LLM communication error: {0}")]
    LlmError(String),
}

/// Configuration for the verification engine
#[derive(Debug, Clone)]
pub struct VerificationConfig {
    pub min_implementation_confidence: Confidence,
    pub min_test_coverage: Confidence,
    pub min_semantic_coverage: Confidence,
    pub max_execution_timeout: std::time::Duration,
    pub llm_endpoint: Option<String>,
    pub enable_ai_generation: bool,
    pub sandbox_config: SandboxConfig,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            min_implementation_confidence: Confidence::new(0.7).unwrap(),
            min_test_coverage: Confidence::new(0.8).unwrap(), 
            min_semantic_coverage: Confidence::new(0.8).unwrap(),
            max_execution_timeout: std::time::Duration::from_secs(300),
            llm_endpoint: None,
            enable_ai_generation: false,
            sandbox_config: SandboxConfig::default(),
        }
    }
}

/// Configuration for execution sandbox
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub max_memory_mb: u64,
    pub max_cpu_time_seconds: u64,
    pub allowed_network: bool,
    pub allowed_filesystem_paths: Vec<String>,
    pub environment_variables: HashMap<String, String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 512,
            max_cpu_time_seconds: 30,
            allowed_network: false,
            allowed_filesystem_paths: vec!["/tmp".to_string()],
            environment_variables: HashMap::new(),
        }
    }
}

/// Main verification engine that orchestrates the verification chain
pub struct VerificationEngine {
    config: VerificationConfig,
    requirement_extractor: Box<dyn RequirementExtractor>,
    implementation_checker: Box<dyn ImplementationChecker>,
    test_checker: Box<dyn TestChecker>,
    semantic_verifier: Box<dyn SemanticVerifier>,
}

impl VerificationEngine {
    pub fn new(
        config: VerificationConfig,
        requirement_extractor: Box<dyn RequirementExtractor>,
        implementation_checker: Box<dyn ImplementationChecker>,
        test_checker: Box<dyn TestChecker>,
        semantic_verifier: Box<dyn SemanticVerifier>,
    ) -> Self {
        Self {
            config,
            requirement_extractor,
            implementation_checker,
            test_checker,
            semantic_verifier,
        }
    }

    /// Verify a claim and return the complete verification result
    pub async fn verify_claim(&self, claim: &Claim) -> Result<VerificationResult, VerificationError> {
        let mut work_items: Vec<WorkItem> = Vec::new();
        let mut chain_links = Vec::new();

        // Step 1: Extract requirements
        let requirements = self.requirement_extractor
            .extract_requirements(claim)
            .await
            .map_err(|e| VerificationError::RequirementExtraction(e.to_string()))?;
        
        chain_links.push(ChainLink::Requirements(requirements.clone()));

        // Step 2: Check implementation
        let impl_check = self.implementation_checker
            .check_implementation(&requirements)
            .await
            .map_err(|e| VerificationError::ImplementationCheck(e.to_string()))?;

        if impl_check.status == ImplementationStatus::NotFound {
            return Ok(VerificationResult {
                claim_id: claim.id,
                status: ChainStatus::NotStarted,
                work_items: vec![self.create_implementation_work_item(claim, &requirements).await?],
                evidence: None,
                verified_at: chrono::Utc::now(),
            });
        }

        chain_links.push(ChainLink::Implementation(impl_check.clone()));

        // Step 3: Check tests
        let test_check = self.test_checker
            .check_tests(&impl_check)
            .await
            .map_err(|e| VerificationError::TestDetection(e.to_string()))?;

        if test_check.test_cases.is_empty() {
            return Ok(VerificationResult {
                claim_id: claim.id,
                status: ChainStatus::NeedsTests,
                work_items: vec![self.create_test_creation_work_item(claim, &impl_check).await?],
                evidence: None,
                verified_at: chrono::Utc::now(),
            });
        }

        chain_links.push(ChainLink::Tests(test_check.clone()));

        // Step 4: Execute tests (delegated to execution engine)
        let execution_result = self.execute_tests(&test_check).await?;
        
        if execution_result.status != ExecutionStatus::Passed {
            return Ok(VerificationResult {
                claim_id: claim.id,
                status: ChainStatus::TestsFailing,
                work_items: vec![self.create_fix_implementation_work_item(claim, &execution_result).await?],
                evidence: None,
                verified_at: chrono::Utc::now(),
            });
        }

        chain_links.push(ChainLink::Execution(execution_result.clone()));

        // Step 5: Semantic verification
        let semantic_result = self.semantic_verifier
            .verify_test_coverage(claim, &test_check)
            .await
            .map_err(|e| VerificationError::SemanticVerification(e.to_string()))?;

        if semantic_result.coverage_score.value() < self.config.min_semantic_coverage.value() {
            return Ok(VerificationResult {
                claim_id: claim.id,
                status: ChainStatus::TestsInadequate,
                work_items: vec![self.create_improve_tests_work_item(claim, &semantic_result).await?],
                evidence: None,
                verified_at: chrono::Utc::now(),
            });
        }

        chain_links.push(ChainLink::SemanticVerification(semantic_result.clone()));

        // All verification steps passed
        let evidence = VerificationEvidence {
            implementation: impl_check,
            tests: test_check,
            execution: execution_result,
            semantic_verification: semantic_result,
            confidence: Confidence::new(0.9).unwrap(), // High confidence when all steps pass
        };

        Ok(VerificationResult {
            claim_id: claim.id,
            status: ChainStatus::Verified,
            work_items: vec![],
            evidence: Some(evidence),
            verified_at: chrono::Utc::now(),
        })
    }

    async fn execute_tests(&self, test_suite: &TestSuite) -> Result<ExecutionResult, VerificationError> {
        // This would delegate to the ExecutionEngine
        // For now, return a placeholder
        Ok(ExecutionResult {
            test_suite_id: test_suite.id,
            status: ExecutionStatus::Passed,
            results: vec![],
            total_passed: test_suite.test_cases.len(),
            total_failed: 0,
            total_errors: 0,
            coverage: Some(0.85),
            executed_at: chrono::Utc::now(),
            execution_time: std::time::Duration::from_secs(5),
        })
    }

    async fn create_implementation_work_item(&self, claim: &Claim, requirements: &[Requirement]) -> Result<WorkItem, VerificationError> {
        Ok(WorkItem {
            id: uuid::Uuid::new_v4(),
            work_item_type: WorkItemType::ImplementRequirements,
            claim_id: claim.id,
            title: format!("Implement: {}", claim.statement),
            description: format!("Implement the following requirements for claim: {}", claim.statement),
            status: WorkItemStatus::Pending,
            created_at: chrono::Utc::now(),
            assignee: None,
            estimated_effort: 6, // Medium effort by default
            required_skills: vec!["programming".to_string(), "architecture".to_string()],
            specification: serde_json::to_value(requirements).unwrap(),
        })
    }

    async fn create_test_creation_work_item(&self, claim: &Claim, implementation: &Implementation) -> Result<WorkItem, VerificationError> {
        Ok(WorkItem {
            id: uuid::Uuid::new_v4(),
            work_item_type: WorkItemType::CreateTests,
            claim_id: claim.id,
            title: format!("Create tests for: {}", claim.statement),
            description: format!("Create comprehensive tests for claim: {}", claim.statement),
            status: WorkItemStatus::Pending,
            created_at: chrono::Utc::now(),
            assignee: None,
            estimated_effort: 4, // Lower effort - tests are more predictable
            required_skills: vec!["testing".to_string(), "programming".to_string()],
            specification: serde_json::to_value(implementation).unwrap(),
        })
    }

    async fn create_fix_implementation_work_item(&self, claim: &Claim, execution_result: &ExecutionResult) -> Result<WorkItem, VerificationError> {
        Ok(WorkItem {
            id: uuid::Uuid::new_v4(),
            work_item_type: WorkItemType::FixImplementation,
            claim_id: claim.id,
            title: format!("Fix failing tests for: {}", claim.statement),
            description: format!("Fix implementation so tests pass for claim: {}", claim.statement),
            status: WorkItemStatus::Pending,
            created_at: chrono::Utc::now(),
            assignee: None,
            estimated_effort: 5,
            required_skills: vec!["debugging".to_string(), "programming".to_string()],
            specification: serde_json::to_value(execution_result).unwrap(),
        })
    }

    async fn create_improve_tests_work_item(&self, claim: &Claim, semantic_result: &SemanticResult) -> Result<WorkItem, VerificationError> {
        Ok(WorkItem {
            id: uuid::Uuid::new_v4(),
            work_item_type: WorkItemType::ImproveTests,
            claim_id: claim.id,
            title: format!("Improve test coverage for: {}", claim.statement),
            description: format!("Improve tests to better verify claim: {}", claim.statement),
            status: WorkItemStatus::Pending,
            created_at: chrono::Utc::now(),
            assignee: None,
            estimated_effort: 3,
            required_skills: vec!["testing".to_string()],
            specification: serde_json::to_value(semantic_result).unwrap(),
        })
    }
}

/// Trait for extracting requirements from claims
#[async_trait]
pub trait RequirementExtractor: Send + Sync {
    async fn extract_requirements(&self, claim: &Claim) -> Result<Vec<Requirement>, Box<dyn std::error::Error + Send + Sync>>;
}

/// Trait for checking if implementation exists for requirements  
#[async_trait]
pub trait ImplementationChecker: Send + Sync {
    async fn check_implementation(&self, requirements: &[Requirement]) -> Result<Implementation, Box<dyn std::error::Error + Send + Sync>>;
}

/// Trait for detecting tests for implementations
#[async_trait]
pub trait TestChecker: Send + Sync {
    async fn check_tests(&self, implementation: &Implementation) -> Result<TestSuite, Box<dyn std::error::Error + Send + Sync>>;
}

/// Trait for semantic verification of test coverage
#[async_trait]
pub trait SemanticVerifier: Send + Sync {
    async fn verify_test_coverage(&self, claim: &Claim, test_suite: &TestSuite) -> Result<SemanticResult, Box<dyn std::error::Error + Send + Sync>>;
}

/// Default LLM-based requirement extractor
pub struct LlmRequirementExtractor {
    llm_client: LlmClient,
}

impl LlmRequirementExtractor {
    pub fn new(llm_client: LlmClient) -> Self {
        Self { llm_client }
    }
}

#[async_trait]
impl RequirementExtractor for LlmRequirementExtractor {
    async fn extract_requirements(&self, claim: &Claim) -> Result<Vec<Requirement>, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            r#"Extract concrete technical requirements from this claim:

Claim: {}
Source: {}

Break this down into specific, testable requirements. For each requirement, provide:
1. A clear description of what needs to exist
2. Acceptance criteria that can be verified
3. Priority level (1-10)

Return as JSON array of requirements."#,
            claim.statement,
            claim.source_excerpt
        );

        let response = self.llm_client.generate(&prompt).await?;
        
        // Parse response and create Requirement objects
        // This is simplified - real implementation would have proper JSON parsing
        Ok(vec![Requirement {
            id: uuid::Uuid::new_v4(),
            claim_id: claim.id,
            requirement_type: RequirementType::Functional,
            description: format!("Extracted requirement for: {}", claim.statement),
            acceptance_criteria: vec!["Should be implemented".to_string()],
            priority: 5,
            extracted_at: chrono::Utc::now(),
        }])
    }
}

/// Placeholder LLM client - would be replaced with actual implementation
pub struct LlmClient {
    endpoint: Option<String>,
}

impl LlmClient {
    pub fn new(endpoint: Option<String>) -> Self {
        Self { endpoint }
    }

    pub async fn generate(&self, _prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Placeholder implementation
        Ok("Generated response".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_verification_config_default() {
        let config = VerificationConfig::default();
        assert_eq!(config.min_implementation_confidence.value(), 0.7);
        assert_eq!(config.min_test_coverage.value(), 0.8);
        assert!(!config.enable_ai_generation);
    }

    #[tokio::test]
    async fn test_llm_requirement_extractor() {
        let llm_client = LlmClient::new(None);
        let extractor = LlmRequirementExtractor::new(llm_client);
        
        let claim = Claim {
            id: uuid::Uuid::new_v4(),
            artifact_id: uuid::Uuid::new_v4(),
            statement: "Password reset functionality works".to_string(),
            claim_type: ClaimType::Functional,
            extraction_confidence: Confidence::new(0.9).unwrap(),
            source_excerpt: "// Implement password reset".to_string(),
            extracted_at: chrono::Utc::now(),
            verification_chain: None,
        };

        let requirements = extractor.extract_requirements(&claim).await.unwrap();
        assert!(!requirements.is_empty());
        assert_eq!(requirements[0].claim_id, claim.id);
    }
}