//! Core types for SATS v2
//! 
//! This module defines the foundational types for the verification chain system,
//! work item generation, and execution verification.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Unique identifier for entities
pub type Id = uuid::Uuid;

/// Timestamp for tracking when entities were created
pub type Timestamp = chrono::DateTime<chrono::Utc>;

/// Confidence score between 0.0 and 1.0
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Confidence(f64);

impl Confidence {
    pub fn new(value: f64) -> Result<Self, InvalidConfidence> {
        if (0.0..=1.0).contains(&value) {
            Ok(Confidence(value))
        } else {
            Err(InvalidConfidence(value))
        }
    }
    
    pub fn value(&self) -> f64 {
        self.0
    }
}

#[derive(Error, Debug)]
#[error("Confidence must be between 0.0 and 1.0, got {0}")]
pub struct InvalidConfidence(f64);

/// Location where an artifact exists
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Location {
    File { path: String, line_range: Option<(u32, u32)> },
    Url(String),
    Commit { hash: String, file_path: Option<String> },
    Ticket { system: String, id: String },
    Memory { session_id: String, interaction_id: String },
}

impl Location {
    pub fn display(&self) -> String {
        match self {
            Location::File { path, line_range: Some((start, end)) } => {
                format!("{}:{}-{}", path, start, end)
            }
            Location::File { path, line_range: None } => path.clone(),
            Location::Url(url) => url.clone(),
            Location::Commit { hash, file_path: Some(path) } => {
                format!("{}:{}", &hash[..8], path)
            }
            Location::Commit { hash, file_path: None } => hash[..8].to_string(),
            Location::Ticket { system, id } => format!("{}:{}", system, id),
            Location::Memory { session_id, interaction_id } => {
                format!("session:{}:{}", &session_id[..8], interaction_id)
            }
        }
    }
}

/// Types of artifacts that can be analyzed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArtifactType {
    Code,
    Test,
    Documentation,
    Commit,
    Ticket,
    Comment,
    Specification,
    Discussion,
}

/// A piece of content that can be analyzed for claims
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Artifact {
    pub id: Id,
    pub artifact_type: ArtifactType,
    pub content: String,
    pub location: Location,
    pub created_at: Timestamp,
    pub author: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Types of claims about system behavior
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClaimType {
    Functional,
    Performance,
    Security,
    Behavior,
    Structure,
    Requirement,
    Testing,
}

/// A statement extracted from an artifact that makes a claim about the system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Claim {
    pub id: Id,
    pub artifact_id: Id,
    pub statement: String,
    pub claim_type: ClaimType,
    pub extraction_confidence: Confidence,
    pub source_excerpt: String,
    pub extracted_at: Timestamp,
    /// The verification chain tracking implementation status
    pub verification_chain: Option<VerificationChain>,
}

/// Status of a verification chain
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChainStatus {
    NotStarted,
    NeedsTests,
    TestsFailing,
    TestsInadequate,
    Verified,
}

/// Individual link in the verification chain
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChainLink {
    Requirements(Vec<Requirement>),
    Implementation(Implementation),
    Tests(TestSuite),
    Execution(ExecutionResult),
    SemanticVerification(SemanticResult),
}

/// Complete verification chain for a claim
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationChain {
    pub claim_id: Id,
    pub status: ChainStatus,
    pub links: Vec<ChainLink>,
    pub created_at: Timestamp,
    pub last_verified_at: Option<Timestamp>,
    pub missing_links: Vec<WorkItemType>,
}

impl VerificationChain {
    pub fn get_requirements(&self) -> Option<&Vec<Requirement>> {
        self.links.iter().find_map(|link| match link {
            ChainLink::Requirements(reqs) => Some(reqs),
            _ => None,
        })
    }

    pub fn get_implementation(&self) -> Option<&Implementation> {
        self.links.iter().find_map(|link| match link {
            ChainLink::Implementation(impl_) => Some(impl_),
            _ => None,
        })
    }

    pub fn get_tests(&self) -> Option<&TestSuite> {
        self.links.iter().find_map(|link| match link {
            ChainLink::Tests(tests) => Some(tests),
            _ => None,
        })
    }

    pub fn get_execution_result(&self) -> Option<&ExecutionResult> {
        self.links.iter().find_map(|link| match link {
            ChainLink::Execution(result) => Some(result),
            _ => None,
        })
    }

    pub fn get_semantic_verification(&self) -> Option<&SemanticResult> {
        self.links.iter().find_map(|link| match link {
            ChainLink::SemanticVerification(result) => Some(result),
            _ => None,
        })
    }
}

/// Types of requirements that can be extracted from claims
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RequirementType {
    Functional,
    NonFunctional,
    Interface,
    Behavior,
    Constraint,
}

/// A requirement extracted from a claim
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Requirement {
    pub id: Id,
    pub claim_id: Id,
    pub requirement_type: RequirementType,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub priority: u8, // 1-10 scale
    pub extracted_at: Timestamp,
}

/// Status of an implementation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImplementationStatus {
    NotFound,
    Partial,
    Complete,
    Broken,
}

/// Implementation details for a set of requirements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Implementation {
    pub id: Id,
    pub requirements: Vec<Id>, // Requirement IDs
    pub status: ImplementationStatus,
    pub location: Location,
    pub code_snippets: Vec<String>,
    pub detected_at: Timestamp,
    pub confidence: Confidence,
}

/// A test case that verifies behavior
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestCase {
    pub id: Id,
    pub name: String,
    pub description: String,
    pub test_code: String,
    pub location: Location,
    pub test_type: TestType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TestType {
    Unit,
    Integration,
    EndToEnd,
    Performance,
    Security,
}

/// A collection of tests for an implementation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestSuite {
    pub id: Id,
    pub implementation_id: Id,
    pub test_cases: Vec<TestCase>,
    pub framework: String,
    pub total_tests: usize,
    pub detected_at: Timestamp,
}

/// Result of a single test execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestResult {
    pub test_case_id: Id,
    pub passed: bool,
    pub output: String,
    pub error_message: Option<String>,
    pub execution_time: std::time::Duration,
    pub coverage: Option<f64>,
}

/// Status of test execution
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExecutionStatus {
    NotRun,
    Running,
    Passed,
    Failed,
    Error,
    Timeout,
}

/// Result of executing a test suite
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub test_suite_id: Id,
    pub status: ExecutionStatus,
    pub results: Vec<TestResult>,
    pub total_passed: usize,
    pub total_failed: usize,
    pub total_errors: usize,
    pub coverage: Option<f64>,
    pub executed_at: Timestamp,
    pub execution_time: std::time::Duration,
}

impl ExecutionResult {
    pub fn success_rate(&self) -> f64 {
        if self.results.is_empty() {
            0.0
        } else {
            self.total_passed as f64 / self.results.len() as f64
        }
    }
}

/// Gaps found in semantic verification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SemanticGap {
    TestNameMismatch { test_name: String, actual_behavior: String },
    MissingAssertion { claim_aspect: String },
    UncoveredEdgeCase { edge_case: String },
    IncorrectAssumption { assumption: String, reality: String },
}

/// Result of semantic verification of tests
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticResult {
    pub claim_id: Id,
    pub test_suite_id: Id,
    pub coverage_score: Confidence,
    pub gaps: Vec<SemanticGap>,
    pub verified_aspects: Vec<String>,
    pub suggestions: Vec<String>,
    pub analyzed_at: Timestamp,
}

/// Types of work items that can be generated
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkItemType {
    ImplementRequirements,
    CreateTests,
    FixImplementation,
    ImproveTests,
    Documentation,
    Performance,
    Security,
}

/// Status of a work item
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkItemStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
    Blocked,
}

/// Base work item trait - all work items implement this
pub trait WorkItemSpec {
    fn to_prompt(&self) -> String;
    fn estimated_effort(&self) -> u8; // 1-10 scale
    fn required_skills(&self) -> Vec<String>;
    fn is_suitable_for_ai(&self) -> bool;
}

/// A work item that needs to be completed
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkItem {
    pub id: Id,
    pub work_item_type: WorkItemType,
    pub claim_id: Id,
    pub title: String,
    pub description: String,
    pub status: WorkItemStatus,
    pub created_at: Timestamp,
    pub assignee: Option<String>,
    pub estimated_effort: u8,
    pub required_skills: Vec<String>,
    pub specification: serde_json::Value, // Type-specific spec as JSON
}

/// Complete verification result with evidence
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationResult {
    pub claim_id: Id,
    pub status: ChainStatus,
    pub work_items: Vec<WorkItem>,
    pub evidence: Option<VerificationEvidence>,
    pub verified_at: Timestamp,
}

/// Evidence that supports verification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationEvidence {
    pub implementation: Implementation,
    pub tests: TestSuite,
    pub execution: ExecutionResult,
    pub semantic_verification: SemanticResult,
    pub confidence: Confidence,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_creation() {
        assert!(Confidence::new(0.5).is_ok());
        assert!(Confidence::new(1.0).is_ok());
        assert!(Confidence::new(0.0).is_ok());
        assert!(Confidence::new(1.1).is_err());
        assert!(Confidence::new(-0.1).is_err());
    }

    #[test]
    fn test_verification_chain_accessors() {
        let claim_id = uuid::Uuid::new_v4();
        let mut chain = VerificationChain {
            claim_id,
            status: ChainStatus::NotStarted,
            links: vec![],
            created_at: chrono::Utc::now(),
            last_verified_at: None,
            missing_links: vec![],
        };

        // Test empty chain
        assert!(chain.get_requirements().is_none());
        assert!(chain.get_implementation().is_none());

        // Add requirements
        let reqs = vec![Requirement {
            id: uuid::Uuid::new_v4(),
            claim_id,
            requirement_type: RequirementType::Functional,
            description: "Test requirement".to_string(),
            acceptance_criteria: vec!["Should work".to_string()],
            priority: 5,
            extracted_at: chrono::Utc::now(),
        }];
        
        chain.links.push(ChainLink::Requirements(reqs.clone()));
        
        let retrieved_reqs = chain.get_requirements().unwrap();
        assert_eq!(retrieved_reqs.len(), 1);
        assert_eq!(retrieved_reqs[0].description, "Test requirement");
    }

    #[test] 
    fn test_execution_result_success_rate() {
        let result = ExecutionResult {
            test_suite_id: uuid::Uuid::new_v4(),
            status: ExecutionStatus::Passed,
            results: vec![], // Empty results
            total_passed: 0,
            total_failed: 0,
            total_errors: 0,
            coverage: None,
            executed_at: chrono::Utc::now(),
            execution_time: std::time::Duration::from_secs(1),
        };
        
        assert_eq!(result.success_rate(), 0.0);
        
        let mut result_with_tests = result.clone();
        result_with_tests.total_passed = 3;
        result_with_tests.results = vec![
            TestResult {
                test_case_id: uuid::Uuid::new_v4(),
                passed: true,
                output: "".to_string(),
                error_message: None,
                execution_time: std::time::Duration::from_millis(100),
                coverage: None,
            };
            4 // 4 test results total
        ];
        
        assert_eq!(result_with_tests.success_rate(), 0.75); // 3/4 passed
    }
}