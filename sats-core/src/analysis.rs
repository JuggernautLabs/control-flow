use crate::types::*;
use async_trait::async_trait;
use std::collections::HashMap;

/// Error types for narrative analysis operations
#[derive(thiserror::Error, Debug)]
pub enum AnalysisError {
    #[error("Failed to extract claims: {0}")]
    ClaimExtraction(String),
    #[error("Failed to discover relationships: {0}")]
    RelationshipDiscovery(String),
    #[error("Failed to check alignment: {0}")]
    AlignmentCheck(String),
    #[error("Invalid artifact type for analysis: {0:?}")]
    InvalidArtifactType(ArtifactType),
    #[error("LLM client error: {0}")]
    LlmClient(String),
    #[error("Configuration error: {0}")]
    Configuration(String),
}

/// Results from claim extraction with metadata
#[derive(Debug, Clone)]
pub struct ClaimExtractionResult {
    pub claims: Vec<Claim>,
    pub processing_time_ms: u64,
    pub model_used: String,
    pub extraction_metadata: HashMap<String, String>,
}

/// Results from relationship discovery
#[derive(Debug, Clone)]
pub struct RelationshipDiscoveryResult {
    pub relationships: Vec<Relationship>,
    pub processing_time_ms: u64,
    pub candidates_analyzed: usize,
    pub discovery_metadata: HashMap<String, String>,
}

/// Results from alignment checking
#[derive(Debug, Clone)]
pub struct AlignmentResult {
    pub alignments: Vec<Alignment>,
    pub processing_time_ms: u64,
    pub evidence_count: usize,
    pub alignment_metadata: HashMap<String, String>,
}

/// Trait for extracting claims from different types of artifacts
/// 
/// This is the core interface for "narrative analysis" - understanding
/// the human-readable claims and statements made in various artifacts.
#[async_trait]
pub trait ClaimExtractor: Send + Sync {
    /// Extract claims from a single artifact
    async fn extract_claims(
        &self,
        artifact: &Artifact,
        config: &AnalysisConfig,
    ) -> Result<ClaimExtractionResult, AnalysisError>;
    
    /// Check if this extractor can handle the given artifact type
    fn can_handle(&self, artifact_type: &ArtifactType) -> bool;
    
    /// Get the confidence threshold this extractor recommends for the artifact type
    fn recommended_confidence_threshold(&self, artifact_type: &ArtifactType) -> Confidence;
}

/// Trait for discovering semantic relationships between artifacts
#[async_trait]
pub trait RelationshipDiscoverer: Send + Sync {
    /// Find relationships between a new artifact and existing artifacts
    async fn discover_relationships(
        &self,
        target_artifact: &Artifact,
        candidate_artifacts: &[Artifact],
        config: &AnalysisConfig,
    ) -> Result<RelationshipDiscoveryResult, AnalysisError>;
    
    /// Find relationships between all pairs in a set of artifacts
    async fn discover_batch_relationships(
        &self,
        artifacts: &[Artifact],
        config: &AnalysisConfig,
    ) -> Result<RelationshipDiscoveryResult, AnalysisError>;
}

/// Trait for checking how well evidence supports claims
#[async_trait]
pub trait AlignmentChecker: Send + Sync {
    /// Check alignment between a single claim and potential evidence
    async fn check_alignment(
        &self,
        claim: &Claim,
        evidence_artifact: &Artifact,
        config: &AnalysisConfig,
    ) -> Result<Alignment, AnalysisError>;
    
    /// Check alignment for multiple claims against evidence artifacts
    async fn check_batch_alignment(
        &self,
        claims: &[Claim],
        evidence_artifacts: &[Artifact],
        config: &AnalysisConfig,
    ) -> Result<AlignmentResult, AnalysisError>;
}

/// Trait for analyzing project health and detecting gaps
#[async_trait]
pub trait GapAnalyzer: Send + Sync {
    /// Find gaps in coverage, alignment, or consistency
    async fn analyze_gaps(
        &self,
        claims: &[Claim],
        alignments: &[Alignment],
        relationships: &[Relationship],
        config: &AnalysisConfig,
    ) -> Result<Vec<Gap>, AnalysisError>;
    
    /// Generate overall project health metrics
    async fn calculate_project_health(
        &self,
        claims: &[Claim],
        alignments: &[Alignment],
        gaps: &[Gap],
    ) -> Result<ProjectHealth, AnalysisError>;
}

/// Strategy pattern for different claim extraction approaches
#[derive(Debug, Clone)]
pub enum ClaimExtractionStrategy {
    /// Extract functional claims from code (what the code does)
    CodeFunctional,
    /// Extract architectural claims from code (how the code is structured)
    CodeArchitectural,
    /// Extract test coverage claims from test files
    TestCoverage,
    /// Extract behavior claims from test descriptions
    TestBehavior,
    /// Extract requirement claims from documentation
    DocumentationRequirements,
    /// Extract API claims from documentation
    DocumentationApi,
    /// Extract change claims from commit messages
    CommitChanges,
    /// Extract business requirement claims from tickets
    TicketRequirements,
    /// Extract acceptance criteria from tickets
    TicketAcceptance,
}

/// Context information for guiding claim extraction
#[derive(Debug, Clone)]
pub struct ExtractionContext {
    /// Related artifacts that might provide context
    pub related_artifacts: Vec<Artifact>,
    /// Project-specific terminology or domain knowledge
    pub domain_context: HashMap<String, String>,
    /// Previous claims from similar artifacts for consistency
    pub similar_claims: Vec<Claim>,
    /// Extraction strategy to use
    pub strategy: ClaimExtractionStrategy,
}

/// Enhanced claim extractor that uses context for better results
#[async_trait]
pub trait ContextualClaimExtractor: ClaimExtractor {
    /// Extract claims with additional context for better accuracy
    async fn extract_claims_with_context(
        &self,
        artifact: &Artifact,
        context: &ExtractionContext,
        config: &AnalysisConfig,
    ) -> Result<ClaimExtractionResult, AnalysisError>;
}

/// Prompt templates for different types of analysis
#[derive(Debug, Clone)]
pub struct PromptTemplates {
    pub code_functional_extraction: String,
    pub test_behavior_extraction: String,
    pub documentation_requirements: String,
    pub commit_changes: String,
    pub relationship_discovery: String,
    pub alignment_checking: String,
}

impl Default for PromptTemplates {
    fn default() -> Self {
        Self {
            code_functional_extraction: "Extract functional claims from this code...".to_string(),
            test_behavior_extraction: "Extract behavior claims from these tests...".to_string(),
            documentation_requirements: "Extract requirement claims from this documentation...".to_string(),
            commit_changes: "Extract change claims from this commit message...".to_string(),
            relationship_discovery: "Find relationships between these artifacts...".to_string(),
            alignment_checking: "Check alignment between claim and evidence...".to_string(),
        }
    }
}

/// Configuration for LLM-based analysis
#[derive(Debug, Clone)]
pub struct LlmAnalysisConfig {
    /// Model to use for analysis
    pub model: String,
    /// Maximum tokens per analysis request
    pub max_tokens: u32,
    /// Temperature for analysis (lower = more consistent)
    pub temperature: f32,
    /// Custom prompt templates
    pub prompt_templates: PromptTemplates,
    /// Whether to use structured output format
    pub use_structured_output: bool,
    /// Retry configuration for failed requests
    pub retry_config: std::collections::HashMap<String, String>,
}

impl Default for LlmAnalysisConfig {
    fn default() -> Self {
        Self {
            model: "claude-3-sonnet-20240229".to_string(),
            max_tokens: 4096,
            temperature: 0.1,
            prompt_templates: PromptTemplates::default(),
            use_structured_output: true,
            retry_config: std::collections::HashMap::new(),
        }
    }
}

/// Trait for validating analysis results before storing
pub trait AnalysisValidator {
    /// Validate extracted claims for quality and consistency
    fn validate_claims(&self, claims: &[Claim], config: &AnalysisConfig) -> Vec<String>;
    
    /// Validate discovered relationships for consistency
    fn validate_relationships(&self, relationships: &[Relationship]) -> Vec<String>;
    
    /// Validate alignment scores for reasonableness
    fn validate_alignments(&self, alignments: &[Alignment]) -> Vec<String>;
}

/// Metrics for tracking analysis performance
#[derive(Debug, Clone)]
pub struct AnalysisMetrics {
    pub total_artifacts_processed: usize,
    pub total_claims_extracted: usize,
    pub total_relationships_discovered: usize,
    pub total_alignments_checked: usize,
    pub average_processing_time_ms: f64,
    pub error_count: usize,
    pub confidence_distribution: HashMap<String, usize>, // buckets like "0.0-0.2", "0.2-0.4", etc.
}