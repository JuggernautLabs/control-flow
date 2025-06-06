//! SATS v2: Semantic Alignment Tracking System v2
//! 
//! Implementation gap discovery and work generation system that transforms
//! from passive alignment measurement to active work generation.
//!
//! Key innovation: Tracking implementation chains from claims to verified execution.

pub mod types;
pub mod verification;
pub mod execution;
pub mod work_items;
pub mod semantic;
pub mod generators;
pub mod workflow;
pub mod workflow_impl;
pub mod code_references;
pub mod ai_agents;
pub mod verification_claim_extractor;

// Re-export core types
pub use types::{
    Claim, ClaimType, Artifact, ArtifactType, Location,
    VerificationChain, ChainStatus, ChainLink,
    WorkItem, WorkItemType, WorkItemStatus, 
    Requirement, RequirementType,
    Implementation, ImplementationStatus,
    TestSuite, TestCase, TestResult,
    ExecutionResult, ExecutionStatus,
    SemanticResult, SemanticGap,
    VerificationResult, VerificationEvidence,
    Confidence, Timestamp, Id,
};

pub use verification::{
    VerificationEngine, VerificationConfig,
    RequirementExtractor, ImplementationChecker, 
    TestChecker, SemanticVerifier,
};

pub use execution::{
    ExecutionEngine, ExecutionEnvironment,
    ExecutionError,
};

pub use work_items::{
    WorkItemManager, WorkItemGenerator,
    ImplementationWorkItem, TestCreationWorkItem,
    FixImplementationWorkItem, ImproveTestsWorkItem,
    WorkItemAssignment, AssignmentStrategy,
};

pub use semantic::{
    SemanticAnalyzer, ClaimAnalyzer,
    TestCoverageAnalyzer, GapAnalyzer,
    RiskLevel,
};

pub use generators::{
    ImplementationGenerator, TestGenerator,
    SpecificationGenerator,
};

pub use workflow::{
    WorkflowOrchestrator, WorkflowResult, WorkflowError,
    Discussion, DiscussionAnalysis, TestSpec, CompilationResult,
    DiscussionAnalyzer, ClaimExtractor, TestSpecGenerator,
    TestSemanticVerifier, CompilationVerifier, ExecutionVerifier,
};

pub use workflow_impl::{
    LlmDiscussionAnalyzer, LlmClaimExtractor, LlmTestSpecGenerator,
    LlmTestSemanticVerifier, RustCompilationVerifier,
    LlmTddImplementationGenerator, DefaultExecutionVerifier,
};

pub use code_references::{
    CodeLocation, CodeReference, CodeMetadata, ExecutionInfo,
    TestReference, TestSpecification, ImplementationReference,
    ProgrammingLanguage, CodeType, TestType,
    TestInput, TestOutput, EdgeCase,
};

pub use ai_agents::{
    AgentOrchestrator, AiAgent, AgentTask, AgentTaskResult,
    AgentType, AgentTaskInput, AgentTaskOutput, AgentError,
    MockCompilationAgent, MockTestExecutionAgent,
    BuildConfiguration, TaskConstraints, TaskPriority,
};

pub use verification_claim_extractor::{
    VerificationClaimExtractor, ClaudeVerificationExtractor,
    VerificationExtractionResult, VerificationError,
};

/// Common result type for SATS v2 operations
pub type SatsResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Version of SATS v2
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}